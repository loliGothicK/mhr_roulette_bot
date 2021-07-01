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

use octocrab::OctocrabBuilder;
use once_cell::sync::OnceCell;

pub static GITHUB_CLIENT: OnceCell<Client> = OnceCell::new();

#[derive(Debug)]
pub struct Client {
    client: octocrab::Octocrab,
}

impl Client {
    pub fn init<Builder>(builder: Builder) -> anyhow::Result<()>
    where
        Builder: FnOnce(OctocrabBuilder) -> anyhow::Result<OctocrabBuilder>,
    {
        GITHUB_CLIENT
            .set(Client {
                client: builder(OctocrabBuilder::new())?.build()?,
            })
            .map_err(|_| anyhow::anyhow!("Failed to set github client."))
    }

    pub fn global() -> Option<&'static octocrab::Octocrab> {
        GITHUB_CLIENT.get().map(|github| &github.client)
    }
}
