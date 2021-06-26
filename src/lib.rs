#![allow(incomplete_features)]
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

pub mod build_info;
pub mod concepts;
pub mod data;
pub mod executors;
pub mod global;
pub mod model;
pub mod parser;
pub mod stream;
