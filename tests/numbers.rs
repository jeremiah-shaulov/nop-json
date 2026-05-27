//! Number parsing (integers and floats), range/overflow handling, exponents,
//! reading numbers that are encoded as JSON strings, and number -> string formatting.

use nop_json::{Reader, Value};

fn read_one<T: nop_json::TryFromJson>(json: &str) -> std::io::Result<T>
{	Reader::new(json.bytes()).read()
}

#[test]
fn integers_basic()
{	assert_eq!(read_one::<i32>("0").unwrap(), 0);
	assert_eq!(read_one::<i32>("123").unwrap(), 123);
	assert_eq!(read_one::<i32>("-123").unwrap(), -123);
	assert_eq!(read_one::<u32>("4000000000").unwrap(), 4000000000);
}

#[test]
fn integer_type_boundaries()
{	// i8
	assert_eq!(read_one::<i8>("127").unwrap(), 127);
	assert_eq!(read_one::<i8>("-128").unwrap(), -128);
	assert!(read_one::<i8>("128").is_err());
	assert!(read_one::<i8>("-129").is_err());
	// u8
	assert_eq!(read_one::<u8>("0").unwrap(), 0);
	assert_eq!(read_one::<u8>("255").unwrap(), 255);
	assert!(read_one::<u8>("256").is_err());
	assert!(read_one::<u8>("-1").is_err());
	// i16
	assert_eq!(read_one::<i16>("32767").unwrap(), 32767);
	assert_eq!(read_one::<i16>("-32768").unwrap(), -32768);
	assert!(read_one::<i16>("32768").is_err());
	// u16
	assert_eq!(read_one::<u16>("65535").unwrap(), 65535);
	assert!(read_one::<u16>("65536").is_err());
}

#[test]
fn integers_64_and_128()
{	assert_eq!(read_one::<i64>("9223372036854775807").unwrap(), i64::MAX);
	assert_eq!(read_one::<i64>("-9223372036854775808").unwrap(), i64::MIN);
	assert_eq!(read_one::<u64>("18446744073709551615").unwrap(), u64::MAX);
	assert_eq!(read_one::<i128>("170141183460469231731687303715884105727").unwrap(), i128::MAX);
	assert_eq!(read_one::<u128>("340282366920938463463374607431768211455").unwrap(), u128::MAX);
	// overflow of u64
	assert!(read_one::<u64>("18446744073709551616").is_err());
}

#[test]
fn integers_with_exponent()
{	assert_eq!(read_one::<i32>("1e2").unwrap(), 100);
	assert_eq!(read_one::<i32>("1e+2").unwrap(), 100);
	assert_eq!(read_one::<i8>("-1e2").unwrap(), -100);
	assert_eq!(read_one::<i32>("12e3").unwrap(), 12000);
}

#[test]
fn integers_truncate_fraction()
{	// digits after the dot are ignored when reading into an integer
	assert_eq!(read_one::<i32>("12.99").unwrap(), 12);
	assert_eq!(read_one::<u16>("0.01").unwrap(), 0);
	assert_eq!(read_one::<i32>("-7.5").unwrap(), -7);
}

#[test]
fn integers_from_bool_and_null()
{	assert_eq!(read_one::<i32>("true").unwrap(), 1);
	assert_eq!(read_one::<i32>("false").unwrap(), 0);
	assert_eq!(read_one::<i32>("null").unwrap(), 0);
}

#[test]
fn integers_from_string()
{	assert_eq!(read_one::<i32>("\"123\"").unwrap(), 123);
	assert_eq!(read_one::<i32>("\"-45\"").unwrap(), -45);
	assert_eq!(read_one::<i8>("\"-1e+2\"").unwrap(), -100);
	assert!(read_one::<i8>("\"128\"").is_err());
	// non-numeric string reads as 0
	assert_eq!(read_one::<i32>("\"hello\"").unwrap(), 0);
}

#[test]
fn floats_basic()
{	assert_eq!(read_one::<f64>("3000.0").unwrap(), 3000.0);
	assert_eq!(read_one::<f64>("-3000.0").unwrap(), -3000.0);
	assert_eq!(read_one::<f64>("0.5").unwrap(), 0.5);
	assert_eq!(read_one::<f32>("100.5").unwrap(), 100.5);

	let n: f64 = read_one("123e-7").unwrap();
	assert!((n - 123e-7).abs() < 1e-16);
}

