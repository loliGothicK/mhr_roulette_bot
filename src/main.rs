#![feature(format_args_capture)]
#![feature(backtrace)]

use mhr_roulette::{
    global, github,
    stream::{build_client, Msg},
};
use std::error::Error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    github::Client::init(|builder| {
        builder
            .add_preview("mhr_roulette_bot")
            .base_url(std::env::var("GITHUB_PERSONAL_TOKEN").unwrap()).unwrap()
            .personal_token(std::env::var("GITHUB_PERSONAL_TOKEN").unwrap())
    });

    let tx = global::SRX.sender();

    tokio::spawn(async move {
        let mut client = build_client().await.expect("client");
        if let Err(why) = client.start().await {
            let _ = tx
                .send(Msg::Issue {
                    cause: format!("Client error: {why}"),
                    backtrace: format!("Client error: {:?}", why.backtrace()),
                })
                .await;
        }
    });

    loop {
        if let Ok(ref mut guardian) = global::SRX.receiver().try_lock() {
            let rx = &mut *guardian;
            while let Some(msg) = rx.recv().await {
                match msg {
                    Msg::Issue { cause, backtrace } => {
                        log::error!("{}", format!("{cause} [{backtrace}]"));
                    }
                    Msg::Info { title, description } => {
                        log::info!(
                            "{}",
                            format!(
                                "INFO: {{ {title} => {} }}",
                                description.unwrap_or_else(|| "No description".to_owned())
                            )
                        );
                    }
                    Msg::Debug { title, description } => {
                        log::debug!(
                            "{}",
                            format!(
                                "DEBUG: {{ {title} => {} }}",
                                description.unwrap_or_else(|| "No description".to_owned())
                            )
                        );
                    }
                    Msg::Event { title, description } => {
                        log::info!(
                            "{}",
                            format!(
                                "{{ Event: {title} => {} }}",
                                description.unwrap_or_else(|| "No description".to_owned())
                            )
                        );
                    }
                }
            }
        }
    }
}
