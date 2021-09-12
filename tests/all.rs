use nop_json::{Reader, Value, TryFromJson, ValidateJson, DebugToJson, WriteToJson, escape, escape_bytes};
use std::io::{self, Error};
use std::f32;

#[test]
fn test_number()
{	let mut reader = Reader::new(r#" 0 0.01 123 128 -128 -129 255 -1 -1e+2 1e2 123e-7 0.0123e-10 3000.0 -3000.0 1e-1000000 1e10000000000 "Infinity" "-Infinity" "hello" true null "#.bytes());
	assert_eq!(reader.read::<i16>().unwrap(), 0); // 0
	assert_eq!(reader.read::<u16>().unwrap(), 0); // 0.01
	assert_eq!(reader.read::<i8>().unwrap(), 123); // 123
	assert!(reader.read::<i8>().is_err()); // 128
	assert_eq!(reader.read::<i8>().unwrap(), -128); // -128
	assert!(reader.read::<i8>().is_err()); // -129
	assert_eq!(reader.read::<u8>().unwrap(), 255); // 255
	assert!(reader.read::<u8>().is_err()); // -1
	assert_eq!(reader.read::<i8>().unwrap(), -100); // -1e+2
	assert_eq!(reader.read::<i8>().unwrap(), 100); // 1e2
	let n: f32 = reader.read().unwrap(); // 123e-7
	assert!(n > 123e-7 - 0.1e-10 && n < 123e-7 + 0.1e-10);
	let n: f64 = reader.read().unwrap(); // 0.0123e-10
	assert!(n > 0.0123e-10 - 0.1e-16 && n < 0.0123e-10 + 0.1e-16);
	assert_eq!(reader.read::<f64>().unwrap(), 3000.0); // 3000.0
	assert_eq!(reader.read::<f64>().unwrap(), -3000.0); // -3000.0
	assert_eq!(reader.read::<f32>().unwrap(), 0.0); // 1e-1000000
	assert!(reader.read::<f64>().unwrap().is_nan()); // 1e10000000000
	assert!(reader.read::<f64>().unwrap().is_infinite()); // "Infinity"
	assert!(reader.read::<f64>().unwrap().is_infinite()); // "-Infinity"
	assert!(reader.read::<f64>().unwrap().is_nan()); // "hello"
	assert_eq!(reader.read::<i16>().unwrap(), 1); // true
	assert_eq!(reader.read::<f32>().unwrap(), 0.0); // null
}

#[test]
fn test_number_as_string()
{	let mut reader = Reader::new(r#" "0" "0.01" "123" "128" "-128" "-129" "255" "-1" "-1e+2" "1e2" "123e-7" "0.0123e-10" "1e-1000000" "1e10000000000" "Infinity" "-Infinity" "hello" true "#.bytes());
	assert_eq!(reader.read::<i16>().unwrap(), 0); // 0
	assert_eq!(reader.read::<u16>().unwrap(), 0); // 0.01
	assert_eq!(reader.read::<i8>().unwrap(), 123); // 123
	assert!(reader.read::<i8>().is_err()); // 128
	assert_eq!(reader.read::<i8>().unwrap(), -128); // -128
	assert!(reader.read::<i8>().is_err()); // -129
	assert_eq!(reader.read::<u8>().unwrap(), 255); // 255
	assert!(reader.read::<u8>().is_err()); // -1
	assert_eq!(reader.read::<i8>().unwrap(), -100); // -1e+2
	assert_eq!(reader.read::<i8>().unwrap(), 100); // 1e2
	let n: f32 = reader.read().unwrap(); // 123e-7
	assert!(n > 123e-7 - 0.1e-10 && n < 123e-7 + 0.1e-10);
	let n: f64 = reader.read().unwrap(); // 0.0123e-10
	assert!(n > 0.0123e-10 - 0.1e-16 && n < 0.0123e-10 + 0.1e-16);
	assert_eq!(reader.read::<f32>().unwrap(), 0.0); // 1e-1000000
	assert!(reader.read::<f64>().unwrap().is_nan()); // 1e10000000000
	assert!(reader.read::<f64>().unwrap().is_infinite()); // "Infinity"
	assert!(reader.read::<f64>().unwrap().is_infinite()); // "-Infinity"
	assert!(reader.read::<f64>().unwrap().is_nan()); // "hello"
	assert_eq!(reader.read::<i16>().unwrap(), 1); // true
}

#[test]
fn test_number_to_string()
{	let mut reader = Reader::new("123 12.3 0.123 0.0123 123e3 123e7 123e8 123e-1 123e-3 123e-13".bytes());
	assert_eq!(reader.read::<String>().unwrap(), "123");
	assert_eq!(reader.read::<String>().unwrap(), "12.3");
	assert_eq!(reader.read::<String>().unwrap(), "0.123");
	assert_eq!(reader.read::<String>().unwrap(), "0.0123");
	assert_eq!(reader.read::<String>().unwrap(), "123000");
	assert_eq!(reader.read::<String>().unwrap(), "1230000000");
	assert_eq!(reader.read::<String>().unwrap(), "123e8");
	assert_eq!(reader.read::<String>().unwrap(), "12.3");
	assert_eq!(reader.read::<String>().unwrap(), "0.123");
	assert_eq!(reader.read::<String>().unwrap(), "123e-13");
}

#[test]
fn test_value_number_to_string()
{	let mut reader = Reader::new("123 12.3 0.123 0.0123 123e3 123e7 123e8 123e-1 123e-3 123e-13".bytes());
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "123");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "12.3");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "0.123");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "0.0123");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "123000");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "1230000000");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "123e8");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "12.3");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "0.123");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "123e-13");
}

