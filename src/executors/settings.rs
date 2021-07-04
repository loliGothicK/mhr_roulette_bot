/*
 * ISC License
 *
 * Copyright (c) 2021 Mitama Lab
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 *
 */

#![allow(clippy::nonstandard_macro_braces)]
use anyhow::Context;
use itertools::Itertools;
use serenity::model::user::User;
use std::{
    collections::HashSet,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
use thiserror::Error;

use super::utility::JobStatus;
use crate::{
    data::{Monster, QuestID, Range, Weapon},
    error::{CommandError, QueryError},
    global::{sync_all, CONFIG, CONN, QUESTS},
    model::{
        request::{Message, Request},
        response::{About, Choices, Options, Response, SettingsSubCommands},
        translate::TranslateTo,
    },
    parser::ValidateFor,
};
use roulette_macros::bailout;

use crate::concepts::SameAs;

/// # settings command
///
/// ## sub-commands
/// - info
///     - quest
///     - monster
///     - weapon
///     - members
/// - members
///     - set [user(s)]
///     - add [user]
///     - remove [user]
/// - range [lower] [upper]
/// - exclude
///     - set [choice] [item(s)]
///     - add [choice] [item]
///     - remove [choice] [item]
/// - target
///     - set [choice] [item(s)]
///     - add [choice] [item]
///     - remove [choice] [item]
/// - obliterate
///     - quest
///     - monster
///     - weapon
pub fn settings(items: &[Response]) -> anyhow::Result<Request> {
    match items.translate_to::<SettingsSubCommands>()? {
        SettingsSubCommands::Info(choice) => Ok(info(choice).unwrap()),
        SettingsSubCommands::Members(opt, ref users) => members(opt, users.to_vec()),
        SettingsSubCommands::Range(lower, upper) => range(lower, upper),
        SettingsSubCommands::Exclude(opt, choice, arg) => exclude(opt, choice, arg),
        SettingsSubCommands::Target(opt, choice, arg) => target(opt, choice, arg),
        SettingsSubCommands::Obliterate(choice) => obliterate(choice),
    }
}

/// Returns information about `choice`.
fn info(about: About) -> anyhow::Result<Request, !> {
    Ok(Request::Message(match about {
        About::Quest => {
            let settings = &CONFIG.lock().unwrap().settings;
            let target_quests = if settings.target.quest.is_empty() {
                "Target quest(s): Random\n".to_string()
            } else {
                format!(
                    "Target quest(s):\n{}",
                    settings
                        .target
                        .quest
                        .iter()
                        .map(|id| { QUESTS[id.0 as usize][id.1 as usize].title() })
                        .join("\n")
                )
            };
            let excluded_quests = if settings.excluded.quest.is_empty() {
                "Excluded quest(s): No\n".to_string()
            } else {
                format!(
                    "Excluded quest(s):\n{}",
                    settings
                        .excluded
                        .quest
                        .iter()
                        .map(|id| format!("{}-{}", id.0, id.1))
                        .join("\n")
                )
            };
            Message::String(format!(
                "Quest rank range: ★{lower} - ★{upper}\n{target}{excluded}",
                lower = settings.range.lower,
                upper = settings.range.upper,
                target = target_quests,
                excluded = excluded_quests,
            ))
        }
        About::Monster => {
            let settings = &CONFIG.lock().unwrap().settings;
            let target_monsters = if settings.target.monster.is_empty() {
                "Target monster(s): Random\n".to_string()
            } else {
                format!(
                    "Target monster(s):\n{}",
                    settings.target.monster.iter().map(Monster::ja).join("\n")
                )
            };
            let excluded_monsters = if settings.excluded.monster.is_empty() {
                "Excluded monster(s): No\n".to_string()
            } else {
                format!(
                    "Excluded monster(s):\n{}",
                    settings.excluded.monster.iter().map(Monster::ja).join("\n")
                )
            };
            Message::String(format!(
                "{target}{excluded}",
                target = target_monsters,
                excluded = excluded_monsters,
            ))
        }
        About::Weapon => {
            let settings = &CONFIG.lock().unwrap().settings;
            if settings.excluded.weapon.is_empty() {
                Message::String("Excluded weapon(s): No".to_string())
            } else {
                Message::String(format!(
                    "Excluded weapon(s):\n{}",
                    settings.excluded.weapon.iter().map(Weapon::ja).join("\n")
                ))
            }
        }
        About::Members => Message::String(format!(
            "Current members: {}",
            CONFIG.lock().unwrap().members.iter().join(", ")
        )),
    }))
}

#[derive(Debug, Error)]
enum Query {
    #[error(
        r#"
        INSERT INTO hunters (id, name) VALUES ({id:?}, {name:?})
            ON CONFLICT (id)
                DO UPDATE SET
                    name = {name:?},
                    updated_at = datetime('now', 'localtime')
    "#
    )]
    UpsetMember { id: u64, name: String },
}

