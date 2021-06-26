use itertools::Itertools;
use anyhow::Context;
use serenity::model::user::User;
use std::collections::HashSet;
use std::sync::{Arc, Condvar, Mutex};
use std::{thread, time::Duration};
use thiserror::Error;

use crate::data::{Pick, Range};
use crate::global::{sync_all, CONFIG, CONN};
use crate::model::request::{Message, Request};
use crate::model::response::{About, Choices, Options, Response, SettingsSubCommands};
use crate::model::translate::TranslateTo;
use crate::parser::Validator;

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
                    settings.target.quest.iter().join("\n")
                )
            };
            let excluded_quests = if settings.excluded.quest.is_empty() {
                "Excluded quest(s): No\n".to_string()
            } else {
                format!(
                    "Excluded quest(s):\n{}",
                    settings.excluded.quest.iter().join("\n")
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
                    settings.target.monster.iter().join("\n")
                )
            };
            let excluded_monsters = if settings.excluded.monster.is_empty() {
                "Excluded monster(s): No\n".to_string()
            } else {
                format!(
                    "Excluded monster(s):\n{}",
                    settings.excluded.monster.iter().join("\n")
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
                    settings.excluded.weapon.iter().join("\n")
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
    #[error(r#"
        INSERT INTO hunters (id, name) VALUES ({id:?}, {name:?})
            ON CONFLICT (id)
                DO UPDATE SET
                    name = {name:?},
                    updated_at = datetime('now', 'localtime')
    "#)]
    UpsetMember {
        id: u64,
        name: String,
    }
}

/// Change current member as specified in `opt`.
fn members(opt: Options, users: Vec<User>) -> anyhow::Result<Request> {
    #[allow(clippy::mutex_atomic)]
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    thread::spawn(move || -> anyhow::Result<()> {
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
                let mut pending = lock.lock().unwrap();
                let conn = CONN.lock().unwrap();

                for user in users.iter() {
                    let query = Query::UpsetMember { id: user.id.0, name: user.name.clone() };
                    if let Err(err) = conn.execute(format!("{query}"))
                        .with_context(|| anyhow::anyhow!("Failed to query with `{query}`"))
                    {
                        *pending = false;
                        cvar.notify_one();
                        return Err(err);
                    }
                }

                let mut pending = lock.lock().unwrap();
                *pending = false;
                cvar.notify_one();
                break Ok(());
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(
            lock.lock().unwrap(),
            Duration::from_millis(100),
            |&mut pending| pending,
        )
        .unwrap();
    if result.1.timed_out() {
        anyhow::bail!("thread timeout");
    }
    sync_all()?;
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

/// Sets the range of target quest rank into `[lower, upper]`.
fn range(lower: i64, upper: i64) -> anyhow::Result<Request> {
    #[allow(clippy::mutex_atomic)]
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
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
                let mut pending = lock.lock().unwrap();
                *pending = false;
                cvar.notify_one();
                break;
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(
            lock.lock().unwrap(),
            Duration::from_millis(100),
            |&mut pending| pending,
        )
        .unwrap();
    if result.1.timed_out() {
        anyhow::bail!("thread timeout");
    }
    sync_all()?;
    Ok(Request::Message(Message::String(
        CONFIG.lock().unwrap().settings.range.as_pretty_string(),
    )))
}

/// Configure excluded quest(s)/monster(s)/weapon(s).
/// - set/add/remove: as specified in `opt`.
/// - quest(s)/monster(s)/weapon(s): as specified in `choice`.
fn exclude(opt: Options, choice: Choices, arg: String) -> anyhow::Result<Request> {
    let args: HashSet<_> = arg
        .split_whitespace()
        .validate(choice)?
        .into_iter()
        .collect();
    #[allow(clippy::mutex_atomic)]
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut config) = conf.try_lock() {
                match opt {
                    Options::Set => {
                        *config.settings.excluded.pick_mut(&choice) = args;
                    }
                    Options::Add => {
                        for arg in args {
                            config.settings.excluded.pick_mut(&choice).insert(arg);
                        }
                    }
                    Options::Remove => {
                        for arg in args {
                            config.settings.excluded.pick_mut(&choice).remove(&arg);
                        }
                    }
                }
                let mut pending = lock.lock().unwrap();
                *pending = false;
                cvar.notify_one();
                break;
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(
            lock.lock().unwrap(),
            Duration::from_millis(100),
            |&mut pending| pending,
        )
        .unwrap();
    if result.1.timed_out() {
        anyhow::bail!("thread timeout");
    }
    sync_all()?;
    Ok(Request::Message(Message::String("Done!".to_string())))
}

fn target(opt: Options, choice: Choices, arg: String) -> anyhow::Result<Request> {
    #[allow(clippy::mutex_atomic)]
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conf = Arc::clone(&*CONFIG);
    thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut config) = conf.try_lock() {
                let args: HashSet<_> = arg.split_whitespace().map(String::from).collect();
                match opt {
                    Options::Set => {
                        *config.settings.target.pick_mut(&choice) = args;
                    }
                    Options::Add => {
                        for arg in args {
                            config.settings.target.pick_mut(&choice).insert(arg);
                        }
                    }
                    Options::Remove => {
                        for arg in args {
                            config.settings.target.pick_mut(&choice).remove(&arg);
                        }
                    }
                }
                let mut pending = lock.lock().unwrap();
                *pending = false;
                cvar.notify_one();
                break;
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(
            lock.lock().unwrap(),
            Duration::from_millis(100),
            |&mut pending| pending,
        )
        .unwrap();
    if result.1.timed_out() {
        anyhow::bail!("thread timeout");
    }
    sync_all()?;
    Ok(Request::Message(Message::String("Done!".to_string())))
}

fn obliterate(choice: Choices) -> anyhow::Result<Request> {
    #[allow(clippy::mutex_atomic)]
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
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
                let mut pending = lock.lock().unwrap();
                *pending = false;
                cvar.notify_one();
                break;
            }
        }
    });
    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let result = cvar
        .wait_timeout_while(
            lock.lock().unwrap(),
            Duration::from_millis(100),
            |&mut pending| pending,
        )
        .unwrap();
    if result.1.timed_out() {
        anyhow::bail!("thread timeout");
    }
    sync_all()?;
    Ok(Request::Message(Message::String("Cleared!".to_owned())))
}
