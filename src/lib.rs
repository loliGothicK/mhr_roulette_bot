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

#![allow(incomplete_features)] // Yeah!

// fn foo() -> Result<(), !> { Ok(()) }
//                       ~~~ never type
#![feature(never_type)]
// std::async::Stream
#![feature(async_stream)]
// <const N: {Integer}>: where Foo<{N + 1}>
//  ~~~~~~~~~~~~~~~~~~             ~~~~~~~
//  const generics                 const evaluatable checked
#![feature(const_evaluatable_checked)]
#![feature(const_generics)]
// let foo = ...;
// println!("{foo}");
//          ~~~~~~~ format args capture (seem to be C#)
#![feature(format_args_capture)]
// Stable:  move || async move { ... }
// Nightly: async move || { ... }
#![feature(async_closure)]
#![feature(backtrace)]
#![feature(type_name_of_val)]
#![feature(fn_traits)]
#![feature(box_syntax)]

pub mod build_info;
pub mod concepts;
pub mod data;
pub mod error;
pub mod executors;
pub mod github;
pub mod global;
pub mod model;
pub mod parser;
pub mod bot;
