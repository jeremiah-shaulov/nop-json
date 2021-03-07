# nop-json

[![Documentation](https://docs.rs/nop-json/badge.svg)](https://docs.rs/nop-json)
[![crates.io](https://img.shields.io/crates/v/nop-json.svg)](https://crates.io/crates/nop-json)

This is full-featured modern JSON implementation according to ECMA-404 standard.

This crate allows deserialization of JSON `Iterator<u8>` stream into primitive types (`bool`, `i32`, etc.),
Strings and any other types that implement special trait called `TryFromJson`, which can be implemented
automatically through `#[derive(TryFromJson)]` for your structs and enums.

And serialization back to JSON through `DebugToJson` trait, that acts like [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html), allowing to
print your objects with `println!()` and such. Or through `WriteToJson` trait that allows to write
to a `io::Write` stream.

This crate allows to read whitespece-separated JSON values from stream in sequence. It also allows to pipe blob strings to a writer.

## Documentation

[Read on crates.io](https://docs.rs/nop-json)
