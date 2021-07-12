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

use anyhow::{anyhow, Context};
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    http::Http,
    model::{
        gateway::Ready,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType},
            Interaction, InteractionResponseType,
        },
    },
};
use std::{env, fmt::Debug};
use tracing::{span, Level};

use crate::{
    concepts::SameAs,
    error::{ErrorExt, TriageTag},
    executors::interaction_endpoint,
    global,
    global::CENTRAL,
    model::{
        request,
        request::{Message, Request},
    },
    parser::Parser,
};
use serenity::{
    builder::{CreateEmbed, CreateInteractionResponse},
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
    },
    utils::Colour,
};

pub trait MsgSender<Msg: Debug> {
    fn send_msg(self)
    where
        Self: SameAs<Msg>;
}

impl<T: Debug + Send + Sync + 'static> MsgSender<anyhow::Result<T>> for anyhow::Result<T> {
    fn send_msg(self)
    where
        Self: SameAs<anyhow::Result<T>>,
    {
        let tx = global::CENTRAL.sender();
        match self {
            Ok(msg) => {
                tokio::spawn(async move {
                    let _ = tx
                        .send(Msg::Info {
                            title: "Succeeded in sending a response to the Discord API".to_owned(),
                            description: Some(format!("{msg:?}")),
                        })
                        .await;
                });
            }
            Err(err) => {
                tokio::spawn(async move {
                    let _ = tx
                        .send(Msg::Issue {
                            kind: "http error".into(),
                            tag: TriageTag::NotBad,
                            cause: format!("{err:?}"),
                            backtrace: format!("{}", err.backtrace()),
                        })
                        .await;
                });
            }
        }
    }
}

/// Handler for the BOT
#[derive(Debug)]
struct Handler;

/// Message sections for Sender/Receiver
#[derive(Debug)]
pub enum Msg {
    /// Message that report issues
    Issue {
        kind: String,
        tag: TriageTag,
        cause: String,
        backtrace: String,
    },
    /// Message that useful information
    Info {
        title: String,
        description: Option<String>,
    },
    /// Message that detailed information
    Debug {
        title: String,
        description: Option<String>,
    },
    /// Message for event trigger
    Event {
        title: String,
        description: Option<String>,
    },
}

enum Interactions {
    Command(ApplicationCommandInteraction),
    Component(Box<MessageComponentInteraction>),
}

