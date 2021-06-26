use super::commands::*;
use super::{Response, SlashCommand};
use crate::concepts::SameAs;
use crate::model::translate::TranslateTo;
use anyhow::Context;
use serenity::model::{channel::PartialChannel, guild::Role, user::User};
use std::collections::HashMap;

type OptionValue = serenity::model::interactions::ApplicationCommandInteractionDataOptionValue;

impl<Target> TranslateTo<Vec<Target>> for &[Response]
where
    Response: TranslateTo<Target>,
{
    fn translate_to<T>(&self) -> anyhow::Result<Vec<Target>>
    where
        T: SameAs<Vec<Target>>,
    {
        Ok(self
            .iter()
            .filter_map(|item| item.translate_to::<Target>().ok())
            .collect())
    }
}

impl TranslateTo<String> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<String>
    where
        T: SameAs<String>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::String(value) = &**boxed {
                return Ok(value.clone());
            }
        }
        Err(anyhow::anyhow!("cannot transmute to String: {:?}", &self))
    }
}

impl TranslateTo<i64> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<i64>
    where
        T: SameAs<i64>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::Integer(value) = **boxed {
                return Ok(value);
            }
        }
        Err(anyhow::anyhow!("cannot transmute to String: {:?}", &self))
    }
}

impl TranslateTo<User> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<User>
    where
        T: SameAs<User>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::User(user, _) = &**boxed {
                return Ok(user.clone());
            }
        }
        Err(anyhow::anyhow!("cannot transmute to String: {:?}", &self))
    }
}

impl TranslateTo<Role> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<Role>
    where
        T: SameAs<Role>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::Role(role) = &**boxed {
                return Ok(role.clone());
            }
        }
        Err(anyhow::anyhow!("cannot transmute to String: {:?}", &self))
    }
}

impl TranslateTo<PartialChannel> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<PartialChannel>
    where
        T: SameAs<PartialChannel>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::Channel(p_channel) = &**boxed {
                return Ok(p_channel.clone());
            }
        }
        Err(anyhow::anyhow!("cannot transmute to String: {:?}", &self))
    }
}

impl TranslateTo<Commands> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<Commands>
    where
        T: SameAs<Commands>,
    {
        match self {
            Response::SlashCommand(SlashCommand::Command(cmd)) if cmd == "version" => {
                Ok(Commands::Version)
            }
            Response::SlashCommand(SlashCommand::Command(cmd)) if cmd == "settings" => {
                Ok(Commands::Settings)
            }
            Response::SlashCommand(SlashCommand::Command(cmd)) if cmd == "generate" => {
                Ok(Commands::Generate)
            }
            Response::SlashCommand(SlashCommand::Command(cmd)) if cmd == "statistics" => {
                Ok(Commands::Statistics)
            }
            unknown => Err(anyhow::anyhow!(
                "ERROR: cannot transmute to Commands {:?}",
                unknown
            )),
        }
    }
}

impl TranslateTo<Options> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<Options>
    where
        T: SameAs<Options>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::String(opt) = &**boxed {
                return match &opt[..] {
                    "set" => Ok(Options::Set),
                    "add" => Ok(Options::Add),
                    "remove" => Ok(Options::Remove),
                    _ => anyhow::bail!("ERROR: cannot transmute: {}", opt),
                };
            }
        }
        Err(anyhow::anyhow!("ERROR: cannot transmute: {:?}", &self))
    }
}

impl TranslateTo<Choices> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<Choices>
    where
        T: SameAs<Choices>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::String(opt) = &**boxed {
                return match &opt[..] {
                    "quest" => Ok(Choices::Quest),
                    "monster" => Ok(Choices::Monster),
                    "weapon" => Ok(Choices::Weapon),
                    _ => anyhow::bail!("ERROR: cannot transmute: {}", opt),
                };
            }
        }
        Err(anyhow::anyhow!("ERROR: cannot transmute: {:?}", &self))
    }
}

