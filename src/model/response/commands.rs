use strum_macros::{AsRefStr, EnumIter, EnumString, IntoStaticStr};

use serenity::model::user::User;

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
