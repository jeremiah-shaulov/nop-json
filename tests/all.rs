use nop_json::{Reader, Value, TryFromJson, DebugToJson, escape, escape_bytes};
use std::io::Error;
use std::f32;

#[test]
fn test_number()
{	let mut reader = Reader::new(r#" 0 0.01 123 128 -128 -129 255 -1 -1e+2 1e2 123e-7 0.0123e-10 1e-1000000 1e10000000000 "Infinity" "-Infinity" "hello" true "#.bytes());
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
{	#[derive(PartialEq, Default, TryFromJson, DebugToJson)]
	struct Person
	{	first_name: String,
		last_name: String,
	}

	#[derive(PartialEq, TryFromJson, DebugToJson)]
	struct Song
	{	name: String,
		year: i32,
		#[json(special_artist)] artist: Person,
	}

	#[derive(PartialEq, TryFromJson, DebugToJson)]
	#[json(what)]
	enum Subj
	{	#[json(sng(song))] Song(Song),
		#[json("boots-size")] Boots(usize)
	}

	#[derive(PartialEq, TryFromJson, DebugToJson)]
	enum Obj
	{	#[json(song)] Song(Song),
		#[json("boots-size")] Boots(usize)
	}

	// Test 1 (Subj)
	let input =
	r#"	[	{	"what": "sng",
				"song":
				{	"name": "Slow Dancing",
					"year": 1985,
					"special_artist":
					{	"first_name": "Ramsey",
						"last_name": "Lewis"
					}
				}
			}
		]
	"#;
	let expected_result = vec!
	[	Subj::Song
		(	Song
			{	name: "Slow Dancing".to_string(),
				year: 1985,
				artist: Person
				{	first_name: "Ramsey".to_string(),
					last_name: "Lewis".to_string(),
				}
			}
		)
	];
	let mut reader = Reader::new(input.bytes());
	let subj_0: Vec<Subj> = reader.read().unwrap();
	assert_eq!(subj_0, expected_result);

	// Test 2 (Subj)
	let input =
	r#"	[	[	{	"what": "Boots",
					"boots-size": 40
				},
				{	"what": "Boots",
					"boots-size": 41
				}
			]
		]
	"#;
	let expected_result = vec![vec![Subj::Boots(40), Subj::Boots(41)]];
	let mut reader = Reader::new(input.bytes());
	let subj_1: Vec<Vec<Subj>> = reader.read().unwrap();
	assert_eq!(subj_1, expected_result);

	// Test 3 (Obj)
	let input =
	r#"	[	{	"song":
				{	"name": "Slow Dancing",
					"year": 1985,
					"special_artist":
					{	"first_name": "Ramsey",
						"last_name": "Lewis"
					}
				}
			}
		]
	"#;
	let expected_result = vec!
	[	Obj::Song
		(	Song
			{	name: "Slow Dancing".to_string(),
				year: 1985,
				artist: Person
				{	first_name: "Ramsey".to_string(),
					last_name: "Lewis".to_string(),
				}
			}
		)
	];
	let mut reader = Reader::new(input.bytes());
	let obj_0: Vec<Obj> = reader.read().unwrap();
	assert_eq!(obj_0, expected_result);

	// Test 4 (Obj)
	let input =
	r#"	[	[	{	"boots-size": 40
				}
			],
			[	{	"boots-size": 41
				}
			]
		]
	"#;
	let expected_result = vec![vec![Obj::Boots(40)], vec![Obj::Boots(41)]];
	let mut reader = Reader::new(input.bytes());
	let obj_1: Vec<Vec<Obj>> = reader.read().unwrap();
	assert_eq!(obj_1, expected_result);

	// Serialize back

	let input = format!("{:?}", subj_0);
	let mut reader = Reader::new(input.bytes());
	let subj_0_back: Vec<Subj> = reader.read().unwrap();
	assert_eq!(subj_0, subj_0_back);

	let input = format!("{:?}", subj_1);
	let mut reader = Reader::new(input.bytes());
	let subj_1_back: Vec<Vec<Subj>> = reader.read().unwrap();
	assert_eq!(subj_1, subj_1_back);

	let input = format!("{:?}", obj_0);
	let mut reader = Reader::new(input.bytes());
	let obj_0_back: Vec<Obj> = reader.read().unwrap();
	assert_eq!(obj_0, obj_0_back);

	let input = format!("{:?}", obj_1);
	let mut reader = Reader::new(input.bytes());
	let obj_1_back: Vec<Vec<Obj>> = reader.read().unwrap();
	assert_eq!(obj_1, obj_1_back);
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