#[test]
fn floats_signed_zero()
{	let mzero: f64 = read_one("\"-0\"").unwrap();
	assert!(mzero == 0.0 && mzero.is_sign_negative());
	let zero: f64 = read_one("\"0\"").unwrap();
	assert!(zero == 0.0 && !zero.is_sign_negative());
}

#[test]
fn floats_special_values()
{	assert!(read_one::<f64>("\"Infinity\"").unwrap().is_infinite());
	assert!(read_one::<f64>("\"Infinity\"").unwrap() > 0.0);
	assert!(read_one::<f64>("\"-Infinity\"").unwrap().is_infinite());
	assert!(read_one::<f64>("\"-Infinity\"").unwrap() < 0.0);
	assert!(read_one::<f64>("\"NaN\"").unwrap().is_nan());
	assert!(read_one::<f64>("\"hello\"").unwrap().is_nan());
	assert!(read_one::<f32>("\"Infinity\"").unwrap().is_infinite());
}

#[test]
fn floats_underflow_overflow()
{	assert_eq!(read_one::<f32>("1e-1000000").unwrap(), 0.0);     // underflow -> 0
	assert!(read_one::<f64>("1e10000000000").unwrap().is_nan()); // exponent overflow -> NaN
}

#[test]
fn floats_from_bool_and_null()
{	assert_eq!(read_one::<f64>("true").unwrap(), 1.0);
	assert_eq!(read_one::<f64>("false").unwrap(), 0.0);
	assert_eq!(read_one::<f64>("null").unwrap(), 0.0);
}

/// Numbers read into a `String` get canonically formatted.
#[test]
fn number_to_string_formatting()
{	let cases =
	[	("123", "123"),
		("12.3", "12.3"),
		("0.123", "0.123"),
		("0.0123", "0.0123"),
		("123e3", "123000"),
		("123e7", "1230000000"),
		("123e8", "123e8"),
		("123e-1", "12.3"),
		("123e-3", "0.123"),
		("123e-13", "123e-13"),
		("0", "0"),
		("-0", "0"),
		("1000", "1000"),
		("1000000000", "1000000000"),  // 10^9, stays expanded
		("10000000000", "1e10"),       // 10^10, switches to exponent form
		("-3000", "-3000"),
	];
	for (input, expected) in cases
	{	assert_eq!(read_one::<String>(input).unwrap(), expected, "input was {input}");
	}
}

/// `Value::to_string()` (the Display impl) must format numbers identically to reading into a String.
#[test]
fn value_number_to_string_matches()
{	let cases = ["123", "12.3", "0.123", "0.0123", "123e3", "123e7", "123e8", "123e-1", "123e-3", "123e-13", "10000000000"];
	for input in cases
	{	let via_string = read_one::<String>(input).unwrap();
		let via_value = read_one::<Value>(input).unwrap().to_string();
		assert_eq!(via_string, via_value, "mismatch for {input}");
	}
}

/// And `String::try_from(Value)` (a separate code path) must agree too.
#[test]
fn value_tryinto_string_matches()
{	use std::convert::TryInto;
	let cases = ["123", "12.3", "0.123", "123e8", "123e-3", "10000000000", "-3000"];
	for input in cases
	{	let canonical = read_one::<String>(input).unwrap();
		let value: Value = read_one(input).unwrap();
		let converted: String = value.try_into().unwrap();
		assert_eq!(canonical, converted, "mismatch for {input}");
	}
}

#[test]
fn value_number_representation()
{	assert_eq!(read_one::<Value>("-3000").unwrap(), Value::Number(3, 3, true));
	assert_eq!(read_one::<Value>("-3000.0").unwrap(), Value::Number(3, 3, true));
	assert_eq!(read_one::<Value>("-3000.00e1").unwrap(), Value::Number(3, 4, true));
	assert_eq!(read_one::<Value>("-3000.00e-1").unwrap(), Value::Number(3, 2, true));
	assert_eq!(read_one::<Value>("-1234.56e-1").unwrap(), Value::Number(123456, -3, true));
	assert_eq!(read_one::<Value>("0").unwrap(), Value::Number(0, 0, false));
}