/// Change current member as specified in `opt`.
fn members(opt: Options, users: Vec<User>) -> anyhow::Result<Request> {
    let pair = Arc::new((Mutex::new(JobStatus::Pending), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    let handle = thread::spawn(move || -> anyhow::Result<()> {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut config) = conf.try_lock() {
                let users: HashSet<_> = users.iter().cloned().collect();
                match opt {
                    Options::Set => {
                        config.members = users.clone();
                    }
                    Options::Add => {
                        for user in users.iter() {
                            config.members.insert(user.clone());
                        }
                    }
                    Options::Remove => {
                        for user in users.iter() {
                            config.members.remove(user);
                        }
                    }
                }

                // We should Upset members name
                let mut status = lock.lock().unwrap();
                let conn = CONN.lock().unwrap();

                for user in users.iter() {
                    let query = Query::UpsetMember {
                        id: user.id.0,
                        name: user.name.clone(),
                    };
                    if let Err(err) = conn.execute(format!("{query}")) {
                        *status = JobStatus::ExitFailure;
                        cvar.notify_one();
                        return Err(QueryError::FailedToStore {
                            raw: format!("{err}"),
                            query: format!("{query}"),
                        })
                        .with_context(|| anyhow::anyhow!("Query failed."));
                    }
                }

                let mut status = lock.lock().unwrap();
                *status = JobStatus::ExitSuccess;
                cvar.notify_one();
                break Ok(());
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(lock.lock().unwrap(), Duration::from_millis(100), |status| {
            *status == JobStatus::Pending
        })
        .unwrap();
    loop {
        if result.0.ne(&JobStatus::Pending) {
            handle.join().unwrap()?;
            break;
        } else if result.1.timed_out() {
            bailout!(
                "TLE",
                CommandError::TimeLimitExceeded {
                    command: "settings members".to_string(),
                    wait_for: Duration::from_millis(100),
                }
            );
        }
    }
    sync_all().map_err(|err| {
        anyhow::Error::from(CommandError::FailedToSync {
            command: "settings members".to_string(),
            io_error: err,
        })
        .context("sync_all failed.")
    })?;
    Ok(Request::Message(Message::String(format!(
        "members = {:?}",
        CONFIG
            .lock()
            .unwrap()
            .members
            .iter()
            .map(|user| &user.name)
            .collect::<Vec<_>>()
    ))))
}

/// Sets the range of target quest rank static_cast `[lower, upper]`.
fn range(lower: i64, upper: i64) -> anyhow::Result<Request> {
    let pair = Arc::new((Mutex::new(JobStatus::Pending), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut config) = conf.try_lock() {
                config.settings.range = Range {
                    lower: lower as usize,
                    upper: upper as usize,
                };
                let mut status = lock.lock().unwrap();
                *status = JobStatus::ExitSuccess;
                cvar.notify_one();
                break;
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(lock.lock().unwrap(), Duration::from_millis(100), |status| {
            *status == JobStatus::Pending
        })
        .unwrap();
    loop {
        if result.0.ne(&JobStatus::Pending) {
            break;
        } else if result.1.timed_out() {
            bailout!(
                "TLE",
                CommandError::TimeLimitExceeded {
                    command: "settings range".to_string(),
                    wait_for: Duration::from_millis(100),
                }
            );
        }
    }
    sync_all().map_err(|err| {
        anyhow::Error::from(CommandError::FailedToSync {
            command: "settings range".to_string(),
            io_error: err,
        })
        .context("sync_all failed.")
    })?;
    Ok(Request::Message(Message::String(
        CONFIG.lock().unwrap().settings.range.as_pretty_string(),
    )))
}

trait SmartCast<T> {
    fn smart_cast<U>(self) -> anyhow::Result<HashSet<T>>
    where
        U: SameAs<T>;
}

impl SmartCast<QuestID> for String {
    fn smart_cast<U>(self) -> anyhow::Result<HashSet<QuestID>>
    where
        U: SameAs<QuestID>,
    {
        Ok(self
            .split_whitespace()
            .validate_for::<QuestID>()?
            .parse()?
            .into_iter()
            .collect::<HashSet<_>>())
    }
}

impl SmartCast<Monster> for String {
    fn smart_cast<U>(self) -> anyhow::Result<HashSet<Monster>>
    where
        U: SameAs<Monster>,
    {
        Ok(self
            .split_whitespace()
            .validate_for::<Monster>()?
            .parse()?
            .into_iter()
            .collect::<HashSet<_>>())
    }
}

impl SmartCast<Weapon> for String {
    fn smart_cast<U>(self) -> anyhow::Result<HashSet<Weapon>>
    where
        U: SameAs<Weapon>,
    {
        Ok(self
            .split_whitespace()
            .validate_for::<Weapon>()?
            .parse()?
            .into_iter()
            .collect::<HashSet<_>>())
    }
}

/// Configure excluded quest(s)/monster(s)/weapon(s).
/// - set/add/remove: as specified in `opt`.
/// - quest(s)/monster(s)/weapon(s): as specified in `choice`.
fn exclude(opt: Options, choice: Choices, arg: String) -> anyhow::Result<Request> {
    let pair = Arc::new((Mutex::new(JobStatus::Pending), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    let handle = thread::spawn(move || -> anyhow::Result<()> {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut config) = conf.try_lock() {
                match opt {
                    Options::Set => match choice {
                        Choices::Quest => {
                            let quests = arg.smart_cast::<QuestID>()?;
                            config.settings.excluded.quest = quests;
                        }
                        Choices::Monster => {
                            let monsters = arg.smart_cast::<Monster>()?;
                            config.settings.excluded.monster = monsters;
                        }
                        Choices::Weapon => {
                            let weapons = arg.smart_cast::<Weapon>()?;
                            config.settings.excluded.weapon = weapons;
                        }
                    },
                    Options::Add => match choice {
                        Choices::Quest => {
                            for quest in arg.smart_cast::<QuestID>()? {
                                config.settings.excluded.quest.insert(quest);
                            }
                        }
                        Choices::Monster => {
                            for monster in arg.smart_cast::<Monster>()? {
                                config.settings.excluded.monster.insert(monster);
                            }
                        }
                        Choices::Weapon => {
                            for weapon in arg.smart_cast::<Weapon>()? {
                                config.settings.excluded.weapon.insert(weapon);
                            }
                        }
                    },
                    Options::Remove => match choice {
                        Choices::Quest => {
                            for quest in arg.smart_cast::<QuestID>()? {
                                config.settings.excluded.quest.remove(&quest);
                            }
                        }
                        Choices::Monster => {
                            for monster in arg.smart_cast::<Monster>()? {
                                config.settings.excluded.monster.remove(&monster);
                            }
                        }
                        Choices::Weapon => {
                            for weapon in arg.smart_cast::<Weapon>()? {
                                config.settings.excluded.weapon.remove(&weapon);
                            }
                        }
                    },
                }
                let mut status = lock.lock().unwrap();
                *status = JobStatus::ExitSuccess;
                cvar.notify_one();
                break Ok(());
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(lock.lock().unwrap(), Duration::from_millis(100), |status| {
            *status == JobStatus::Pending
        })
        .unwrap();
    loop {
        if result.0.ne(&JobStatus::Pending) {
            handle.join().unwrap()?;
            break;
        } else if result.1.timed_out() {
            bailout!(
                "TLE",
                CommandError::TimeLimitExceeded {
                    command: "settings exclude".to_string(),
                    wait_for: Duration::from_millis(100),
                }
            );
        }
    }
    sync_all().map_err(|err| {
        anyhow::Error::from(CommandError::FailedToSync {
            command: "settings exclude".to_string(),
            io_error: err,
        })
        .context("sync_all failed.")
    })?;
    Ok(Request::Message(Message::String("Done!".to_string())))
}

fn target(opt: Options, choice: Choices, arg: String) -> anyhow::Result<Request> {
    let pair = Arc::new((Mutex::new(JobStatus::Pending), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    let handle = thread::spawn(move || -> anyhow::Result<()> {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut config) = conf.try_lock() {
                match opt {
                    Options::Set => match choice {
                        Choices::Quest => {
                            let quests = arg.smart_cast::<QuestID>()?;
                            config.settings.target.quest = quests;
                        }
                        Choices::Monster => {
                            let monsters = arg.smart_cast::<Monster>()?;
                            config.settings.target.monster = monsters;
                        }
                        Choices::Weapon => {
                            let weapons = arg.smart_cast::<Weapon>()?;
                            config.settings.target.weapon = weapons;
                        }
                    },
                    Options::Add => match choice {
                        Choices::Quest => {
                            for quest in arg.smart_cast::<QuestID>()? {
                                config.settings.target.quest.insert(quest);
                            }
                        }
                        Choices::Monster => {
                            for monster in arg.smart_cast::<Monster>()? {
                                config.settings.target.monster.insert(monster);
                            }
                        }
                        Choices::Weapon => {
                            for weapon in arg.smart_cast::<Weapon>()? {
                                config.settings.target.weapon.insert(weapon);
                            }
                        }
                    },
                    Options::Remove => match choice {
                        Choices::Quest => {
                            for quest in arg.smart_cast::<QuestID>()? {
                                config.settings.target.quest.remove(&quest);
                            }
                        }
                        Choices::Monster => {
                            for monster in arg.smart_cast::<Monster>()? {
                                config.settings.target.monster.remove(&monster);
                            }
                        }
                        Choices::Weapon => {
                            for weapon in arg.smart_cast::<Weapon>()? {
                                config.settings.target.weapon.remove(&weapon);
                            }
                        }
                    },
                }
                let mut status = lock.lock().unwrap();
                *status = JobStatus::ExitSuccess;
                cvar.notify_one();
                break Ok(());
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(lock.lock().unwrap(), Duration::from_millis(100), |status| {
            *status == JobStatus::Pending
        })
        .unwrap();
    loop {
        if result.0.ne(&JobStatus::Pending) {
            handle.join().unwrap()?;
            break;
        } else if result.1.timed_out() {
            bailout!(
                "TLE",
                CommandError::TimeLimitExceeded {
                    command: "settings target".to_string(),
                    wait_for: Duration::from_millis(100),
                }
            );
        }
    }
    sync_all().map_err(|err| {
        anyhow::Error::from(CommandError::FailedToSync {
            command: "settings target".to_string(),
            io_error: err,
        })
        .context("sync_all failed.")
    })?;
    Ok(Request::Message(Message::String("Done!".to_string())))
}

fn obliterate(choice: Choices) -> anyhow::Result<Request> {
    let pair = Arc::new((Mutex::new(JobStatus::Pending), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut config) = conf.try_lock() {
                match choice {
                    Choices::Quest => {
                        config.settings.target.quest.clear();
                        config.settings.excluded.quest.clear();
                    }
                    Choices::Monster => {
                        config.settings.target.monster.clear();
                        config.settings.excluded.monster.clear();
                    }
                    Choices::Weapon => {
                        config.settings.excluded.weapon.clear();
                        config.settings.target.weapon.clear();
                    }
                }
                let mut status = lock.lock().unwrap();
                *status = JobStatus::ExitSuccess;
                cvar.notify_one();
                break;
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(lock.lock().unwrap(), Duration::from_millis(100), |status| {
            *status == JobStatus::Pending
        })
        .unwrap();
    loop {
        if result.0.ne(&JobStatus::Pending) {
            break;
        } else if result.1.timed_out() {
            bailout!(
                "TLE",
                CommandError::TimeLimitExceeded {
                    command: "settings obliterate".to_string(),
                    wait_for: Duration::from_millis(100),
                }
            );
        }
    }
    sync_all().map_err(|err| {
        anyhow::Error::from(CommandError::FailedToSync {
            command: "settings obliterate".to_string(),
            io_error: err,
        })
        .context("sync_all failed.")
    })?;
    Ok(Request::Message(Message::String("Cleared!".to_owned())))
}
