use crate::model::response::{Component, Response, SlashCommand};
use serenity::model::interactions::{
    ApplicationCommandInteractionData, ApplicationCommandInteractionDataOption,
    ApplicationCommandOptionType, ComponentType, MessageComponent,
};

type DataOptions = Vec<ApplicationCommandInteractionDataOption>;

pub trait Parser {
    fn parse(&self) -> anyhow::Result<Vec<(String, Response)>>;
}

/// # Parse an Message Component
/// Parse an interaction containing messages.
/// More detail, see [DEVELOPER PORTAL](https://discord.com/developers/docs/interactions/slash-commands#data-models-and-types).
impl Parser for ApplicationCommandInteractionData {
    fn parse(&self) -> anyhow::Result<Vec<(String, Response)>> {
        type ParserImpl<'a> = &'a dyn Fn(
            &Parser,
            &mut Vec<(String, Response)>,
            &DataOptions,
        ) -> anyhow::Result<Vec<(String, Response)>>;

        let mut items = vec![(
            "command".to_string(),
            Response::SlashCommand(SlashCommand::Command(self.name.clone())),
        )];

        struct Parser<'a> {
            parser: ParserImpl<'a>,
        }

        let parser = Parser {
            parser: &|succ, ret, options| {
                if options.is_empty() {
                    Ok(ret.clone())
                } else {
                    type Type = ApplicationCommandOptionType;
                    for option in options {
                        match option.kind {
                            Type::SubCommand => {
                                ret.push((
                                    "sub_command".to_string(),
                                    Response::SlashCommand(SlashCommand::SubCommand(
                                        option.name.clone(),
                                    )),
                                ));
                            }
                            Type::String
                            | Type::Integer
                            | Type::Boolean
                            | Type::User
                            | Type::Channel
                            | Type::Role => {
                                ret.push((
                                    option.name.clone(),
                                    Response::SlashCommand(SlashCommand::Option(Box::new(
                                        option.resolved.as_ref().unwrap().clone(),
                                    ))),
                                ));
                            }
                            x => {
                                anyhow::bail!("invalid option type: {:?}", x);
                            }
                        }
                    }
                    if let Some(last) = options.last() {
                        (succ.parser)(succ, ret, &last.options)
                    } else {
                        Ok(ret.clone())
                    }
                }
            },
        };
        (parser.parser)(&parser, &mut items, &self.options)
    }
}

/// # Parse an Message Component
/// Parse an interaction containing messages.
/// More detail, see [DEVELOPER PORTAL](https://discord.com/developers/docs/interactions/message-components).
impl Parser for MessageComponent {
    fn parse(&self) -> anyhow::Result<Vec<(String, Response)>> {
        match self.component_type {
            ComponentType::Button => Ok(vec![(
                self.custom_id.clone(),
                Response::Component(Component::Button(self.custom_id.clone())),
            )]),
            ComponentType::SelectMenu => Ok(vec![(
                self.custom_id.clone(),
                Response::Component(Component::SelectMenu(self.values.clone())),
            )]),
            _ => anyhow::bail!("{:?}", &self),
        }
    }
}
