//! Edge cases and malformed input: EOF, empty/nested containers, skipping values,
//! duplicate keys, escaped object keys, trailing content and assorted lenient quirks.

use nop_json::{Reader, Value};
use std::collections::HashMap;

fn read<T: nop_json::TryFromJson>(json: &str) -> std::io::Result<T>
{	Reader::new(json.bytes()).read()
}

#[test]
fn empty_or_whitespace_input_is_error()
{	assert!(read::<i32>("").is_err());
	assert!(read::<i32>("    ").is_err());
	assert!(read::<Value>("").is_err());
	assert!(read::<String>("\t\n ").is_err());
}

#[test]
fn empty_containers()
{	assert_eq!(read::<Value>("[]").unwrap(), Value::Array(vec![]));
	assert_eq!(read::<Vec<i32>>("[]").unwrap(), Vec::<i32>::new());
	let m: HashMap<String, i32> = read("{}").unwrap();
	assert!(m.is_empty());
	match read::<Value>("{}").unwrap()
	{	Value::Object(o) => assert!(o.is_empty()),
		_ => panic!("expected object"),
	}
}

#[test]
fn deeply_nested_arrays()
{	let depth = 64;
	let json = format!("{}42{}", "[".repeat(depth), "]".repeat(depth));
	let mut v: Value = read(&json).unwrap();
	for _ in 0 .. depth
	{	v = match v
		{	Value::Array(mut a) =>
			{	assert_eq!(a.len(), 1);
				a.pop().unwrap()
			}
			other => panic!("expected array, got {other:?}"),
		};
	}
	assert_eq!(v, Value::Number(42, 0, false));
}

#[test]
fn mixed_type_array_into_value()
{	let v: Value = read(r#"[null, true, 1, "two", [3], {"k": 4}]"#).unwrap();
	match v
	{	Value::Array(a) =>
		{	assert_eq!(a.len(), 6);
			assert!(a[0].is_null());
			assert!(a[1].is_bool());
			assert!(a[2].is_number());
			assert!(a[3].is_string());
			assert!(a[4].is_array());
			assert!(a[5].is_object());
		}
		_ => panic!("expected array"),
	}
}

#[test]
fn skip_value_by_reading_unit()
{	let mut reader = Reader::new(r#" [1, 2, 3]  "next"  42 "#.bytes());
	let _: () = reader.read().unwrap();         // skip the array
	assert_eq!(reader.read::<String>().unwrap(), "next");
	let _: () = reader.read().unwrap();         // skip the number
	// stream now exhausted
	assert!(reader.read::<i32>().is_err());
}

#[test]
fn malformed_json_errors()
{	assert!(read::<Vec<i32>>("[1, 2,").is_err());        // unterminated array
	assert!(read::<Vec<i32>>("[1 2]").is_err());          // missing comma
	assert!(read::<Value>(r#"{"a" 1}"#).is_err());        // missing colon
	assert!(read::<Value>(r#"{"a": }"#).is_err());        // missing value
	assert!(read::<Value>("[1, 2]extra").is_ok());        // trailing data after a full value is fine for a single read
}

#[test]
fn duplicate_keys_last_wins()
{	let m: HashMap<String, i32> = read(r#"{"a": 1, "a": 2}"#).unwrap();
	assert_eq!(m.get("a"), Some(&2));
	assert_eq!(m.len(), 1);
}

#[test]
fn escaped_object_keys_are_decoded()
{	let m: HashMap<String, i32> = read(r#"{"ab": 5}"#).unwrap();
	assert_eq!(m.get("ab"), Some(&5));

	let m: HashMap<String, i32> = read("{\"key\\nwith\\tescapes\": 1}").unwrap();
	assert_eq!(m.get("key\nwith\tescapes"), Some(&1));
}

#[test]
fn reader_continues_after_one_value()
{	// a single read consumes exactly one value; the reader can keep going
	let mut reader = Reader::new(r#"{"a": 1} {"b": 2}"#.bytes());
	let first: HashMap<String, i32> = reader.read().unwrap();
	let second: HashMap<String, i32> = reader.read().unwrap();
	assert_eq!(first.get("a"), Some(&1));
	assert_eq!(second.get("b"), Some(&2));
}

#[test]
fn lenient_numeric_quirks()
{	// leading zeroes are tolerated
	assert_eq!(read::<i32>("007").unwrap(), 7);
	// capital E exponent
	assert_eq!(read::<i32>("1E2").unwrap(), 100);
	// negative zero collapses to zero for integers
	assert_eq!(read::<i32>("-0").unwrap(), 0);
}

#[test]
fn unbalanced_brackets_error()
{	assert!(read::<Value>("[1, 2, 3").is_err());
	assert!(read::<Value>(r#"{"a": 1"#).is_err());
	assert!(read::<Value>("]").is_err());
	assert!(read::<Value>("}").is_err());
}
