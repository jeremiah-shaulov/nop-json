//! The `WriteToJson` trait: writing to an `io::Write` sink, derive, manual impl,
//! and agreement with `DebugToJson::to_json_string()`.

use nop_json::{WriteToJson, DebugToJson};
use std::io;
use std::collections::BTreeMap;

fn to_json<T: WriteToJson<Vec<u8>>>(value: T) -> String
{	let mut out: Vec<u8> = Vec::new();
	value.write_to_json(&mut out).unwrap();
	String::from_utf8(out).unwrap()
}

#[test]
fn scalars()
{	assert_eq!(to_json(0i16), "0");
	assert_eq!(to_json(123i8), "123");
	assert_eq!(to_json(-128i32), "-128");
	assert_eq!(to_json(true), "true");
	assert_eq!(to_json(false), "false");
	assert_eq!(to_json(()), "null");
	assert_eq!(to_json(" Hello ".to_string()), "\" Hello \"");
}

#[test]
fn floats_special_values()
{	assert_eq!(to_json(f32::INFINITY), "\"Infinity\"");
	assert_eq!(to_json(f64::INFINITY), "\"Infinity\"");
	assert_eq!(to_json(f64::NEG_INFINITY), "\"-Infinity\"");
	assert_eq!(to_json(f64::NAN), "\"NaN\"");
}

#[test]
fn chars_are_escaped()
{	assert_eq!(to_json('a'), "\"a\"");
	assert_eq!(to_json('\n'), "\"\\n\"");
	assert_eq!(to_json('"'), "\"\\\"\"");
}

#[test]
fn collections_and_maps()
{	assert_eq!(to_json(vec![1, 2, 3]), "[1,2,3]");
	assert_eq!(to_json(Vec::<i32>::new()), "[]");
	assert_eq!(to_json(Some(5)), "5");
	assert_eq!(to_json(None::<i32>), "null");
	assert_eq!(to_json(Box::new(9)), "9");

	let mut m = BTreeMap::new();
	m.insert("a".to_string(), 1);
	m.insert("b".to_string(), 2);
	assert_eq!(to_json(m), "{\"a\":1,\"b\":2}");
}

#[test]
fn derived_struct()
{	#[derive(WriteToJson)]
	struct Person<T>
	{	first_name: T,
		last_name: T,
	}
	let p = Person {first_name: "John".to_string(), last_name: "Doe".to_string()};
	assert_eq!(to_json(p), "{\"first_name\":\"John\",\"last_name\":\"Doe\"}");
}

#[test]
fn manual_impl()
{	struct Point {x: i32, y: i32}
	impl<W: io::Write> WriteToJson<W> for Point
	{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
		{	write!(out, "{{\"x\":")?;
			self.x.write_to_json(out)?;
			write!(out, ",\"y\":")?;
			self.y.write_to_json(out)?;
			write!(out, "}}")
		}
	}
	assert_eq!(to_json(Point {x: 1, y: 2}), "{\"x\":1,\"y\":2}");
}

#[test]
fn matches_debug_to_json()
{	// the two serialization paths should agree
	assert_eq!(to_json(vec![1, 2, 3]), vec![1, 2, 3].to_json_string());
	assert_eq!(to_json(f64::INFINITY), f64::INFINITY.to_json_string());
	assert_eq!(to_json('\t'), '\t'.to_json_string());
	assert_eq!(to_json("a\"b".to_string()), "a\"b".to_string().to_json_string());
}
