use nop_json::{Reader, TryFromJson, DebugToJson, escape};

#[test]
fn test_number()
{	let mut reader = Reader::new(r#" 0 0.01 123 128 -128 -129 255 -1 -1e+2 1e2 123e-7 0.0123e-10 1e-1000000 1e10000000000 "hello" true "#.as_bytes());
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
	assert!(reader.read::<f64>().is_err()); // 1e10000000000
	assert_eq!(reader.read::<f64>().unwrap(), 0.0); // "hello"
	assert_eq!(reader.read::<i16>().unwrap(), 1); // true
}

#[test]
fn test_number_as_string()
{	let mut reader = Reader::new(r#" "0" "0.01" "123" "128" "-128" "-129" "255" "-1" "-1e+2" "1e2" "123e-7" "0.0123e-10" "1e-1000000" "1e10000000000" "hello" true "#.as_bytes());
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
	assert!(reader.read::<f64>().is_err()); // 1e10000000000
	assert_eq!(reader.read::<f64>().unwrap(), 0.0); // "hello"
	assert_eq!(reader.read::<i16>().unwrap(), 1); // true
}

#[test]
fn test_number_to_string()
{	let mut reader = Reader::new("123 12.3 0.123 0.0123 123e3 123e7 123e8 123e-1 123e-3 123e-13".as_bytes());
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
fn test_string()
{	let mut reader = Reader::new(r#" "abc" "abcdefghijklmnopqrstuvwxyz" "שלום" "\u0061\u0062\u0063 \u05E9\u05Dc\u05d5\u05dd" "\\ \n" "#.as_bytes());
	assert_eq!(reader.read::<String>().unwrap(), "abc");
	assert_eq!(reader.read::<String>().unwrap(), "abcdefghijklmnopqrstuvwxyz");
	assert_eq!(reader.read::<String>().unwrap(), "שלום");
	assert_eq!(reader.read::<String>().unwrap(), "abc שלום");
	assert_eq!(reader.read::<String>().unwrap(), "\\ \n");
}

#[test]
fn test_bytes()
{	let mut reader = Reader::new(r#" "abc" "abcdefghijklmnopqrstuvwxyz" "שלום" "\u0061\u0062\u0063 \u05E9\u05Dc\u05d5\u05dd" "\\ \n" "#.as_bytes());
	assert_eq!(reader.read_bytes().unwrap(), b"abc");
	assert_eq!(reader.read_bytes().unwrap(), b"abcdefghijklmnopqrstuvwxyz");
	assert_eq!(reader.read_bytes().unwrap(), "שלום".as_bytes());
	assert_eq!(reader.read_bytes().unwrap(), "abc שלום".as_bytes());
	assert_eq!(reader.read_bytes().unwrap(), b"\\ \n");
}

#[test]
fn test_char()
{	let mut reader = Reader::new(r#" "abc" "א" "\u05E9" "#.as_bytes());
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
	let mut reader = Reader::new(input.as_bytes());
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
	let mut reader = Reader::new(input.as_bytes());
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
	let mut reader = Reader::new(input.as_bytes());
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
	let mut reader = Reader::new(input.as_bytes());
	let obj_1: Vec<Vec<Obj>> = reader.read().unwrap();
	assert_eq!(obj_1, expected_result);

	// Serialize back

	let input = format!("{:?}", subj_0);
	let mut reader = Reader::new(input.as_bytes());
	let subj_0_back: Vec<Subj> = reader.read().unwrap();
	assert_eq!(subj_0, subj_0_back);

	let input = format!("{:?}", subj_1);
	let mut reader = Reader::new(input.as_bytes());
	let subj_1_back: Vec<Vec<Subj>> = reader.read().unwrap();
	assert_eq!(subj_1, subj_1_back);

	let input = format!("{:?}", obj_0);
	let mut reader = Reader::new(input.as_bytes());
	let obj_0_back: Vec<Obj> = reader.read().unwrap();
	assert_eq!(obj_0, obj_0_back);

	let input = format!("{:?}", obj_1);
	let mut reader = Reader::new(input.as_bytes());
	let obj_1_back: Vec<Vec<Obj>> = reader.read().unwrap();
	assert_eq!(obj_1, obj_1_back);
}

#[test]
fn test_escape()
{	assert_eq!(escape("abc"), "abc");
	assert_eq!(escape("a\\b\"c"), "a\\\\b\\\"c");
}
