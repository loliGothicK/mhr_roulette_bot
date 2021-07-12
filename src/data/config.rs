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

use crate::data::{Monster, QuestID, Weapon};
use serde_derive::{Deserialize, Serialize};
use serenity::model::prelude::User;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub members: HashSet<User>,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub ranks: TargetRank,
    pub target: Target,
    pub excluded: Excluded,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetRank {
    pub ranks: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    pub quest: HashSet<QuestID>,
    pub monster: HashSet<Monster>,
    pub weapon: HashSet<Weapon>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Excluded {
    pub quest: HashSet<QuestID>,
    pub monster: HashSet<Monster>,
    pub weapon: HashSet<Weapon>,
}
