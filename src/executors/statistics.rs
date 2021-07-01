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

use super::utility::JobStatus;
use crate::{
    data::Weapon,
    error::{CommandError, LogicError, QueryError},
    global::CONN,
    model::{
        request::{Message, Request},
        response::{Response, StatisticsSubCommands},
        translate::TranslateTo,
    },
};
use anyhow::Context;
use roulette_macros::{bailout, pretty_info};
use serenity::{builder::CreateEmbed, utils::Colour};

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
            "statistics <user> [weapon_keys] [since] [until]",
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

fn valid_date(date: &str, param: &str) -> anyhow::Result<String> {
    Ok(DateTime::parse_from_rfc3339(date)
        .map_err(|err| QueryError::InvalidDate {
            param: param.to_string(),
            actual: date.to_string(),
            source: err,
        })?
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
            weapons
                .contains(column)
                .as_result(
                    column.to_string(),
                    QueryError::InvalidWeapon {
                        param: "weapon_keys".to_string(),
                        actual: column.to_string(),
                    },
                )
                .with_context(|| anyhow::anyhow!("validation error."))
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
    ChargeBlade(usize),
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

impl Stat {
    fn into_field(self) -> (&'static str, usize, bool) {
        match self {
            Stat::GreatSword(n) => (Weapon::GreatSword.ja(), n, true),
            Stat::LongSword(n) => (Weapon::LongSword.ja(), n, true),
            Stat::SwordAndShield(n) => (Weapon::SwordAndShield.ja(), n, true),
            Stat::DualBlades(n) => (Weapon::DualBlades.ja(), n, true),
            Stat::Lance(n) => (Weapon::Lance.ja(), n, true),
            Stat::Gunlance(n) => (Weapon::Gunlance.ja(), n, true),
            Stat::Hammer(n) => (Weapon::Hammer.ja(), n, true),
            Stat::HuntingHorn(n) => (Weapon::HuntingHorn.ja(), n, true),
            Stat::SwitchAxe(n) => (Weapon::SwitchAxe.ja(), n, true),
            Stat::ChargeBlade(n) => (Weapon::ChargeBlade.ja(), n, true),
            Stat::InsectGlaive(n) => (Weapon::InsectGlaive.ja(), n, true),
            Stat::LightBowgun(n) => (Weapon::LightBowgun.ja(), n, true),
            Stat::HeavyBowgun(n) => (Weapon::HeavyBowgun.ja(), n, true),
            Stat::Bow(n) => (Weapon::Bow.ja(), n, true),
            Stat::TackleOnly(n) => (Weapon::TackleOnly.ja(), n, true),
            Stat::CounterOnly(n) => (Weapon::CounterOnly.ja(), n, true),
            Stat::MeleeAttackOnly(n) => (Weapon::MeleeAttackOnly.ja(), n, true),
            Stat::SkillsOnly(n) => (Weapon::SkillsOnly.ja(), n, true),
            Stat::PalamuteOnly(n) => (Weapon::PalamuteOnly.ja(), n, true),
            Stat::InsectOnly(n) => (Weapon::InsectOnly.ja(), n, true),
            Stat::BomOnly(n) => (Weapon::BomOnly.ja(), n, true),
        }
    }
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
            "charge_blade" => Stat::ChargeBlade(count),
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
            unknown => {
                let expr = stringify!(self);
                let typename = std::any::type_name_of_val(unknown);
                bailout!(
                    "Unknown weapon_key",
                    LogicError::UnreachableGuard {
                        expr: format!("{expr}: {typename}"),
                        value: unknown.to_string(),
                        info: pretty_info!(),
                    }
                );
            }
        })
    }
}

#[tracing::instrument]
fn query(
    user: User,
    weapon: Option<String>,
    since: Option<String>,
    until: Option<String>,
) -> anyhow::Result<Request> {
    let pair = Arc::new((Mutex::new(JobStatus::Pending), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let conn = Arc::clone(&*CONN);

    let handle = thread::spawn(move || -> anyhow::Result<Request> {
        let (lock, cvar) = &*pair2;
        loop {
            if let Ok(ref mut conn) = conn.try_lock() {
                let response = (|| -> anyhow::Result<Vec<Stat>> {
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
                            let begin = valid_date(&begin, "since")?;
                            let end = valid_date(&end, "until")?;
                            Some(format!("date(generated_at) BETWEEN {begin} AND {end}"))
                        }
                        (Some(begin), None) => {
                            let begin = valid_date(&begin, "since")?;
                            Some(format!("{begin} <= date(generated_at) "))
                        }
                        (None, Some(end)) => {
                            let end = valid_date(&end, "until")?;
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

                    if let Err(err) = query_result {
                        bailout!(
                            "query error",
                            QueryError::FailedToAggregate {
                                raw: format!("{err}"),
                                query
                            }
                        );
                    }

                    result.into_iter().collect::<anyhow::Result<Vec<_>>>()
                })();

                let mut status = lock.lock().unwrap();
                if let Err(err) = response {
                    *status = JobStatus::ExitFailure;
                    cvar.notify_one();
                    return Err(err);
                } else {
                    let mut embed = CreateEmbed::default();
                    embed
                        .title(user.name)
                        .fields(response?.into_iter().map(|stat| stat.into_field()));
                    *status = JobStatus::ExitSuccess;
                    cvar.notify_one();
                    break Ok(Request::Message(Message::Embed(embed)));
                }
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
                    command: "statistics query".to_string(),
                    wait_for: Duration::from_millis(1000),
                }
            );
        }
    }
}
