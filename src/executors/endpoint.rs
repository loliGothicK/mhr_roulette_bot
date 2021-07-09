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
    error::LogicError,
    executors::*,
    model::{
        request::Request,
        response::{Commands, Response},
        translate::TranslateTo,
    },
};
use itertools::Itertools;
use roulette_macros::{bailout, pretty_info};

#[tracing::instrument]
pub fn interaction_endpoint(items: &[(String, Response)]) -> anyhow::Result<Request> {
    tracing::debug!(got = ?items);
    match items {
        [first, options @ ..] => {
            if let Ok(command) = first.1.translate_to::<Commands>() {
                let option_values = options.iter().map(|(_, v)| v).cloned().collect_vec();
                match command {
                    Commands::Settings => settings(&option_values),
                    Commands::Generate => generate(&option_values),
                    Commands::Statistics => statistics(options),
                    Commands::Version => Ok(version().unwrap()),
                }
            } else {
                let expr = stringify!(first);
                let typename = std::any::type_name_of_val(first);
                bailout!(
                    "FATAL ERROR: Got unknown slash commands or component interactions",
                    LogicError::UnreachableGuard {
                        expr: format!("{expr}: {typename}"),
                        value: format!("{first:?}"),
                        info: pretty_info!(),
                    }
                )
            }
        }
        [] => bailout!(
            "No interaction",
            LogicError::UnreachableGuard {
                expr: "[]".to_owned(),
                value: "[]".to_owned(),
                info: pretty_info!()
            }
        ),
    }
}
