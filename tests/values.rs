//! The `Value` type: predicates, indexing, and `TryFrom`/`TryInto` conversions both directions.

use nop_json::{Reader, Value};
use std::convert::TryInto;
use std::collections::HashMap;

#[test]
fn predicates()
{	let v: Value = Reader::new(r#" [null, false, 12.3, "12.3", [], {}] "#.bytes()).read().unwrap();
	let items = match v {Value::Array(a) => a, _ => unreachable!()};
	assert!(items[0].is_null());
	assert!(items[1].is_bool());
	assert!(items[2].is_number());
	assert!(items[3].is_string());
	assert!(items[4].is_array());
	assert!(items[5].is_object());
	// each predicate is false for the others
	assert!(!items[0].is_bool());
	assert!(!items[2].is_string());
	assert!(!items[4].is_object());
}

#[test]
fn value_to_int()
{	let v: i32 = Value::Number(123, 0, false).try_into().unwrap();
	assert_eq!(v, 123);
	let v: i32 = Value::Number(3, 3, true).try_into().unwrap();
	assert_eq!(v, -3000);
	let v: u32 = Value::Number(3, 3, false).try_into().unwrap();
	assert_eq!(v, 3000);
	// negative into unsigned -> error
	assert!(TryInto::<u32>::try_into(Value::Number(1, 0, true)).is_err());
	// overflow -> error
	assert!(TryInto::<i8>::try_into(Value::Number(200, 0, false)).is_err());
}

#[test]
fn value_to_float()
{	let v: f64 = Value::Number(1234, -2, false).try_into().unwrap();
	assert!((v - 12.34).abs() < 1e-9);
	let v: f64 = Value::Number(5, 0, true).try_into().unwrap();
	assert_eq!(v, -5.0);
	let v: f64 = Value::Null.try_into().unwrap();
	assert_eq!(v, 0.0);
}

#[test]
fn value_to_bool()
{	assert_eq!(TryInto::<bool>::try_into(Value::Null).unwrap(), false);
	assert_eq!(TryInto::<bool>::try_into(Value::Bool(true)).unwrap(), true);
	assert_eq!(TryInto::<bool>::try_into(Value::Number(0, 0, false)).unwrap(), false);
	assert_eq!(TryInto::<bool>::try_into(Value::Number(5, 0, false)).unwrap(), true);
	assert_eq!(TryInto::<bool>::try_into(Value::String("x".to_string())).unwrap(), true);
}

#[test]
fn value_to_string_from_nonnumbers()
{	assert_eq!(TryInto::<String>::try_into(Value::Null).unwrap(), "null");
	assert_eq!(TryInto::<String>::try_into(Value::Bool(true)).unwrap(), "true");
	assert_eq!(TryInto::<String>::try_into(Value::String("hi".to_string())).unwrap(), "hi");
}

#[test]
fn value_to_char()
{	assert_eq!(TryInto::<char>::try_into(Value::String("abc".to_string())).unwrap(), 'a');
	assert_eq!(TryInto::<char>::try_into(Value::Bool(true)).unwrap(), 't');
	assert_eq!(TryInto::<char>::try_into(Value::Null).unwrap(), 'n');
	assert!(TryInto::<char>::try_into(Value::String(String::new())).is_err());
}

#[test]
fn value_to_vec()
{	let v = Value::Array(vec![Value::Bool(true), Value::Bool(false)]);
	let out: Vec<bool> = v.try_into().unwrap();
	assert_eq!(out, vec![true, false]);

	let v = Value::Array(vec![Value::Number(1, 0, false), Value::Number(2, 0, false)]);
	let out: Vec<i32> = v.try_into().unwrap();
	assert_eq!(out, vec![1, 2]);

	// null converts to an empty vec
	let out: Vec<i32> = Value::Null.try_into().unwrap();
	assert_eq!(out, Vec::<i32>::new());
}

#[test]
fn into_value_from_primitives()
{	let v: Value = 3u32.try_into().unwrap();
	assert_eq!(v, Value::Number(3, 0, false));
	let v: Value = (-5i32).try_into().unwrap();
	assert_eq!(v, Value::Number(5, 0, true));
	let v: Value = true.try_into().unwrap();
	assert_eq!(v, Value::Bool(true));
	let v: Value = ().try_into().unwrap();
	assert_eq!(v, Value::Null);
	let v: Value = "hi".to_string().try_into().unwrap();
	assert_eq!(v, Value::String("hi".to_string()));
	let v: Value = 'z'.try_into().unwrap();
	assert_eq!(v, Value::String("z".to_string()));
}

#[test]
fn into_value_from_collections()
{	let v: Value = vec![true, false, true].try_into().unwrap();
	assert_eq!(v, Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)]));

	let v: Value = vec![1i32, 2, 3].try_into().unwrap();
	assert_eq!(v, Value::Array(vec![Value::Number(1, 0, false), Value::Number(2, 0, false), Value::Number(3, 0, false)]));
}

#[test]
fn from_str()
{	use std::str::FromStr;
	assert_eq!(Value::from_str("abc").unwrap(), Value::String("abc".to_string()));
	assert_eq!("xyz".parse::<Value>().unwrap(), Value::String("xyz".to_string()));
}

#[test]
fn index_by_key()
{	let mut obj = HashMap::new();
	obj.insert("name".to_string(), Value::String("John".to_string()));
	let v = Value::Object(obj);
	assert_eq!(v["name"], Value::String("John".to_string()));
	// missing key -> Null
	assert_eq!(v["missing"], Value::Null);
	// indexing a non-object -> Null
	assert_eq!(Value::Null["whatever"], Value::Null);
}