impl Interactions {
    pub async fn create_interaction_response<F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce(&mut CreateInteractionResponse) -> &mut CreateInteractionResponse,
    {
        match self {
            Interactions::Command(command) => command.create_interaction_response(http, f).await?,
            Interactions::Component(component) => {
                (*component).create_interaction_response(http, f).await?
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity::client::Context, ready: Ready) {
        let span = span!(Level::TRACE, "establish connection", ?ready);
        let _enter = span.enter();

        tracing::event!(Level::INFO, "{} is connected!", ready.user.name);
        let interactions = ApplicationCommand::get_global_application_commands(&ctx.http)
            .await
            .map_err(|err| anyhow!("http error: {:?}", err))
            .and_then(|commands| {
                serde_json::to_string(&commands).with_context(|| anyhow!("failed to serialize"))
            });
        tracing::info!("I have the following global slash command(s): {interactions:?}",);
    }

    async fn interaction_create(&self, ctx: serenity::client::Context, interaction: Interaction) {
        let result = {
            if let Some(command) = interaction.clone().application_command() {
                Some(
                    command
                        .data
                        .parse()
                        .and_then(|items| interaction_endpoint(&items))
                        .map(|ok| (ok, Interactions::Command(command.clone())))
                        .map_err(|err| (err, Interactions::Command(command.clone()))),
                )
            } else if let Some(component) = interaction.clone().message_component() {
                Some(
                    component
                        .data
                        .parse()
                        .and_then(|items| interaction_endpoint(&items))
                        .map(|ok| (ok, Interactions::Component(Box::new(component.clone()))))
                        .map_err(|err| (err, Interactions::Component(Box::new(component.clone())))),
                )
            } else {
                None
            }
        };
        // un-expected interaction => skip
        let result = if let Some(res) = result {
            res
        } else {
            return;
        };
        match result {
            Err((err, interactions)) => {
                let mut embed = CreateEmbed::default();
                embed
                    .colour(Colour::RED)
                    .title("INTERACTION ERROR:")
                    .description(format!("{err:?}"));

                let json = serde_json::to_string(&embed.0);

                interactions
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.add_embed(embed))
                    })
                    .await
                    .map(|_| format!(r#"{{ "response" => "{json:?}" }}"#))
                    .map_err(|#[allow(unused)] err| anyhow!("http error: {err} with {json:?}"))
                    .send_msg();

                let _ = CENTRAL
                    .sender()
                    .send(Msg::Issue {
                        kind: err.kind().to_string(),
                        tag: err.triage().unwrap_or(TriageTag::NotBad),
                        cause: format!("{err:?}"),
                        backtrace: format!("{}", err.backtrace()),
                    })
                    .await;
            }
            Ok((response, interactions)) => match response {
                Request::Message(msg) => match msg {
                    Message::String(msg) => {
                        interactions
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| message.content(&msg))
                            })
                            .await
                            .map(|_| format!(r#"{{ "response" => "{msg}" }}"#))
                            .map_err(|#[allow(unused)] err| anyhow!("http error: {err} with {msg}"))
                            .send_msg();
                    }
                    Message::Embed(embed) => {
                        let json = serde_json::to_string(&embed.0);
                        interactions
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| message.add_embed(embed))
                            })
                            .await
                            .map(|_| format!(r#"{{ "response" => {json:?}"#))
                            .map_err(|err| anyhow!("http error: {} with {:?}", err, json))
                            .send_msg();
                    }
                },
                Request::Components(component) => {
                    interactions
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|data| match component {
                                    request::Component::Buttons(buttons) => {
                                        data.content("Hello Button.").components(|components| {
                                            components.create_action_row(|action_row| {
                                                for button in buttons.into_iter() {
                                                    action_row.add_button(button);
                                                }
                                                action_row
                                            })
                                        })
                                    }
                                    request::Component::SelectMenu {
                                        custom_id,
                                        min_value,
                                        max_value,
                                        options,
                                    } => data
                                        .content("I'll stabilize select menu when it's documented.")
                                        .components(|components| {
                                            components.create_action_row(|act| {
                                                act.create_select_menu(|select_menu| {
                                                    select_menu
                                                        .placeholder("選択肢がありません")
                                                        .custom_id(custom_id)
                                                        .min_values(min_value)
                                                        .max_values(max_value)
                                                        .options(|builder| {
                                                            for opt in options {
                                                                builder.create_option(|o| {
                                                                    o.description(opt.description)
                                                                        .value(opt.value)
                                                                        .label(opt.label)
                                                                });
                                                            }
                                                            builder
                                                        })
                                                })
                                            })
                                        }),
                                })
                        })
                        .await
                        .map_err(|err| anyhow!("http error: {}", err))
                        .send_msg();
                }
                Request::Update { .. } => {
                    // TODO
                }
            },
        }
    }
}

