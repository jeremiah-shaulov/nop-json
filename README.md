# nop-json

This is full-featured modern JSON implementation according to ECMA-404 standard.

This crate allows deserialization of JSON `io::Read` stream into primitive types (`bool`, `i32`, etc.),
String and any other types that implement special trait called `TryFromJson`, which can be implemented
automatically through `#[derive(TryFromJson)]` for your structs and enums.

And serialization back to JSON through `DebugToJson` trait, that acts like `Debug`, allowing to
print your objects with `println!()` and such.

It allows to read whitespece-separated JSON values from stream in sequence. It also allows to pipe blob strings to a writer.

This implementation avoids unnecessary memory allocations and temporary object creations.

## Examples

### Deserializing simple values

```rust
use nop_json::Reader;

let mut reader = Reader::new(r#" true  100.5  "Hello"  [true, false] "#.as_bytes());

let the_true: bool = reader.read().unwrap();
let the_hundred_point_five: f32 = reader.read().unwrap();
let the_hello: String = reader.read().unwrap();
let the_array: Vec<bool> = reader.read().unwrap();

assert_eq!(the_true, true);
assert_eq!(the_hundred_point_five, 100.5);
assert_eq!(the_hello, "Hello");
assert_eq!(the_array, vec![true, false]);
```
First need to create a `Reader` object giving it something that implements `std::io::Read`. In example above i use `&[u8]`.

Then call reader.read() to read each value from stream to some variable that implements `TryFromJson`.
This crate has implementation of `TryFromJson` for many primitive types, `Vec`, `HashMap`, and more.

### Deserializing any JSON values

```rust
use nop_json::{Reader, Value};
use std::convert::TryInto;

let mut reader = Reader::new(r#" true  100.5  "Hello"  [true, false] "#.as_bytes());

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
We have generic `Value` type that can hold any JSON node.

### Deserializing/serializing objects

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

let mut reader = Reader::new(r#" {"point": {"x": 0, "y": 0}} "#.as_bytes());
let obj: Geometry = reader.read().unwrap();
println!("Serialized back to JSON: {:?}", obj);
```
See `TryFromJson`, `DebugToJson`.

### Serializing scalar values

You can println!() word "true" or "false" to serialize a boolean. Also numbers can be printed as println!() does by default.
The format is JSON-compatible. To serialize a &str, you can use `escape` function.

Alternatively you can create a `Value` object, and serialize it.

### Skipping a value from stream

To skip current value without storing it (and allocating memory), read it to the `()` type.
```rust
use nop_json::Reader;

let mut reader = Reader::new(r#" true  100.5  "Hello"  [true, false] "#.as_bytes());

let _: () = reader.read().unwrap();
let _: () = reader.read().unwrap();
let _: () = reader.read().unwrap();
let _: () = reader.read().unwrap();
```

### Reading binary data
See `read_blob`.

License: MIT