#[test]
fn test_string()
{	let mut reader = Reader::new(r#" "abc" "abcdefghijklmnopqrstuvwxyz" "שלום" "\u0061\u0062\u0063 \u05E9\u05Dc\u05d5\u05dd" "\\ \n" "#.bytes());
	assert_eq!(reader.read::<String>().unwrap(), "abc");
	assert_eq!(reader.read::<String>().unwrap(), "abcdefghijklmnopqrstuvwxyz");
	assert_eq!(reader.read::<String>().unwrap(), "שלום");
	assert_eq!(reader.read::<String>().unwrap(), "abc שלום");
	assert_eq!(reader.read::<String>().unwrap(), "\\ \n");
}

#[test]
fn test_bytes()
{	let mut reader = Reader::new(r#" "abc" "abcdefghijklmnopqrstuvwxyz" "שלום" "\u0061\u0062\u0063 \u05E9\u05Dc\u05d5\u05dd" "\\ \n" "#.bytes());
	assert_eq!(reader.read_bytes().unwrap(), b"abc");
	assert_eq!(reader.read_bytes().unwrap(), b"abcdefghijklmnopqrstuvwxyz");
	assert_eq!(reader.read_bytes().unwrap(), "שלום".as_bytes());
	assert_eq!(reader.read_bytes().unwrap(), "abc שלום".as_bytes());
	assert_eq!(reader.read_bytes().unwrap(), b"\\ \n");
}

#[test]
fn test_char()
{	let mut reader = Reader::new(r#" "abc" "א" "\u05E9" "#.bytes());
	assert_eq!(reader.read::<char>().unwrap(), 'a');
	assert_eq!(reader.read::<char>().unwrap(), 'א');
	assert_eq!(reader.read::<char>().unwrap(), 'ש');
}

#[test]
fn test_object()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct User
	{	id: usize,
		name: String,
		#[json(all_posts)] posts: Vec<Post>,
	}

	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Post
	{	id: usize,
		title: String,
		is_published: bool,
	}

	// Test 1: Record (enum with type)
	#[derive(PartialEq, TryFromJson, ValidateJson, DebugToJson)]
	#[json(type)]
	enum Record
	{	#[json(user("user-data"))] User(User),
		#[json("timestamp-data")] Timestamp(u32),
	}
	let input =
	r#"	[	{	"type": "user",
				"user-data":
				{	"id": 1,
					"name": "John",
					"all_posts":
					[	{	"id": "1",
							"title": "Hello",
							"is_published": true
						},
						{	"id": "2",
							"title": "Subj",
							"is_published": false
						}
					]
				}
			},
			{	"type": "Timestamp",
				"timestamp-data": 1611495933
			}
		]
	"#;
	let expected_result = vec!
	[	Record::User
		(	User
			{	id: 1,
				name: "John".to_string(),
				posts: vec!
				[	Post
					{	id: 1,
						title: "Hello".to_string(),
						is_published: true,
					},
					Post
					{	id: 2,
						title: "Subj".to_string(),
						is_published: false,
					},
				]
			}
		),
		Record::Timestamp(1611495933),
	];
	let mut reader = Reader::new(input.bytes());
	let subj_0: Vec<Record> = reader.read().unwrap();
	assert_eq!(subj_0, expected_result);

	// Test 2: Record2 (enum without type)
	#[derive(PartialEq, TryFromJson, ValidateJson, DebugToJson)]
	enum Record2
	{	#[json(user)] User(User),
		#[json("timestamp")] Timestamp(u32),
	}
	let input =
	r#"	[	{	"user":
				{	"id": 1,
					"name": "John",
					"all_posts":
					[	{	"id": "1",
							"title": "Hello",
							"is_published": true
						},
						{	"id": "2",
							"title": "Subj",
							"is_published": false
						}
					]
				}
			},
			{	"timestamp": 1611495933
			}
		]
	"#;
	let expected_result = vec!
	[	Record2::User
		(	User
			{	id: 1,
				name: "John".to_string(),
				posts: vec!
				[	Post
					{	id: 1,
						title: "Hello".to_string(),
						is_published: true,
					},
					Post
					{	id: 2,
						title: "Subj".to_string(),
						is_published: false,
					},
				]
			}
		),
		Record2::Timestamp(1611495933),
	];
	let mut reader = Reader::new(input.bytes());
	let subj_0: Vec<Record2> = reader.read().unwrap();
	assert_eq!(subj_0, expected_result);
}

