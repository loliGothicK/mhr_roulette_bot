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

use crate::{
    build_info::*,
    model::request::{Message, Request},
};
use serenity::{builder::CreateEmbed, utils::Colour};

pub fn version() -> anyhow::Result<Request, !> {
    let mut embed = CreateEmbed::default();
    embed
        .colour(Colour::DARK_BLUE)
        .title(format!("{PKG_NAME} v{PKG_VERSION}"))
        .field(
            "Supported Monster Hunter Rise Version: ",
            "Version 3.1.0 (2021-06-26)",
            false,
        )
        .field("RUSTC_VERSION: ", RUSTC_VERSION, false)
        .field("TARGET: ", TARGET, false)
        .field("OPT_LEVEL: ", OPT_LEVEL, false);
    Ok(Request::Message(Message::Embed(embed)))
}
