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

use serenity::model::user::User;
use strum_macros::{AsRefStr, EnumIter, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Commands {
    Version,
    Settings,
    Generate,
    Statistics,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, IntoStaticStr, EnumString, EnumIter,
)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Options {
    Set,
    Add,
    Remove,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, IntoStaticStr, EnumString, EnumIter,
)]
#[strum(serialize_all = "snake_case")]
pub enum Choices {
    Quest,
    Monster,
    Weapon,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, IntoStaticStr, EnumString, EnumIter,
)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum About {
    Quest,
    Monster,
    Weapon,
    Members,
}

#[derive(Debug)]
pub(crate) enum SettingsSubCommands {
    Info(About),
    Members(Options, Vec<User>),
    Range(i64, i64),
    Exclude(Options, Choices, String),
    Target(Options, Choices, String),
    Obliterate(Choices),
}

#[derive(Debug)]
pub(crate) enum StatisticsSubCommands {
    Help,
    Query {
        from: User,
        weapon: Option<String>,
        since: Option<String>,
        until: Option<String>,
    },
}