#[test]
fn test_escape()
{	assert_eq!(escape("abc"), "abc");
	assert_eq!(escape("a\\b\"c\n"), "a\\\\b\\\"c\\n");
}

#[test]
fn test_escape_bytes()
{	assert_eq!(escape_bytes(b"abc").as_ref(), b"abc");
	assert_eq!(escape_bytes(b"a\\b\"c\n").as_ref(), b"a\\\\b\\\"c\\n");
}

#[test]
fn test_value()
{	use std::convert::TryInto;

	let mut reader = Reader::new(r#" 123 "#.bytes());
	let v: Value = reader.read().unwrap();
	let v: i32 = v.try_into().unwrap();
	assert_eq!(v, 123);

	assert_eq!(Reader::new(r#" [null] "#.bytes()).read::<Value>().unwrap(), Value::Array(vec![Value::Null]));
	assert_eq!(Reader::new(r#" [false] "#.bytes()).read::<Value>().unwrap(), Value::Array(vec![Value::Bool(false)]));
	assert_eq!(Reader::new(r#" [true] "#.bytes()).read::<Value>().unwrap(), Value::Array(vec![Value::Bool(true)]));
	assert_eq!(Reader::new(r#" [-3000] "#.bytes()).read::<Value>().unwrap(), Value::Array(vec![Value::Number(3, 3, true)]));
	assert_eq!(Reader::new(r#" [-3000.0] "#.bytes()).read::<Value>().unwrap(), Value::Array(vec![Value::Number(3, 3, true)]));
	assert_eq!(Reader::new(r#" [-3000.00e1] "#.bytes()).read::<Value>().unwrap(), Value::Array(vec![Value::Number(3, 4, true)]));
	assert_eq!(Reader::new(r#" [-3000.00e-1] "#.bytes()).read::<Value>().unwrap(), Value::Array(vec![Value::Number(3, 2, true)]));
}

#[test]
fn test_pipe()
{	use std::io;

	const READER_BUFFER_SIZE: usize = 128;
	const N_CHARS: usize = 300;
	let mut data = Vec::new();
	let mut expected_data = Vec::new();
	data.push(b'"');
	for i in 0..N_CHARS
	{	let i = (i % 256) as u8;
		if i < 32
		{	for c in format!("\\u{:04x}", i).chars()
			{	data.push(c as u8);
			}
		}
		else if i==b'\\' || i==b'"'
		{	data.push(b'\\');
			data.push(i);
		}
		else
		{	data.push(i);
		}
		expected_data.push(i);
	}
	data.push(b'"');

	struct Writer
	{	data: Vec<u8>,
		n_parts: usize,
	}
	impl io::Write for Writer
	{	fn write(&mut self, buf: &[u8]) -> io::Result<usize>
		{	self.data.extend_from_slice(buf);
			self.n_parts += 1;
			Ok(buf.len())
		}

		fn flush(&mut self) -> Result<(), Error>
		{	Ok(())
		}
	}

	let mut writer = Writer {data: Vec::new(), n_parts: 0};

	let mut reader = Reader::new(data.into_iter());
	reader.pipe_blob(&mut writer).unwrap();

	assert_eq!(writer.data, expected_data);
	assert_eq!(writer.n_parts, (N_CHARS as f64 / READER_BUFFER_SIZE as f64).ceil() as usize);
}

#[test]
fn test_to_json_string()
{	assert_eq!(&0i16.to_json_string(), "0");
	assert_eq!(&0.01f32.to_json_string(), "0.01");
	assert_eq!(&123i8.to_json_string(), "123");
	assert_eq!(&(-128i32).to_json_string(), "-128");
	assert_eq!(&f32::INFINITY.to_json_string(), "\"Infinity\"");
	assert_eq!(&true.to_json_string(), "true");
	assert_eq!(&false.to_json_string(), "false");
	assert_eq!(&" Hello ".to_string().to_json_string(), "\" Hello \"");
}

#[test]
fn test_write()
{	fn to_json<T>(input: T) -> String where T: WriteToJson<Vec<u8>>
	{	let mut out: Vec<u8> = Vec::new();
		input.write_to_json(&mut out).unwrap();
		String::from_utf8_lossy(&out).into_owned()
	}

	// auto derive WriteToJson
	#[derive(PartialEq, WriteToJson)]
	struct Person<T>
	{	first_name: T,
		last_name: T,
	}

	// implement WriteToJson manually
	#[derive(PartialEq)]
	struct Person2
	{	first_name: String,
		last_name: String,
	}
	impl<W> WriteToJson<W> for Person2 where W: io::Write
	{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
		{	write!(out, "{{\"first_name\":")?;
			self.first_name.write_to_json(out)?;
			write!(out, ",\"last_name\":")?;
			self.last_name.write_to_json(out)?;
			write!(out, "}}")
		}
	}

	assert_eq!(&to_json(0i16), "0");
	assert_eq!(&to_json(0.01f32), "0.01");
	assert_eq!(&to_json(123i8), "123");
	assert_eq!(&to_json(-128i32), "-128");
	assert_eq!(&to_json(f32::INFINITY), "\"Infinity\"");
	assert_eq!(&to_json(true), "true");
	assert_eq!(&to_json(false), "false");
	assert_eq!(&to_json(" Hello ".to_string()), "\" Hello \"");
	assert_eq!(&to_json(Person {first_name: "John".to_string(), last_name: "Doe".to_string()}), "{\"first_name\":\"John\",\"last_name\":\"Doe\"}");
	assert_eq!(&to_json(Person2 {first_name: "John".to_string(), last_name: "Doe".to_string()}), "{\"first_name\":\"John\",\"last_name\":\"Doe\"}");
}

#[test]
fn test_value_is()
{	let v: Value = Reader::new(r#" [null, false, 12.3, "12.3", [], {}] "#.bytes()).read().unwrap();
	assert_eq!(v.is_array(), true);
	match v
	{	Value::Array(v) =>
		{	assert_eq!(v[0].is_null(), true);
			assert_eq!(v[0].is_bool(), false);
			assert_eq!(v[0].is_number(), false);
			assert_eq!(v[0].is_string(), false);
			assert_eq!(v[0].is_array(), false);
			assert_eq!(v[0].is_object(), false);

			assert_eq!(v[1].is_null(), false);
			assert_eq!(v[1].is_bool(), true);
			assert_eq!(v[1].is_number(), false);
			assert_eq!(v[1].is_string(), false);
			assert_eq!(v[1].is_array(), false);
			assert_eq!(v[1].is_object(), false);

			assert_eq!(v[2].is_null(), false);
			assert_eq!(v[2].is_bool(), false);
			assert_eq!(v[2].is_number(), true);
			assert_eq!(v[2].is_string(), false);
			assert_eq!(v[2].is_array(), false);
			assert_eq!(v[2].is_object(), false);

			assert_eq!(v[3].is_null(), false);
			assert_eq!(v[3].is_bool(), false);
			assert_eq!(v[3].is_number(), false);
			assert_eq!(v[3].is_string(), true);
			assert_eq!(v[3].is_array(), false);
			assert_eq!(v[3].is_object(), false);

			assert_eq!(v[4].is_null(), false);
			assert_eq!(v[4].is_bool(), false);
			assert_eq!(v[4].is_number(), false);
			assert_eq!(v[4].is_string(), false);
			assert_eq!(v[4].is_array(), true);
			assert_eq!(v[4].is_object(), false);

			assert_eq!(v[5].is_null(), false);
			assert_eq!(v[5].is_bool(), false);
			assert_eq!(v[5].is_number(), false);
			assert_eq!(v[5].is_string(), false);
			assert_eq!(v[5].is_array(), false);
			assert_eq!(v[5].is_object(), true);
		}
		_ => unreachable!()
	}
}
