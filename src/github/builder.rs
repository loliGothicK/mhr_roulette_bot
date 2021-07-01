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

use super::Client;
use crate::error::TriageTag;
use anyhow::Context;
use indexmap::map::IndexMap;
use indoc::indoc;
use itertools::Itertools;
use octocrab::models::issues::Issue;
use serenity::async_trait;
use std::fmt::{Display, Formatter};

pub struct Body {
    summary: String,
    env: String,
    backtrace: Option<String>,
}

#[derive(Debug, Default)]
pub struct EnvironmentBuilder {
    info: IndexMap<String, String>,
}

impl EnvironmentBuilder {
    #[allow(dead_code)]
    fn new() -> Self {
        EnvironmentBuilder {
            info: IndexMap::new(),
        }
    }

    pub fn target(self, x: &str) -> Self {
        let mut info = self.info;
        info.entry("target".to_owned()).or_insert(x.into());
        Self { info }
    }

    pub fn host(self, x: &str) -> Self {
        let mut info = self.info;
        info.entry("host".to_owned()).or_insert(x.into());
        Self { info }
    }

    pub fn opt_level(self, x: &str) -> Self {
        let mut info = self.info;
        info.entry("opt_level".to_owned()).or_insert(x.into());
        Self { info }
    }

    pub fn rustc_version(self, x: &str) -> Self {
        let mut info = self.info;
        info.entry("rustc_version".to_owned()).or_insert(x.into());
        Self { info }
    }

    pub fn profile(self, x: &str) -> Self {
        let mut info = self.info;
        info.entry("profile".to_owned()).or_insert(x.into());
        Self { info }
    }

    pub fn time(self, x: &str) -> Self {
        let mut info = self.info;
        info.entry("time".to_owned()).or_insert(x.into());
        Self { info }
    }

    pub fn commit(self, x: &str) -> Self {
        let mut info = self.info;
        info.entry("commit".to_owned()).or_insert(x.into());
        Self { info }
    }

    pub fn dirty(self, x: bool) -> Self {
        let mut info = self.info;
        info.entry("dirty?".to_owned()).or_insert(format!("{x}"));
        Self { info }
    }
}

impl From<EnvironmentBuilder> for String {
    fn from(env: EnvironmentBuilder) -> Self {
        env.info
            .iter()
            .map(|(key, value)| format!("{key} => {value}"))
            .collect_vec()
            .join("\n")
    }
}

pub struct BodyBuilder<Summary, Env> {
    summary: Summary,
    env: Env,
    backtrace: Option<String>,
}

impl BodyBuilder<(), ()> {
    pub fn new() -> Self {
        BodyBuilder {
            summary: (),
            env: (),
            backtrace: None,
        }
    }
}
impl Default for BodyBuilder<(), ()> {
    fn default() -> Self {
        Self::new()
    }
}
impl BodyBuilder<String, String> {
    pub fn build(self) -> Body {
        Body {
            summary: self.summary,
            env: self.env,
            backtrace: self.backtrace,
        }
    }
}
impl<Summary, Env> BodyBuilder<Summary, Env> {
    pub fn summary(self, summary: impl Into<String>) -> BodyBuilder<String, Env> {
        BodyBuilder {
            summary: summary.into(),
            env: self.env,
            backtrace: self.backtrace,
        }
    }

    pub fn env<Builder, Info>(self, env: Builder) -> BodyBuilder<Summary, String>
    where
        Builder: FnOnce(EnvironmentBuilder) -> Info,
        Info: Into<String>,
    {
        BodyBuilder {
            summary: self.summary,
            env: env(EnvironmentBuilder::default()).into(),
            backtrace: self.backtrace,
        }
    }

    pub fn backtrace(self, backtrace: impl Display) -> BodyBuilder<Summary, Env> {
        BodyBuilder {
            summary: self.summary,
            env: self.env,
            backtrace: Some(format!("{backtrace}")),
        }
    }
}

#[async_trait]
pub trait CreateIssue {
    async fn send(&self) -> anyhow::Result<Issue>;
}

pub struct IssueBuilder<Title, Label, Body> {
    title: Title,
    label: Label,
    body: Body,
}

impl IssueBuilder<(), (), ()> {
    pub fn new() -> Self {
        IssueBuilder {
            title: (),
            label: (),
            body: (),
        }
    }
}

impl Default for IssueBuilder<(), (), ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Tag, Cause, Backtrace> IssueBuilder<Tag, Cause, Backtrace> {
    pub fn title(self, title: impl Into<String>) -> IssueBuilder<String, Cause, Backtrace> {
        IssueBuilder {
            title: title.into(),
            label: self.label,
            body: self.body,
        }
    }

    pub fn triage_tag(self, tag: TriageTag) -> IssueBuilder<Tag, String, Backtrace> {
        IssueBuilder {
            title: self.title,
            label: tag.to_string(),
            body: self.body,
        }
    }

    pub fn body<Builder>(self, builder: Builder) -> IssueBuilder<Tag, Cause, Body>
    where
        Builder: FnOnce(BodyBuilder<(), ()>) -> Body,
    {
        IssueBuilder {
            title: self.title,
            label: self.label,
            body: builder(BodyBuilder::new()),
        }
    }
}

#[async_trait]
impl CreateIssue for IssueBuilder<String, String, Body> {
    async fn send(&self) -> anyhow::Result<Issue> {
        let IssueBuilder { title, label, body } = self;
        create_issue(title, label, body).await
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Body {
            summary,
            env,
            backtrace,
        } = self;
        let body = format!(
            indoc! {r#"
                # Summary
                {}

                ## Build Info
                ```
                {}
                ```

                ## Stack Backtrace
                ```
                {:?}
                ```
            "#},
            summary, env, backtrace
        );
        write!(f, "{}", body)
    }
}

async fn create_issue(title: &str, label: &str, body: &Body) -> anyhow::Result<Issue> {
    match Client::global() {
        Some(client) => client
            .issues("LoliGothick", "mhr_roulette_bot")
            .create(format!("triage({label:?}): {title}"))
            .body(format!("{body}"))
            .labels(vec![label.to_string()])
            .send()
            .await
            .with_context(|| anyhow::anyhow!("send error")),
        None => anyhow::bail!("Client is not initialized"),
    }
}
