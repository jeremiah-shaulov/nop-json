//! Serialization via `DebugToJson` / `to_json_string()` for scalars, floats
//! (including Infinity/NaN), char escaping, containers, maps, tuples and `Value`.

use nop_json::{Reader, Value, DebugToJson};
use std::collections::BTreeMap;

#[test]
fn scalars()
{	assert_eq!(0i16.to_json_string(), "0");
	assert_eq!(123i8.to_json_string(), "123");
	assert_eq!((-128i32).to_json_string(), "-128");
	assert_eq!(i64::MIN.to_json_string(), "-9223372036854775808");
	assert_eq!(u64::MAX.to_json_string(), "18446744073709551615");
	assert_eq!(u128::MAX.to_json_string(), "340282366920938463463374607431768211455");
	assert_eq!(true.to_json_string(), "true");
	assert_eq!(false.to_json_string(), "false");
	assert_eq!(().to_json_string(), "null");
	assert_eq!(" Hello ".to_string().to_json_string(), "\" Hello \"");
}

#[test]
fn floats_finite()
{	assert_eq!(0.01f32.to_json_string(), "0.01");
	assert_eq!(100.5f64.to_json_string(), "100.5");
	assert_eq!((-3000.0f64).to_json_string(), "-3000");
}

#[test]
fn floats_infinity_and_nan_are_quoted()
{	// Regression: f64 Infinity used to be emitted unquoted (invalid JSON); NaN was never matched.
	assert_eq!(f64::INFINITY.to_json_string(), "\"Infinity\"");
	assert_eq!(f64::NEG_INFINITY.to_json_string(), "\"-Infinity\"");
	assert_eq!(f64::NAN.to_json_string(), "\"NaN\"");
	assert_eq!(f32::INFINITY.to_json_string(), "\"Infinity\"");
	assert_eq!(f32::NEG_INFINITY.to_json_string(), "\"-Infinity\"");
	assert_eq!(f32::NAN.to_json_string(), "\"NaN\"");
}

#[test]
fn float_special_values_round_trip()
{	for s in [f64::INFINITY, f64::NEG_INFINITY]
	{	let json = s.to_json_string();
		let back: f64 = Reader::new(json.bytes()).read().unwrap();
		assert_eq!(back, s);
	}
	let json = f64::NAN.to_json_string();
	let back: f64 = Reader::new(json.bytes()).read().unwrap();
	assert!(back.is_nan());
}

#[test]
fn chars()
{	assert_eq!('a'.to_json_string(), "\"a\"");
	assert_eq!('\u{05E9}'.to_json_string(), "\"\u{05E9}\"");
	// Regression: control chars must be escaped, and quote/backslash too.
	assert_eq!('"'.to_json_string(), "\"\\\"\"");
	assert_eq!('\\'.to_json_string(), "\"\\\\\"");
	assert_eq!('\n'.to_json_string(), "\"\\n\"");
	assert_eq!('\t'.to_json_string(), "\"\\t\"");
	assert_eq!('\u{0001}'.to_json_string(), "\"\\u0001\"");
}

#[test]
fn strings_escape_on_output()
{	assert_eq!("a\"b\\c\nd".to_string().to_json_string(), "\"a\\\"b\\\\c\\nd\"");
	assert_eq!("\u{0000}".to_string().to_json_string(), "\"\\u0000\"");
}

#[test]
fn collections()
{	assert_eq!(vec![1, 2, 3].to_json_string(), "[1,2,3]");
	assert_eq!(Vec::<i32>::new().to_json_string(), "[]");
	assert_eq!(vec![vec![1, 2], vec![3]].to_json_string(), "[[1,2],[3]]");
	assert_eq!(vec!["a".to_string(), "b".to_string()].to_json_string(), "[\"a\",\"b\"]");
}

#[test]
fn options_and_boxes()
{	assert_eq!(Some(5).to_json_string(), "5");
	assert_eq!(None::<i32>.to_json_string(), "null");
	assert_eq!(Box::new(7).to_json_string(), "7");
	assert_eq!(vec![Some(1), None, Some(3)].to_json_string(), "[1,null,3]");
}

#[test]
fn tuples()
{	assert_eq!((1, "a".to_string()).to_json_string(), "[1,\"a\"]");
	assert_eq!((1, 2, 3).to_json_string(), "[1,2,3]");
}

#[test]
fn maps_use_deterministic_btree_order()
{	let mut m = BTreeMap::new();
	m.insert("b".to_string(), 2);
	m.insert("a".to_string(), 1);
	assert_eq!(m.to_json_string(), "{\"a\":1,\"b\":2}");

	let empty: BTreeMap<String, i32> = BTreeMap::new();
	assert_eq!(empty.to_json_string(), "{}");
}

#[test]
fn map_keys_are_escaped()
{	let mut m = BTreeMap::new();
	m.insert("a\"b".to_string(), 1);
	assert_eq!(m.to_json_string(), "{\"a\\\"b\":1}");
}

#[test]
fn value_display_and_debug()
{	let v: Value = Reader::new(r#" {"a": [1, "two", null, true]} "#.bytes()).read().unwrap();
	// HashMap-backed object with a single key is deterministic
	assert_eq!(v.to_string(), "{\"a\":[1,\"two\",null,true]}");
	assert_eq!(format!("{:?}", v), "{\"a\":[1,\"two\",null,true]}");
	assert_eq!(Value::Null.to_string(), "null");
	assert_eq!(Value::Array(vec![]).to_string(), "[]");
}