/// # Prepare BOT Client
/// 1. Read the configure toml file.
/// 2. Read the discord token from environment variable `DISCORD_TOKEN`.
/// 3. Read the Bot's application ID from environment variable `APPLICATION_ID`.
/// 4. Establish HTTP connection.
/// 5. Post requests for slash commands to Discord.
/// 6. Finally, build the bot client.
pub async fn prepare_bot_client() -> anyhow::Result<Client> {
    println!(
        "------config.toml-------\n{}------------------------",
        toml::to_string_pretty(&*crate::global::CONFIG.lock().unwrap())?
    );
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()?;

    let http = Http::new_with_token_application_id(&token, application_id);

    // slash commands

    // # settings command
    //
    // ## sub-commands
    // - cat
    // - members
    //     - set [member(s)]
    //     - add [member]
    //     - remove [member]
    // - range [lower] [upper]
    // - exclude
    //     - set [type] [item(s)]
    //     - add [type] [item]
    //     - remove [type] [item]
    // - target
    //     - set [type] [item(s)]
    //     - add [type] [item]
    //     - remove [type] [item]
    // - obliterate
    //     - quest
    //     - monster
    //     - weapon
    //
    let _ = ApplicationCommand::create_global_application_command(&http, |a| {
        a.name("settings")
            .description("Settings[Members/Quest/Weapon]")
            .create_option(|o| {
                o.name("help")
                    .description("Prints help information")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|o| {
                o.name("info")
                    .description("Shows current configurations or data")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|o| {
                        o.name("about")
                            .description("choices")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("quest", "quest")
                            .add_string_choice("monster", "monster")
                            .add_string_choice("weapon", "weapon")
                            .add_string_choice("members", "members")
                            .required(true)
                    })
            })
            .create_option(|o| {
                o.name("members")
                    .description("Specify member candidates")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|o| {
                        o.name("option")
                            .description("set/add/remove")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("set", "set")
                            .add_string_choice("add", "add")
                            .add_string_choice("remove", "remove")
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("user-1")
                            .description("member")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("user-2")
                            .description("member")
                            .kind(ApplicationCommandOptionType::User)
                    })
                    .create_sub_option(|o| {
                        o.name("user-3")
                            .description("member")
                            .kind(ApplicationCommandOptionType::User)
                    })
                    .create_sub_option(|o| {
                        o.name("user-4")
                            .description("member")
                            .kind(ApplicationCommandOptionType::User)
                    })
            })
            .create_option(|o| {
                o.name("range")
                    .description("Specify range of quest rank")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|o| {
                o.name("exclude")
                    .description("Specify quests, monsters, or weapons to exclude")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|o| {
                        o.name("option")
                            .description("set/add/remove")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("set", "set")
                            .add_string_choice("add", "add")
                            .add_string_choice("remove", "remove")
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("type")
                            .description("quest/monster/weapon")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("quest", "quest")
                            .add_string_choice("monster", "monster")
                            .add_string_choice("weapon", "weapon")
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("value")
                            .description("item(s)")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
            })
            .create_option(|o| {
                o.name("target")
                    .description("Configure target candidates (quests and monsters)")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|o| {
                        o.name("option")
                            .description("set/add/remove")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("set", "set")
                            .add_string_choice("add", "add")
                            .add_string_choice("remove", "remove")
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("type")
                            .description("quest/monster/weapon")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("quest", "quest")
                            .add_string_choice("monster", "monster")
                            .add_string_choice("weapon", "weapon")
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("value")
                            .description("item(s)")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
            })
            .create_option(|o| {
                o.name("obliterate")
                    .description("Obliterate target candidates (quests, monsters, or weapons)")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|o| {
                        o.name("type")
                            .description("quest/monster/weapon")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("quest", "quest")
                            .add_string_choice("monster", "monster")
                            .add_string_choice("weapon", "weapon")
                            .required(true)
                    })
            })
    })
    .await?;

    // # generate command
    //
    // ## sub-commands
    // - quest
    // - monster
    //
    let _ = ApplicationCommand::create_global_application_command(&http, |a| {
        a.name("generate")
            .description("generates a monster or quest members and weapons")
            .create_option(|o| {
                o.name("type")
                    .description("quest/monster")
                    .kind(ApplicationCommandOptionType::String)
                    .add_string_choice("quest", "quest")
                    .add_string_choice("monster", "monster")
                    .required(true)
            })
    })
    .await?;

    // # statistics command
    //
    // ## sub-commands
    //  - help
    //  - query
    let _ = ApplicationCommand::create_global_application_command(&http, |a| {
        a.name("statistics")
            .description("statistics query")
            .create_option(|o| {
                o.name("help")
                    .description("Prints help information")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|o| {
                o.name("query")
                    .description("Querying the statistics database")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|o| {
                        o.name("from")
                            .description("Choice a user")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("weapon")
                            .description("specify weapon key")
                            .kind(ApplicationCommandOptionType::String)
                    })
                    .create_sub_option(|o| {
                        o.name("since")
                            .description("YYYY-MM-DD")
                            .kind(ApplicationCommandOptionType::String)
                    })
                    .create_sub_option(|o| {
                        o.name("until")
                            .description("YYYY-MM-DD")
                            .kind(ApplicationCommandOptionType::String)
                    })
            })
    })
    .await?;

    // Test for Unstable Discord APIs
    let _ = ApplicationCommand::create_global_application_command(&http, |a| {
        a.name("version").description("version info")
    })
    .await?;

    log::info!("Now, our client listening on.");

    // Build our client.
    Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .with_context(|| anyhow!("ERROR: failed to build client"))
}
