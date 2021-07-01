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

use std::str::FromStr;

use anyhow::Context;
use boolinator::Boolinator;
use lazy_regex::{lazy_regex, regex_captures, Regex};
use once_cell::sync::Lazy;
use strum::IntoEnumIterator;

use crate::{
    concepts::SameAs,
    data::{Monster, QuestID, Weapon},
    error::CommandError,
    model::response::Choices,
};

static QUEST_ID_REGEX: Lazy<Regex> = lazy_regex!("([1-9])-([0-9])+");

pub struct Validated<'a, Args, T>
where
    Args: Iterator,
    <Args as Iterator>::Item: Into<String>,
{
    accepted: &'a Args,
    _type: std::marker::PhantomData<T>,
}

impl<'a, Args> Validated<'a, Args, QuestID>
where
    Args: Clone + Iterator,
    <Args as Iterator>::Item: Clone + Into<String>,
{
    pub fn parse(&self) -> anyhow::Result<Vec<QuestID>> {
        self.accepted
            .clone()
            .map(|quest_id| -> anyhow::Result<QuestID> {
                let quest_id: String = quest_id.into();
                let (_, rank, number) = regex_captures!("([1-9])-([0-9])+", quest_id.as_str())
                    .with_context(|| anyhow::anyhow!("regex_captures failed."))?;
                Ok(QuestID(rank.parse::<u32>()?, number.parse::<u32>()?))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }
}

impl<'a, Args> Validated<'a, Args, Monster>
where
    Args: Clone + Iterator,
    <Args as Iterator>::Item: Into<String>,
{
    pub fn parse(&self) -> anyhow::Result<Vec<Monster>> {
        self.accepted
            .clone()
            .map(|monster| {
                let monster: String = monster.into();
                Monster::from_str(monster.as_str())
                    .with_context(|| anyhow::anyhow!("parse failed."))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }
}

impl<'a, Args> Validated<'a, Args, Weapon>
where
    Args: Clone + Iterator,
    <Args as Iterator>::Item: Into<String>,
{
    pub fn parse(&self) -> anyhow::Result<Vec<Weapon>> {
        self.accepted
            .clone()
            .map(|weapon| {
                let weapon: String = weapon.into();
                Weapon::from_str(weapon.as_str()).with_context(|| anyhow::anyhow!("parse failed."))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }
}

pub trait ValidateFor<Type> {
    fn validate_for<T>(&self) -> anyhow::Result<Validated<Self, T>>
    where
        Self: Iterator + Sized,
        <Self as Iterator>::Item: Into<String>,
        T: SameAs<Type>;
}

pub trait Validator {
    fn validate(&self, choice: Choices) -> anyhow::Result<Vec<String>>
    where
        Self: Iterator + Sized,
        <Self as Iterator>::Item: Into<String>;
}

impl<Args> Validator for Args
where
    Args: Clone + Iterator,
    <Args as Iterator>::Item: Into<String>,
    String: From<<Args as Iterator>::Item>,
{
    fn validate(&self, choice: Choices) -> anyhow::Result<Vec<String>> {
        match choice {
            Choices::Quest => Ok(self
                .validate_for::<QuestID>()?
                .accepted
                .clone()
                .map(String::from)
                .collect::<Vec<_>>()),
            Choices::Monster => Ok(self
                .validate_for::<Monster>()?
                .accepted
                .clone()
                .map(String::from)
                .collect::<Vec<_>>()),
            Choices::Weapon => Ok(self
                .validate_for::<Weapon>()?
                .accepted
                .clone()
                .map(String::from)
                .collect::<Vec<_>>()),
        }
    }
}

impl<Args> ValidateFor<QuestID> for Args
where
    Args: Clone + Iterator,
    <Args as Iterator>::Item: Into<String>,
{
    fn validate_for<T>(&self) -> anyhow::Result<Validated<Args, T>>
    where
        T: SameAs<QuestID>,
    {
        self.clone()
            .all(|quest_id| {
                let quest_id: String = quest_id.into();
                QUEST_ID_REGEX.is_match(quest_id.as_str())
            })
            .as_result_from(
                || Validated {
                    accepted: self,
                    _type: Default::default(),
                },
                || {
                    let invalid_args = self
                        .clone()
                        .filter_map(|quest_id| {
                            let quest_id: String = quest_id.into();
                            (!QUEST_ID_REGEX.is_match(quest_id.as_str())).as_some(quest_id)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    anyhow::Error::from(CommandError::InvalidArgument { arg: invalid_args })
                },
            )
    }
}

impl<Args> ValidateFor<Monster> for Args
where
    Args: Clone + Iterator,
    <Args as Iterator>::Item: Into<String>,
{
    fn validate_for<T>(&self) -> anyhow::Result<Validated<Args, T>>
    where
        T: SameAs<Monster>,
    {
        self.clone()
            .all(|monster| {
                let monster: String = monster.into();
                Monster::iter()
                    .map(|ref x| x.ja())
                    .any(|x| x == monster.as_str())
            })
            .as_result_from(
                || Validated {
                    accepted: self,
                    _type: Default::default(),
                },
                || {
                    let invalid_args = self
                        .clone()
                        .filter_map(|monster| {
                            let monster: String = monster.into();
                            (!Monster::iter()
                                .map(|ref x| x.ja())
                                .any(|x| x == monster.as_str()))
                            .as_some(monster)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    anyhow::Error::from(CommandError::InvalidArgument { arg: invalid_args })
                },
            )
    }
}

impl<Args> ValidateFor<Weapon> for Args
where
    Args: Clone + Iterator,
    <Args as Iterator>::Item: Into<String>,
{
    fn validate_for<T>(&self) -> anyhow::Result<Validated<Args, T>>
    where
        T: SameAs<Weapon>,
    {
        let keys: Vec<_> = Weapon::iter().map(|weapon| weapon.into()).collect();
        self.clone()
            .all(|weapon_key| {
                let weapon_key: String = weapon_key.into();
                keys.contains(&weapon_key.as_str())
            })
            .as_result_from(
                || Validated {
                    accepted: self,
                    _type: Default::default(),
                },
                || {
                    let invalid_args = self
                        .clone()
                        .filter_map(|weapon_key| {
                            let weapon_key: String = weapon_key.into();
                            (!keys.contains(&weapon_key.as_str())).as_some(weapon_key)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    anyhow::Error::from(CommandError::InvalidArgument { arg: invalid_args })
                },
            )
    }
}
