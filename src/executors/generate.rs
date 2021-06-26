use anyhow::Context;
use itertools::{zip, Itertools};
use rand::distributions::{Distribution, Uniform};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::thread_rng;
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;
use strum::IntoEnumIterator;

use crate::data::{Monster, Order, QuestInfo, Range, Weapon};
use crate::global::{CONFIG, CONN, OBJECTIVES, QUESTS};
use crate::model::request::{Message, Request};
use crate::model::response::{Choices, Response};
use crate::model::translate::TranslateTo;
use serenity::model::user::User;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use sqlite::{Connection};
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
    let general_objectives: Vec<Order> = Order::iter().collect();
    let objectives = weapons
        .choose_multiple(&mut rng, 5 - order_num)
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
    let regulations = zip(
        members.into_iter(),
        Uniform::new(0, weapons.len())
            .sample_iter(&mut rng)
            .map(|idx| weapons[idx as usize]),
    )
    .collect_vec();
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
    InsertIntoLogs {
        id: u64,
        weapon: String,
    },
    #[error(r#"
        INSERT INTO statistics (id, {weapon:?}) VALUES ({id:?}, 1)
            ON CONFLICT (id)
                DO UPDATE SET
                    {weapon:?} = {weapon:?} + 1
    "#)]
    UpsetStatistics {
        id: u64,
        weapon: String,
    }
}

fn store(data: Vec<(User, Weapon)>) -> anyhow::Result<()> {
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conn = Arc::clone(&*CONN);

    let handle = thread::spawn(move || -> anyhow::Result<()> {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut conn) = conn.try_lock() {
                let mut pending = lock.lock().unwrap();

                // First, we should insert results into logs.
                if let Err(err) = execute(QueryKind::InsertIntoLogs, conn, &data) {
                    *pending = false;
                    cvar.notify_one();
                    anyhow::bail!("Fail to query: {:?}", err);
                }

                // Second, we should upset statistics.
                if let Err(err) = execute(QueryKind::UpsetStatistics, conn, &data) {
                    *pending = false;
                    cvar.notify_one();
                    anyhow::bail!("Fail to query: {:?}", err);
                }

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
            Duration::from_millis(1000),
            |&mut pending| pending,
        )
        .unwrap();
    if result.1.timed_out() {
        if *result.0 {
            if let Err(err) = handle.join() {
                anyhow::bail!("thread timeout: {:?}", err);
            }
        }
        anyhow::bail!("thread timeout in progress");
    }
    handle.join().unwrap()
}

fn execute(kind: QueryKind, conn: &mut Connection, data: &[(User, Weapon)]) -> anyhow::Result<()> {
    let exe = || -> anyhow::Result<()> {
        for (user, weapon) in data {
            match kind {
                QueryKind::InsertIntoLogs => {
                    let query = Query::InsertIntoLogs { id: user.id.0, weapon: weapon.to_string() };
                    conn.execute(format!("{query}"))?;
                }
                QueryKind::UpsetStatistics => {
                    let query = Query::UpsetStatistics { id: user.id.0, weapon: weapon.to_string() };
                    conn.execute(format!("{query}"))?;
                }
            }
        }
        Ok(())
    };
    exe().with_context(|| anyhow::anyhow!("Failed to query with `{query}`"))
}
