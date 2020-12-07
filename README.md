# nop-json

[![Documentation](https://docs.rs/nop-json/badge.svg)](https://docs.rs/nop-json)
[![crates.io](https://img.shields.io/crates/v/nop-json.svg)](https://crates.io/crates/nop-json)

This is full-featured modern JSON implementation according to ECMA-404 standard.

This crate allows deserialization of JSON `io::Read` stream into primitive types (`bool`, `i32`, etc.),
Strings and any other types that implement special trait called `TryFromJson`, which can be implemented
automatically through `#[derive(TryFromJson)]` for your structs and enums.

And serialization back to JSON through `DebugToJson` trait, that acts like `Debug`, allowing to
print your objects with `println!()` and such.

It allows to read whitespece-separated JSON values from stream in sequence. It also allows to pipe blob strings to a writer.

This implementation avoids unnecessary memory allocations and temporary object creations.

## Installation

In `Cargo.toml` of your project add:
```toml
[dependencies]
nop-json = "1.0"
```

## Creating the Reader object

First need to create a `Reader` object giving it something that implements `Iterator<Item=u8>`.
We can read from a string like this:

```rust
use nop_json::Reader;

let mut reader = Reader::new(r#" "a JSON string" "#.bytes());
```

To read from a file we need to convert `std::io::Read` to `Iterator<Item=u8>`. We can use `read_iter` crate for this.

```rust
use std::fs::File;
use read_iter::ReadIter; // also add dependency to Cargo.toml
use nop_json::Reader;

let mut file = ReadIter::new(File::open("/tmp/test.json").unwrap());
let mut reader = Reader::new(&mut file);
```

See `Reader::new()` for more details.

## Deserializing simple values

To read JSON values from the input stream, call `reader.read()` method, and assign the result to a variable that implements `TryFromJson` trait.
This crate adds implementation of `TryFromJson` to many primitive types, `Vec`, `HashMap`, and more.

```rust
use nop_json::Reader;

let mut reader = Reader::new(r#" true  100.5  "Hello"  "Infinity"  [true, false] "#.bytes());

let the_true: bool = reader.read().unwrap();
let the_hundred_point_five: f32 = reader.read().unwrap();
let the_hello: String = reader.read().unwrap();
let the_infinity: f32 = reader.read().unwrap();
let the_array: Vec<bool> = reader.read().unwrap();

assert_eq!(the_true, true);
assert_eq!(the_hundred_point_five, 100.5);
assert_eq!(the_hello, "Hello");
assert!(the_infinity.is_infinite());
assert_eq!(the_array, vec![true, false]);
```

## Deserializing any JSON values

We have generic `Value` type that can hold any JSON node.

```rust
use nop_json::{Reader, Value};
use std::convert::TryInto;

let mut reader = Reader::new(r#" true  100.5  "Hello"  [true, false] "#.bytes());

let the_true: Value = reader.read().unwrap();
let the_hundred_point_five: Value = reader.read().unwrap();
let the_hello: Value = reader.read().unwrap();
let the_array: Value = reader.read().unwrap();

assert_eq!(the_true, Value::Bool(true));
let the_hundred_point_five: f32 = the_hundred_point_five.try_into().unwrap();
assert_eq!(the_hundred_point_five, 100.5f32);
assert_eq!(the_hello, Value::String("Hello".to_string()));
assert_eq!(the_array, Value::Array(vec![Value::Bool(true), Value::Bool(false)]));
```

You can parse any JSON document to `Value`.

```rust
use nop_json::{Reader, Value};

let mut reader = Reader::new(r#" {"array": [{"x": 1}, "a string"]} "#.bytes());
let doc: Value = reader.read().unwrap();
assert_eq!(doc.to_string(), r#"{"array":[{"x":1},"a string"]}"#);
```

## Deserializing/serializing structs and enums

To deserialize a struct or an enum, your struct needs to implement `TryFromJson` trait.
To serialize - `DebugToJson`.

```rust
use nop_json::{Reader, TryFromJson, DebugToJson};

#[derive(TryFromJson, DebugToJson, PartialEq)]
struct Point {x: i32, y: i32}

#[derive(TryFromJson, DebugToJson, PartialEq)]
enum Geometry
{	#[json(point)] Point(Point),
	#[json(cx, cy, r)] Circle(i32, i32, i32),
	Nothing,
}

let mut reader = Reader::new(r#" {"point": {"x": 0, "y": 0}} "#.bytes());
let obj: Geometry = reader.read().unwrap();
println!("Serialized back to JSON: {:?}", obj);
```
See `TryFromJson`, `DebugToJson`.

## Serializing scalar values

You can println!() word "true" or "false" to serialize a boolean. Also numbers can be printed as println!() does by default.
The format is JSON-compatible. To serialize a &str, you can use `escape` function.

Alternatively you can create a `Value` object, and serialize it.
```rust
use std::convert::TryInto;
use nop_json::Value;

let the_true: Value = true.try_into().unwrap();
println!("Serialized to JSON: {:?}", the_true);
```

## Skipping a value from stream

To skip current value without storing it (and allocating memory), read it to the `()` type.
```rust
use nop_json::Reader;

let mut reader = Reader::new(r#" true  100.5  "Hello"  [true, false] "#.bytes());

let _: () = reader.read().unwrap();
let _: () = reader.read().unwrap();
let _: () = reader.read().unwrap();
let _: () = reader.read().unwrap();
```

## Reading binary data
See `read_blob`.

License: MIT
