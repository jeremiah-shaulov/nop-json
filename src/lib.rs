//! `nop-json` is a streaming JSON reader and writer. It deserializes a byte stream directly into your
//! Rust types and serializes them back, without building an intermediate document — unless you ask for
//! one with [Value](enum.Value.html).
//!
//! Deserialization reads from any `Iterator<Item=u8>` into any type that implements
//! [TryFromJson](trait.TryFromJson.html): primitive types (`bool`, `i32`, ...), `String`, `char`,
//! containers like `Vec` and `HashMap`, and your own structs and enums via `#[derive(TryFromJson)]`.
//!
//! Serialization goes back to JSON through [DebugToJson](trait.DebugToJson.html), which doubles as a
//! [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html) implementation (so `println!("{:?}", x)`
//! and `x.to_json_string()` produce JSON), or through [WriteToJson](trait.WriteToJson.html), which
//! writes to any `io::Write`.
//!
//! A single [Reader](struct.Reader.html) reads a sequence of whitespace-separated values from one
//! stream, and can pipe binary blobs straight to a writer.
//!
//! The accepted grammar is that of JSON (ECMA-404), with a few JavaScript-inspired conveniences
//! described under [The JSON dialect](#the-json-dialect). When parsing untrusted input, see
//! [Reading untrusted input](#reading-untrusted-input).
//!
//! # Installation
//!
//! In `Cargo.toml` of your project add:
//!
//! ```toml
//! [dependencies]
//! nop-json = "2.1"
//! ```
//!
//! # Examples
//!
//! ## Creating Reader object
//!
//! First need to create a [Reader](struct.Reader.html) object giving it something that implements `Iterator<Item=u8>`.
//! We can read from a string like this:
//!
//! ```
//! use nop_json::Reader;
//!
//! let mut reader = Reader::new(r#" "a JSON string" "#.bytes());
//! ```
//!
//! To read from a file we need to convert `std::io::Read` to `Iterator<Item=u8>`. We can use `read_iter` crate for this.
//!
//! ```no_run
//! use std::fs::File;
//! use read_iter::ReadIter; // also add dependency to Cargo.toml
//! use nop_json::Reader;
//!
//! let mut file = ReadIter::new(File::open("/tmp/test.json").unwrap());
//! let mut reader = Reader::new(&mut file);
//! ```
//!
//! See [Reader::new()](struct.Reader.html#method.new) for more details.
//!
//! ## Deserializing simple values
//!
//! To read JSON values from an input stream, call `reader.read()` method, and assign the result to a variable that implements `TryFromJson`.
//! This crate adds implementation of `TryFromJson` to many primitive types, `Vec`, `HashMap`, and more.
//!
//! ```
//! use nop_json::Reader;
//!
//! let mut reader = Reader::new(r#" true  100.5  "Hello"  "Infinity"  [true, false] "#.bytes());
//!
//! let the_true: bool = reader.read().unwrap();
//! let the_hundred_point_five: f32 = reader.read().unwrap();
//! let the_hello: String = reader.read().unwrap();
//! let the_infinity: f32 = reader.read().unwrap();
//! let the_array: Vec<bool> = reader.read().unwrap();
//!
//! assert_eq!(the_true, true);
//! assert_eq!(the_hundred_point_five, 100.5);
//! assert_eq!(the_hello, "Hello");
//! assert!(the_infinity.is_infinite());
//! assert_eq!(the_array, vec![true, false]);
//! ```
//!
//! ## Deserializing any JSON values
//!
//! We have generic [Value](enum.Value.html) type that can hold any JSON node.
//!
//! ```
//! use nop_json::{Reader, Value};
//!
//! let mut reader = Reader::new(r#" true  100.5  "Hello"  [true, false] "#.bytes());
//!
//! let the_true: Value = reader.read().unwrap();
//! let the_hundred_point_five: Value = reader.read().unwrap();
//! let the_hello: Value = reader.read().unwrap();
//! let the_array: Value = reader.read().unwrap();
//!
//! assert_eq!(the_true, Value::Bool(true));
//! let the_hundred_point_five: f32 = the_hundred_point_five.try_into().unwrap();
//! assert_eq!(the_hundred_point_five, 100.5f32);
//! assert_eq!(the_hello, Value::String("Hello".to_string()));
//! assert_eq!(the_array, Value::Array(vec![Value::Bool(true), Value::Bool(false)]));
//! ```
//!
//! You can parse any JSON document to [Value](enum.Value.html).
//!
//! ```
//! use nop_json::{Reader, Value};
//!
//! let mut reader = Reader::new(r#" {"array": [{"x": 1}, "a string"]} "#.bytes());
//! let doc: Value = reader.read().unwrap();
//! assert_eq!(doc.to_string(), r#"{"array":[{"x":1},"a string"]}"#);
//! ```
//!
//! ## Deserializing/serializing structs and enums
//!
//! To deserialize a struct or an enum, your struct needs to implement [TryFromJson](trait.TryFromJson.html) and [ValidateJson](trait.ValidateJson.html) traits.
//! To serialize - [DebugToJson](trait.DebugToJson.html) and/or [WriteToJson](trait.WriteToJson.html).
//!
//! ```
//! use nop_json::{Reader, TryFromJson, ValidateJson, DebugToJson};
//!
//! #[derive(TryFromJson, ValidateJson, DebugToJson, PartialEq)]
//! struct Point {x: i32, y: i32}
//!
//! #[derive(TryFromJson, ValidateJson, DebugToJson, PartialEq)]
//! enum Geometry
//! {	#[json(point)] Point(Point),
//! 	#[json(cx, cy, r)] Circle(i32, i32, i32),
//! 	Nothing,
//! }
//!
//! let mut reader = Reader::new(r#" {"point": {"x": 0, "y": 0}} "#.bytes());
//! let obj: Geometry = reader.read().unwrap();
//! println!("Serialized back to JSON: {:?}", obj);
//! ```
//! See [TryFromJson](trait.TryFromJson.html), [ValidateJson](trait.ValidateJson.html), [DebugToJson](trait.DebugToJson.html), [WriteToJson](trait.WriteToJson.html).
//!
//! ## Serializing scalar values
//!
//! Booleans and numbers already format to valid JSON with their normal `Display` (`format!("{}", true)`
//! is `"true"`). To serialize a `&str` as a JSON string, wrap it with the [escape](fn.escape.html)
//! function. Alternatively, build a [Value](enum.Value.html) and serialize that.
//!
//! Alternatively you can create a [Value](enum.Value.html) object, and serialize with it any scalar/nonscalar value.
//! ```
//! use nop_json::Value;
//!
//! let the_true: Value = true.try_into().unwrap();
//! println!("Serialized to JSON: {:?}", the_true);
//! # assert_eq!(format!("{:?}", the_true), "true")
//! ```
//!
//! ## Skipping a value from stream
//!
//! To skip current value without storing it (and allocating memory), read it to the `()` type.
//! ```
//! use nop_json::Reader;
//!
//! let mut reader = Reader::new(r#" true  100.5  "Hello"  [true, false] "#.bytes());
//!
//! let _: () = reader.read().unwrap();
//! let _: () = reader.read().unwrap();
//! let _: () = reader.read().unwrap();
//! let _: () = reader.read().unwrap();
//! ```
//!
//! ## Reading binary data
//! See [read_blob](struct.Reader.html#method.read_blob).
//!
//! ## Null, NaN, infinity and -0
//!
//! Reading to a variable of type `Option<T>` can read either `T` or `null`.
//!
//! ```
//! use nop_json::Reader;
//!
//! let mut reader = Reader::new(r#" "non-null"  null "#.bytes());
//!
//! let str_or_null_1: Option<String> = reader.read().unwrap();
//! let str_or_null_2: Option<String> = reader.read().unwrap();
//!
//! assert_eq!(str_or_null_1, Some("non-null".to_string()));
//! assert_eq!(str_or_null_2, None);
//! ```
//!
//! Reading junk to `f32` or `f64` type will read NaN. Reading string "Infinity", "-Infinity" and "-0" will read corresponding floating point numbers.
//!
//! ```
//! use nop_json::Reader;
//!
//! let mut reader = Reader::new(r#" "Hello all!"  "Infinity"  "-Infinity"  "0"  "-0" "#.bytes());
//!
//! let nan: f32 = reader.read().unwrap();
//! let inf: f32 = reader.read().unwrap();
//! let minf: f32 = reader.read().unwrap();
//! let zero: f32 = reader.read().unwrap();
//! let mzero: f32 = reader.read().unwrap();
//!
//! assert!(nan.is_nan());
//! assert_eq!(inf, f32::INFINITY);
//! assert_eq!(minf, f32::NEG_INFINITY);
//! assert!(zero==0.0 && !zero.is_sign_negative());
//! assert!(mzero==0.0 && mzero.is_sign_negative());
//! ```
//!
//! # The JSON dialect
//!
//! `nop-json` reads and writes the JSON grammar of ECMA-404, with a few JavaScript-inspired
//! conveniences:
//!
//! - On read, values are **coerced** toward the requested type: a quoted number (`"123"`) can be read
//!   into a numeric type, a number can be read into a `String`, and reading `true`/`false`/`null` into
//!   a number gives `1`/`0`/`0`.
//! - Non-finite floats travel as JSON **strings**: `f32`/`f64` infinities and NaN serialize as
//!   `"Infinity"`, `"-Infinity"` and `"NaN"`, and reading those strings (or `"-0"`) yields the matching
//!   value, as shown in the section above.
//! - This is **not** JSON5 — comments, single-quoted strings, unquoted keys, hexadecimal numbers and
//!   bare `Infinity`/`NaN` are not accepted.
//!
//! # Reading untrusted input
//!
//! A [Reader](struct.Reader.html) enforces two limits so that hostile input cannot exhaust the stack
//! or memory. To use non-default limits, build the reader with [ReaderBuilder](struct.ReaderBuilder.html):
//!
//! ```
//! use nop_json::ReaderBuilder;
//!
//! let mut reader = ReaderBuilder::new()
//!     .depth_limit(64)            // max array/object nesting; default 256
//!     .value_size_limit(1 << 20)  // max bytes of one string or blob; default 1 GiB
//!     .build(r#" [1, 2, 3] "#.bytes());
//! let arr: Vec<i32> = reader.read().unwrap();
//! assert_eq!(arr, vec![1, 2, 3]);
//! ```
//!
//! `depth_limit` bounds parser recursion, so input nested deeper than the limit returns an error
//! instead of overflowing the stack. `value_size_limit` caps the size of a single in-memory string or
//! blob (it does not limit [pipe_blob](struct.Reader.html#method.pipe_blob), which streams).

mod nop_json;
mod value;
mod debug_to_json;
mod write_to_json;
mod validate_json;
mod escape;

pub use crate::nop_json::{Reader, ReaderBuilder, TryFromJson};
pub use crate::debug_to_json::DebugToJson;
pub use crate::write_to_json::WriteToJson;
pub use crate::validate_json::ValidateJson;
pub use crate::escape::{escape, escape_bytes};
pub use value::Value;
