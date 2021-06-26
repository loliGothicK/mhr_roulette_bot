use anyhow::{anyhow, Context};
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    http::Http,
    model::{
        gateway::Ready,
        interactions::{
            ApplicationCommand, ApplicationCommandOptionType, Interaction, InteractionData,
            InteractionResponseType,
        },
    },
};
use std::env;
use std::fmt::Debug;
use tracing::{span, Level};

use crate::concepts::SameAs;
use crate::executors::interaction_endpoint;
use crate::model::request::{Message, Request};
use crate::{global, model::request, parser::Parser};
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;
use crate::global::SRX;

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
        let tx = global::SRX.sender();
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
                            cause: format!("{err}"),
                            backtrace: format!("{}", err.backtrace()),
                        })
                        .await;
                });
            }
        }
    }
}

/// Handler for bot
#[derive(Debug)]
struct Handler;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Msg {
    Issue {
        cause: String,
        backtrace: String,
    },
    Info {
        title: String,
        description: Option<String>,
    },
    Debug {
        title: String,
        description: Option<String>,
    },
    Event {
        title: String,
        description: Option<String>,
    },
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
        let interaction_result = interaction
            .data
            .as_ref()
            .map(|data| match data {
                InteractionData::ApplicationCommand(command) => command.parse(),
                InteractionData::MessageComponent(component) => component.parse(),
            })
            .transpose()
            .and_then(|maybe_items| maybe_items.ok_or_else(|| anyhow!("no interaction data")))
            .and_then(|items| interaction_endpoint(&items));

        match interaction_result {
            Err(err) => {
                let mut embed = CreateEmbed::default();
                embed
                    .colour(Colour::RED)
                    .title("INTERACTION ERROR:")
                    .description(format!("{err}"));

                let json = serde_json::to_string(&embed.0);

                interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.add_embed(embed))
                    })
                    .await
                    .map(|_| format!(r#"{{ "response" => "{json:?}" }}"#))
                    .map_err(|#[allow(unused)] err| anyhow!("http error: {err} with {json:?}"))
                    .send_msg();

                let _ = SRX.sender()
                    .send(Msg::Issue {
                        cause: format!("{err}"),
                        backtrace: format!("{}", err.backtrace()),
                    })
                    .await;
            }
            Ok(response) => match response {
                Request::Message(msg) => match msg {
                    Message::String(msg) => {
                        interaction
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
                        interaction
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
                    interaction
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
                                    request::Component::SelectMenu(options) => data
                                        .content("I'll stabilize select menu when it's documented.")
                                        .components(|components| {
                                            components.create_action_row(|act| {
                                                act.create_select_menu(|select_menu| {
                                                    select_menu
                                                        .placeholder("選択肢がありません")
                                                        .custom_id("select_menu")
                                                        .min_values(1)
                                                        .max_values(1)
                                                        .options(|builder| {
                                                            for _option in options {
                                                                builder.create_option(|opt| {
                                                                    opt.description("a")
                                                                        .label("a")
                                                                        .value("a")
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
                    interaction
                        .delete_original_interaction_response(&ctx.http)
                        .await
                        .map_err(|err| anyhow!("http error: {}", err))
                        .send_msg();
                }
            },
        }
    }
}

pub async fn build_client() -> anyhow::Result<Client> {
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
                    .create_sub_option(|o| {
                        o.name("lower")
                            .description("lower bound")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice(1, 1)
                            .add_string_choice(2, 2)
                            .add_string_choice(3, 3)
                            .add_string_choice(4, 4)
                            .add_string_choice(5, 5)
                            .add_string_choice(6, 6)
                            .add_string_choice(7, 7)
                            .add_string_choice("8（HR解放後）", 8)
                            .required(true)
                    })
                    .create_sub_option(|o| {
                        o.name("upper")
                            .description("upper bound")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice(1, 1)
                            .add_string_choice(2, 2)
                            .add_string_choice(3, 3)
                            .add_string_choice(4, 4)
                            .add_string_choice(5, 5)
                            .add_string_choice(6, 6)
                            .add_string_choice(7, 7)
                            .add_string_choice("8（HR解放後）", 8)
                            .required(true)
                    })
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
