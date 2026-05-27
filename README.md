# nop-json

[![Documentation](https://docs.rs/nop-json/badge.svg)](https://docs.rs/nop-json)
[![crates.io](https://img.shields.io/crates/v/nop-json.svg)](https://crates.io/crates/nop-json)

`nop-json` is a streaming JSON reader and writer for Rust. It deserializes a byte stream directly
into your own types and serializes them back, without building an intermediate document — unless
you ask for one.

## Features

- **Streaming, low-allocation parsing.** A `Reader` consumes any `Iterator<Item=u8>` — a `&str`, a
  `&[u8]`, a file, a socket — and reads one value at a time.
- **Reads straight into Rust types.** Any type implementing `TryFromJson` can be read with
  `reader.read()`. It's provided for primitives, `String`, `char`, `Option`, `Box`, `Rc`, `Arc`,
  `Vec`, `HashMap`, `BTreeMap`, sets, tuples and more, and can be derived for your own structs and
  enums with `#[derive(TryFromJson)]`.
- **Serializes back to JSON** with `#[derive(DebugToJson)]` (which also gives you a JSON `Debug`
  impl, so `println!("{:?}", x)` and `x.to_json_string()` produce JSON) or `#[derive(WriteToJson)]`
  (which writes to any `io::Write`).
- **Sequences:** one `Reader` reads many whitespace-separated values from a single stream.
- **Binary blobs:** smuggle arbitrary bytes (`0x00`–`0xFF`) through JSON strings and read them back,
  or stream them to a writer with `pipe_blob`.
- **Safe on untrusted input:** configurable nesting-depth and value-size limits (see below).

## Installation

```toml
[dependencies]
nop-json = "2.0"
```

## Quick start

```rust
use nop_json::{Reader, TryFromJson, ValidateJson, DebugToJson};

#[derive(TryFromJson, ValidateJson, DebugToJson, PartialEq)]
struct Point {x: i32, y: i32}

let mut reader = Reader::new(r#" {"x": 1, "y": 2} "#.bytes());
let point: Point = reader.read().unwrap();
assert_eq!(point, Point {x: 1, y: 2});

// ...and serialize it back
assert_eq!(point.to_json_string(), r#"{"x":1,"y":2}"#);
```

## The JSON dialect

`nop-json` accepts the JSON grammar of ECMA-404, plus a few conveniences inspired by JavaScript's
own value conversions:

- **Lenient coercion** on read: a JSON string holding a number (`"123"`) can be read into a numeric
  type, a number can be read into a `String`, and `true`/`false`/`null` read into a number give
  `1`/`0`/`0`.
- **Non-finite floats** travel as JSON *strings*: `f32`/`f64` infinities and NaN serialize as
  `"Infinity"`, `"-Infinity"` and `"NaN"`, and those strings read back to the matching values.
- It is **not** JSON5: comments, single-quoted strings, unquoted keys, hexadecimal numbers and bare
  `Infinity`/`NaN` are not accepted.

## Reading untrusted input

A `Reader` enforces two limits to stay safe against hostile input. Override them with
`ReaderBuilder`:

```rust
use nop_json::ReaderBuilder;

let mut reader = ReaderBuilder::new()
    .depth_limit(64)            // max array/object nesting (default 256)
    .value_size_limit(1 << 20)  // max bytes of one string or blob (default 1 GiB)
    .build(r#" [1, 2, 3] "#.bytes());
```

`depth_limit` bounds parser recursion, so input nested deeper than the limit returns an error
instead of overflowing the stack. `value_size_limit` caps the size of a single in-memory string or
blob.

## Documentation

Full API reference and more examples: [docs.rs/nop-json](https://docs.rs/nop-json).
