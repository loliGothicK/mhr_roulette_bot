use crate::build_info::*;
use crate::model::request::{Message, Request};
use serenity::builder::CreateEmbed;
use serenity::utils::Colour;

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
