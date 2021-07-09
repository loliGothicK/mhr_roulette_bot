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
use itertools::{zip, Itertools};
use rand::{
    distributions::{Distribution, Uniform},
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};
use serenity::{builder::CreateEmbed, utils::Colour};
use strum::IntoEnumIterator;

use crate::{
    data::{Monster, Order, Range, Weapon},
    error::{CommandError, QueryError},
    executors::utility::JobStatus,
    global::{CONFIG, CONN, OBJECTIVES, QUESTS},
    model::{
        request::{Message, Request},
        response::{Choices, Response},
        translate::TranslateTo,
    },
};
use roulette_macros::bailout;
use serenity::model::user::User;
use sqlite::Connection;
use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
use thiserror::Error;

enum GenerateType {
    Quest,
    Monster,
}

pub fn generate(items: &[Response]) -> anyhow::Result<Request> {
    match items {
        [opt] => match opt.clone().translate_to::<Choices>()? {
            Choices::Quest => generate_impl(GenerateType::Quest),
            Choices::Monster => generate_impl(GenerateType::Monster),
            _ => Err(anyhow::anyhow!("unknown command option: {:?}", opt)),
        },
        _ => Err(anyhow::anyhow!("invalid : {:?}", items)),
    }
}

fn generate_impl(gen_type: GenerateType) -> anyhow::Result<Request> {
    let mut rng = thread_rng();
    let config = CONFIG.lock().unwrap();
    let members: Vec<_> = config.members.iter().choose_multiple(&mut rng, 4);
    let weapons: Vec<Weapon> = Weapon::iter().collect();
    let order_num = 5 - members.len();
    let orders = Order::iter()
        .choose_multiple(&mut rng, order_num)
        .into_iter()
        .map(|order| format!("* {order}"))
        .join("\n");
    let regulations = zip(
        members.into_iter(),
        Uniform::new(0, weapons.len())
            .sample_iter(&mut rng)
            .map(|idx| weapons[idx as usize]),
    )
    .collect_vec();
    let general_objectives: Vec<Order> = Order::iter().collect();
    let objectives = regulations
        .iter()
        .map(|(_, w)| w)
        .choose_multiple(&mut rng, 5 - order_num)
        .into_iter()
        .map(|weapon| {
            OBJECTIVES
                .get(weapon)
                .map(|objectives| {
                    let engine = Uniform::new(0usize, objectives.len());
                    let order = &objectives[engine.sample(&mut rng)];
                    Ok(format!("* {order}"))
                })
                .unwrap_or_else(|| -> anyhow::Result<String> {
                    let order = general_objectives
                        .choose(&mut rng)
                        .with_context(|| anyhow::anyhow!("failed to choose."))?;
                    Ok(format!("* {order}"))
                })
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .join("\n");
    let response = match gen_type {
        GenerateType::Quest => {
            let Range { lower, upper } = config.settings.range;
            let quest = QUESTS[lower..upper]
                .choose(&mut rng)
                .map(|quests| quests.choose(&mut rng))
                .flatten()
                .with_context(|| anyhow::anyhow!("failed to choose."))?;
            let mut embed = CreateEmbed::default();
            embed
                .colour(Colour::BLUE)
                .title(quest.title())
                .field("Mandatory Order(s)", quest.objective(), false)
                .field("Optional Orders", orders + "\n" + &objectives, false)
                .fields(
                    regulations
                        .iter()
                        .map(|(user, weapon)| (&user.name, weapon.ja(), true)),
                );
            Ok(Request::Message(Message::Embed(embed)))
        }
        GenerateType::Monster => {
            let monster = Monster::iter()
                .choose(&mut rng)
                .with_context(|| anyhow::anyhow!("failed to choose."))?;
            let mut embed = CreateEmbed::default();
            embed
                .colour(Colour::BLUE)
                .title(monster.ja())
                .field("Optional Orders", orders + "\n" + &objectives, false)
                .fields(
                    regulations
                        .iter()
                        .map(|(user, weapon)| (&user.name, weapon.ja(), true)),
                );
            Ok(Request::Message(Message::Embed(embed)))
        }
    };
    let regulations = regulations
        .into_iter()
        .map(|(user, weapon)| (user.clone(), weapon))
        .collect_vec();
    store(regulations)?;
    response
}

enum QueryKind {
    InsertIntoLogs,
    UpsetStatistics,
}

#[derive(Debug, Error)]
enum Query {
    #[error("INSERT INTO logs (id, weapon) VALUES ({id:?}, {weapon:?})")]
    InsertIntoLogs { id: u64, weapon: String },
    #[error(
        r#"
        INSERT INTO statistics (id, {weapon:?}) VALUES ({id:?}, 1)
            ON CONFLICT (id)
                DO UPDATE SET
                    {weapon:?} = {weapon:?} + 1
    "#
    )]
    UpsetStatistics { id: u64, weapon: String },
}

fn store(data: Vec<(User, Weapon)>) -> anyhow::Result<()> {
    let pair = Arc::new((Mutex::new(JobStatus::Pending), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conn = Arc::clone(&*CONN);

    let handle = thread::spawn(move || -> anyhow::Result<()> {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut conn) = conn.try_lock() {
                let mut status = lock.lock().unwrap();

                // First, we should insert results into logs.
                if let Err((query, err)) = execute(QueryKind::InsertIntoLogs, conn, &data) {
                    *status = JobStatus::ExitFailure;
                    cvar.notify_one();
                    return Err(QueryError::FailedToStore {
                        raw: format!("{err}"),
                        query,
                    })
                    .with_context(|| anyhow::anyhow!("Query failed."));
                }

                // Second, we should upset statistics.
                if let Err((query, err)) = execute(QueryKind::UpsetStatistics, conn, &data) {
                    *status = JobStatus::ExitFailure;
                    cvar.notify_one();
                    return Err(QueryError::FailedToStore {
                        raw: format!("{err}"),
                        query,
                    })
                    .with_context(|| anyhow::anyhow!("Query failed."));
                }

                *status = JobStatus::ExitSuccess;
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
            Duration::from_millis(1000),
            |status| *status == JobStatus::Pending,
        )
        .unwrap();
    loop {
        if result.0.ne(&JobStatus::Pending) {
            break handle
                .join()
                .expect("Couldn't join on the associated thread");
        } else if result.1.timed_out() {
            bailout!(
                "TLE",
                CommandError::TimeLimitExceeded {
                    command: "generate".to_string(),
                    wait_for: Duration::from_millis(1000),
                }
            );
        }
    }
}

fn execute(
    kind: QueryKind,
    conn: &mut Connection,
    data: &[(User, Weapon)],
) -> anyhow::Result<(), (String, sqlite::Error)> {
    for (user, weapon) in data {
        match kind {
            QueryKind::InsertIntoLogs => {
                let query = Query::InsertIntoLogs {
                    id: user.id.0,
                    weapon: weapon.to_string(),
                };
                conn.execute(format!("{query}"))
                    .map_err(|err| (format!("{query}"), err))?;
            }
            QueryKind::UpsetStatistics => {
                let query = Query::UpsetStatistics {
                    id: user.id.0,
                    weapon: weapon.to_string(),
                };
                conn.execute(format!("{query}"))
                    .map_err(|err| (format!("{query}"), err))?;
            }
        }
    }
    Ok(())
}
