//! This is full-featured modern JSON implementation according to ECMA-404 standard.
//!
//! This crate allows deserialization of JSON `Iterator<u8>` stream into primitive types (`bool`, `i32`, etc.),
//! Strings and any other types that implement special trait called [TryFromJson](trait.TryFromJson.html), which can be implemented
//! automatically through `#[derive(TryFromJson)]` for your structs and enums.
//!
//! And serialization back to JSON through [DebugToJson](trait.DebugToJson.html) trait, that acts like [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html), allowing to
//! print your objects with `println!()` and such. Or through [WriteToJson](trait.WriteToJson.html) trait that allows to write
//! to a `io::Write` stream.
//!
//! This crate allows to read whitespece-separated JSON values from stream in sequence. It also allows to pipe blob strings to a writer.
//!
//! # Installation
//!
//! In `Cargo.toml` of your project add:
//!
//! ```toml
//! [dependencies]
//! nop-json = "2.0"
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
//! use std::convert::TryInto;
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
//! You can println!() word "true" or "false" to serialize a boolean. Also numbers can be printed as println!() does by default.
//! The format is JSON-compatible. To serialize a &str, you can use [escape](fn.escape.html) function.
//!
//! Alternatively you can create a [Value](enum.Value.html) object, and serialize with it any scalar/nonscalar value.
//! ```
//! use std::convert::TryInto;
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

extern crate numtoa;
extern crate nop_json_derive;

mod nop_json;
mod value;
mod debug_to_json;
mod write_to_json;

pub use crate::nop_json::{Reader, TryFromJson, ValidateJson, escape, escape_bytes};
pub use crate::debug_to_json::DebugToJson;
pub use crate::write_to_json::WriteToJson;
pub use value::Value;
