#![allow(clippy::mutex_atomic)]

use boolinator::Boolinator;
use chrono::DateTime;
use indoc::indoc;
use itertools::Itertools;
use serenity::model::user::User;
use std::{
    fmt::Debug,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
use strum::IntoEnumIterator;

use crate::{
    data::Weapon,
    global::{CONN, SRX},
    model::{
        request::{Message, Request},
        response::{Response, StatisticsSubCommands},
        translate::TranslateTo,
    },
    stream::Msg,
};
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;

pub fn statistics(items: &[(String, Response)]) -> anyhow::Result<Request> {
    match items.translate_to::<StatisticsSubCommands>()? {
        StatisticsSubCommands::Help => Ok(help()?),
        StatisticsSubCommands::Query {
            from,
            weapon,
            since,
            until,
        } => query(from, weapon, since, until),
    }
}

fn help() -> anyhow::Result<Request, !> {
    let mut embed = CreateEmbed::default();
    embed
        .colour(Colour::MEIBE_PINK)
        .title("statistics")
        .description("You can find out how many times a hunter has used a certain weapon type.")
        .field(
            "Usage:",
            "statistics <user> [weapon_key] [since] [until]",
            false,
        )
        .field(
            "weapon keys:",
            Weapon::iter()
                .map(|key| key.to_string())
                .collect_vec()
                .join("\n"),
            true,
        )
        .field(
            "since:",
            "YYYY-MM-DD: Beginning of the period to be covered.",
            true,
        )
        .field(
            "until:",
            "YYYY-MM-DD: End of the period to be covered.",
            true,
        );
    Ok(Request::Message(Message::Embed(embed)))
}

fn valid_date(date: &str) -> anyhow::Result<String> {
    Ok(DateTime::parse_from_rfc3339(date)?
        .date()
        .format("%Y-%m-%d")
        .to_string())
}

fn valid_weapon(columns: &str) -> anyhow::Result<String> {
    let columns = columns.split(',').map(|column| column.trim()).collect_vec();
    let weapons: Vec<&'static str> = Weapon::iter()
        .map(|weapon| {
            let str: &'static str = weapon.into();
            str
        })
        .collect();
    columns
        .iter()
        .map(|column| {
            weapons.contains(column).as_result(
                column.to_string(),
                anyhow::anyhow!("invalid weapon key: {}", column),
            )
        })
        .collect::<anyhow::Result<Vec<_>>>()
        .map(|weapons| weapons.join(", "))
}

#[derive(Debug, Clone, Copy)]
struct Counter {
    count: i32,
}

#[derive(Debug, Clone, Copy)]
enum Stat {
    GreatSword(usize),
    LongSword(usize),
    SwordAndShield(usize),
    DualBlades(usize),
    Lance(usize),
    Gunlance(usize),
    Hammer(usize),
    HuntingHorn(usize),
    SwitchAxe(usize),
    ChargeLade(usize),
    InsectGlaive(usize),
    LightBowgun(usize),
    HeavyBowgun(usize),
    Bow(usize),
    TackleOnly(usize),
    CounterOnly(usize),
    MeleeAttackOnly(usize),
    SkillsOnly(usize),
    PalamuteOnly(usize),
    InsectOnly(usize),
    BomOnly(usize),
}

trait IntoStat {
    fn into_stat_with(self, count: usize) -> anyhow::Result<Stat>;
}

impl IntoStat for &str {
    fn into_stat_with(self, count: usize) -> anyhow::Result<Stat> {
        Ok(match self {
            "great_sword" => Stat::GreatSword(count),
            "long_sword" => Stat::LongSword(count),
            "sword_and_shield" => Stat::SwordAndShield(count),
            "dual_blades" => Stat::DualBlades(count),
            "lance" => Stat::Lance(count),
            "gunlance" => Stat::Gunlance(count),
            "hammer" => Stat::Hammer(count),
            "hunting_horn" => Stat::HuntingHorn(count),
            "switch_axe" => Stat::SwitchAxe(count),
            "charge_blade" => Stat::ChargeLade(count),
            "insect_glaive" => Stat::InsectGlaive(count),
            "light_bowgun" => Stat::LightBowgun(count),
            "heavy_bowgun" => Stat::HeavyBowgun(count),
            "bow" => Stat::Bow(count),
            "tackle_only" => Stat::TackleOnly(count),
            "counter_only" => Stat::CounterOnly(count),
            "melee_attack_only" => Stat::MeleeAttackOnly(count),
            "skills_only" => Stat::SkillsOnly(count),
            "palamute_only" => Stat::PalamuteOnly(count),
            "insect_only" => Stat::InsectOnly(count),
            "bom_only" => Stat::BomOnly(count),
            unknown => anyhow::bail!("unknown weapon: {}", unknown),
        })
    }
}

fn query(
    user: User,
    weapon: Option<String>,
    since: Option<String>,
    until: Option<String>,
) -> anyhow::Result<Request> {
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conn = Arc::clone(&*CONN);

    let handle = thread::spawn(move || -> anyhow::Result<Request> {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut conn) = conn.try_lock() {
                let weapon = weapon.map_or_else(
                    || {
                        Ok(Weapon::iter()
                            .map(|weapon| weapon.to_string())
                            .collect_vec()
                            .join(", "))
                    },
                    |columns| valid_weapon(&columns),
                )?;
                let date = match (since, until) {
                    (Some(begin), Some(end)) => {
                        let begin = valid_date(&begin)?;
                        let end = valid_date(&end)?;
                        Some(format!("date(generated_at) BETWEEN {begin} AND {end}"))
                    }
                    (Some(begin), None) => {
                        let begin = valid_date(&begin)?;
                        Some(format!("{begin} <= date(generated_at) "))
                    }
                    (None, Some(end)) => {
                        let end = valid_date(&end)?;
                        Some(format!("date(generated_at) <= {end}"))
                    }
                    _ => None,
                };
                let table = if date.is_some() { "logs" } else { "statistics" }.to_string();
                let id = user.id.0;
                let query = format!(
                    indoc! {r#"
                        SELECT {weapon}
                        FROM {table}
                        WHERE
                            id = '{id}'
                            {date}
                    "#},
                    weapon = weapon,
                    table = table,
                    id = id,
                    date = date.unwrap_or_default()
                );

                let mut result = Vec::new();
                let query_result = conn.iterate(&query, |pairs| {
                    for &(column, value) in pairs.iter() {
                        if let Some(Ok(count)) = value.map(|v| v.parse::<usize>()) {
                            result.push(column.into_stat_with(count))
                        }
                    }
                    true
                });

                let mut pending = lock.lock().unwrap();

                if let Err(err) = query_result {
                    *pending = false;
                    cvar.notify_one();
                    anyhow::bail!("Fail to query: {:?}", err);
                }

                *pending = false;
                cvar.notify_one();
                let tx = SRX.sender();
                let _detached = thread::spawn(async move || {
                    let _ = tx
                        .send(Msg::Event {
                            title: "SQLite Query".to_owned(),
                            description: Some(query),
                        })
                        .await;
                });

                let response = result.into_iter().collect::<anyhow::Result<Vec<_>>>()?;
                break Ok(Request::Message(Message::String(format!("{response:?}"))));
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

struct ExitGuard {
    finally: Box<dyn FnMut()>,
}

impl Drop for ExitGuard {
    fn drop(&mut self) {
        (self.finally)();
    }
}