impl TranslateTo<About> for Response {
    fn translate_to<T>(&self) -> anyhow::Result<About>
    where
        T: SameAs<About>,
    {
        if let Response::SlashCommand(SlashCommand::Option(boxed)) = self {
            if let OptionValue::String(opt) = &**boxed {
                return match &opt[..] {
                    "quest" => Ok(About::Quest),
                    "monster" => Ok(About::Monster),
                    "weapon" => Ok(About::Weapon),
                    "members" => Ok(About::Members),
                    _ => anyhow::bail!("ERROR: cannot transmute: {}", opt),
                };
            }
        }
        Err(anyhow::anyhow!("ERROR: cannot transmute: {:?}", &self))
    }
}

impl TranslateTo<SettingsSubCommands> for &[Response] {
    fn translate_to<T>(&self) -> anyhow::Result<SettingsSubCommands>
    where
        T: SameAs<SettingsSubCommands>,
    {
        match self {
            [Response::SlashCommand(SlashCommand::SubCommand(sub_cmd)), choice]
                if sub_cmd == "info" =>
            {
                Ok(SettingsSubCommands::Info(choice.translate_to::<About>()?))
            }
            [Response::SlashCommand(SlashCommand::SubCommand(sub_cmd)), option, users @ ..]
                if sub_cmd == "members" =>
            {
                Ok(SettingsSubCommands::Members(
                    option.translate_to::<Options>()?,
                    users.translate_to::<Vec<User>>()?,
                ))
            }
            [Response::SlashCommand(SlashCommand::SubCommand(sub_cmd)), lower, upper]
                if sub_cmd == "range" =>
            {
                Ok(SettingsSubCommands::Range(
                    lower.translate_to::<i64>()?,
                    upper.translate_to::<i64>()?,
                ))
            }
            [Response::SlashCommand(SlashCommand::SubCommand(sub_cmd)), option, choice, arg]
                if sub_cmd == "exclude" =>
            {
                Ok(SettingsSubCommands::Exclude(
                    option.translate_to::<Options>()?,
                    choice.translate_to::<Choices>()?,
                    arg.translate_to::<String>()?,
                ))
            }
            [Response::SlashCommand(SlashCommand::SubCommand(sub_cmd)), option, choice, arg]
                if sub_cmd == "target" =>
            {
                Ok(SettingsSubCommands::Target(
                    option.translate_to::<Options>()?,
                    choice.translate_to::<Choices>()?,
                    arg.translate_to::<String>()?,
                ))
            }
            [Response::SlashCommand(SlashCommand::SubCommand(sub_cmd)), choice]
                if sub_cmd == "obliterate" =>
            {
                Ok(SettingsSubCommands::Obliterate(
                    choice.translate_to::<Choices>()?,
                ))
            }
            // start without sub-command
            _ => Err(anyhow::anyhow!("error: cannot transmute {:?}", &self)),
        }
    }
}

impl TranslateTo<StatisticsSubCommands> for &[(String, Response)] {
    fn translate_to<T>(&self) -> anyhow::Result<StatisticsSubCommands>
    where
        T: SameAs<StatisticsSubCommands>,
    {
        match self {
            [(_, Response::SlashCommand(SlashCommand::SubCommand(sub_cmd)))]
                if sub_cmd == "help" =>
            {
                Ok(StatisticsSubCommands::Help)
            }
            [(_, Response::SlashCommand(SlashCommand::SubCommand(sub_cmd))), queryable @ ..]
                if sub_cmd == "query" =>
            {
                let from = queryable
                    .iter()
                    .filter_map(|(_, item)| item.translate_to::<User>().ok())
                    .next()
                    .with_context(|| anyhow::anyhow!("no user found."))?;

                let queries = queryable
                    .iter()
                    .filter_map(|(key, item)| {
                        item.translate_to::<String>()
                            .ok()
                            .map(|query| (key.clone(), query))
                    })
                    .collect::<HashMap<_, _>>();

                Ok(StatisticsSubCommands::Query {
                    from,
                    weapon: queries.get("weapon").cloned(),
                    since: queries.get("since").cloned(),
                    until: queries.get("until").cloned(),
                })
            }
            // start without sub-command
            _ => Err(anyhow::anyhow!("error: cannot transmute {:?}", &self)),
        }
    }
}
