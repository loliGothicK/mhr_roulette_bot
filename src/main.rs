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

#![feature(format_args_capture)]
#![feature(backtrace)]

use http::header;
use mhr_roulette::{
    error::TriageTag,
    github,
    github::CreateIssue,
    global,
    stream::{prepare_bot_client, Msg},
};
use octocrab::OctocrabBuilder;
use std::error::Error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // prepare tracing subscriber
    let file_appender = tracing_appender::rolling::hourly(std::env::var("LOG_OUTPUT_PATH").unwrap(), "roulette.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).with_max_level(tracing::Level::DEBUG).init();

    // initialize github client
    github::Client::init(|builder: OctocrabBuilder| -> anyhow::Result<_> {
        Ok(builder
            .add_preview("mhr-roulette-bot")
            .base_url("https://api.github.com")?
            .add_header(header::ACCEPT, "application/vnd.github.v3+json".to_owned())
            .personal_token(std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN")?))
    })?;

    // spawn bot client
    tokio::spawn(async move {
        let mut client = prepare_bot_client().await.expect("client");
        if let Err(why) = client.start().await {
            let _ = global::SRX
                .sender()
                .send(Msg::Issue {
                    kind: "client error".into(),
                    tag: TriageTag::NotBad,
                    cause: format!("Client error: {why}"),
                    backtrace: format!("Client error: {:?}", why.backtrace()),
                })
                .await;
        }
    });

    // lock receiver
    if let Ok(ref mut guardian) = global::SRX.receiver().try_lock() {
        let rx = &mut *guardian;
        // streaming
        while let Some(msg) = rx.recv().await {
            match msg {
                // If an issue reported,
                Msg::Issue {
                    kind,
                    tag,
                    cause,
                    backtrace,
                } => {
                    log::error!(
                        "{}",
                        format!("triage({tag:?}): {kind}\n{cause}\n{backtrace}")
                    );
                    use TriageTag::*;
                    match tag {
                        // and triage tag is Immediate or Delayed,
                        Immediate | Delayed => {
                            // then send the issue to github repository
                            let _ = github::IssueBuilder::new()
                                .triage_tag(tag)
                                .title(kind)
                                .body(|body| {
                                    use mhr_roulette::build_info::*;
                                    body.summary(cause)
                                        .env(|info| {
                                            info.target(TARGET)
                                                .host(HOST)
                                                .profile(PROFILE)
                                                .commit(GIT_COMMIT_HASH.unwrap_or("not git"))
                                                .dirty(GIT_DIRTY.unwrap_or(false))
                                                .time(BUILT_TIME_UTC)
                                        })
                                        .backtrace(backtrace)
                                        .build()
                                })
                                .send()
                                .await;
                        }
                        _ => {}
                    }
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
        Ok(())
    } else {
        anyhow::bail!("cannot lock receiver");
    }
}
