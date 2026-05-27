//! The lower-level `Reader` API: reading whitespace-separated sequences, blobs,
//! piping, the manual `read_object`/`read_object_use_buffer`/`read_array` helpers,
//! error messages that carry the path into the document, and unwrapping the source.

use nop_json::{Reader, TryFromJson, ValidateJson, DebugToJson, escape_bytes};
use std::io::{self, Write};

#[test]
fn reads_whitespace_separated_sequence()
{	let mut reader = Reader::new(r#" true  100.5  "Hello"  [1, 2]  null "#.bytes());
	assert_eq!(reader.read::<bool>().unwrap(), true);
	assert_eq!(reader.read::<f64>().unwrap(), 100.5);
	assert_eq!(reader.read::<String>().unwrap(), "Hello");
	assert_eq!(reader.read::<Vec<i32>>().unwrap(), vec![1, 2]);
	assert_eq!(reader.read::<Option<i32>>().unwrap(), None);
}

#[test]
fn newlines_and_tabs_between_values()
{	let mut reader = Reader::new("1\n\t2\r\n  3".bytes());
	assert_eq!(reader.read::<i32>().unwrap(), 1);
	assert_eq!(reader.read::<i32>().unwrap(), 2);
	assert_eq!(reader.read::<i32>().unwrap(), 3);
}

#[test]
fn unwrap_returns_source_iterator()
{	let mut reader = Reader::new("1 2 3".bytes());
	let _: i32 = reader.read().unwrap();
	let rest: Vec<u8> = reader.unwrap().collect();
	// reading "1" also consumes the following byte as lookahead, so "2 3" remains
	assert_eq!(rest, b"2 3");
}

#[test]
fn read_blob_passes_through_high_bytes()
{	let mut reader = Reader::new(b" \"\x80\x81\" ".iter().copied());
	assert_eq!(reader.read_blob().unwrap(), b"\x80\x81");
}

#[test]
fn blob_round_trip_via_escape_bytes()
{	let data = b"\x00\x01\x80\xFF\"\\ ok";
	let mut json = Vec::new();
	json.push(b'"');
	json.write_all(escape_bytes(data).as_ref()).unwrap();
	json.push(b'"');

	let mut reader = Reader::new(json.into_iter());
	assert_eq!(reader.read_blob().unwrap(), data);
}

#[test]
fn pipe_blob_chunks_across_buffer_boundary()
{	const READER_BUFFER_SIZE: usize = 128;
	const N_CHARS: usize = 300;

	let mut data = Vec::new();
	let mut expected = Vec::new();
	data.push(b'"');
	for i in 0 .. N_CHARS
	{	let b = (i % 256) as u8;
		if b < 32
		{	for c in format!("\\u{b:04x}").chars() {data.push(c as u8)}
		}
		else if b == b'\\' || b == b'"'
		{	data.push(b'\\');
			data.push(b);
		}
		else
		{	data.push(b);
		}
		expected.push(b);
	}
	data.push(b'"');

	struct Sink {data: Vec<u8>, n_parts: usize}
	impl Write for Sink
	{	fn write(&mut self, buf: &[u8]) -> io::Result<usize>
		{	self.data.extend_from_slice(buf);
			self.n_parts += 1;
			Ok(buf.len())
		}
		fn flush(&mut self) -> io::Result<()> {Ok(())}
	}

	let mut sink = Sink {data: Vec::new(), n_parts: 0};
	let mut reader = Reader::new(data.into_iter());
	reader.pipe_blob(&mut sink).unwrap();

	assert_eq!(sink.data, expected);
	assert_eq!(sink.n_parts, (N_CHARS as f64 / READER_BUFFER_SIZE as f64).ceil() as usize);
}

#[test]
fn manual_read_object()
{	let mut reader = Reader::new(r#" {"x": 10, "y": "the y"} "#.bytes());
	let mut x: Option<i32> = None;
	let mut y = String::new();
	reader.read_object
	(	|reader, prop|
		{	match prop.as_ref()
			{	"x" => x = reader.read_prop("x")?,
				"y" => y = reader.read_prop("y")?,
				_ => return Err(reader.format_error_fmt(format_args!("Invalid property: {prop}")))
			}
			Ok(())
		}
	).unwrap();
	assert_eq!(x, Some(10));
	assert_eq!(y, "the y");
}

#[test]
fn manual_read_object_use_buffer()
{	let mut reader = Reader::new(r#" {"x": 10, "y": 20} "#.bytes());
	let mut x = 0;
	let mut y = 0;
	reader.read_object_use_buffer
	(	|reader|
		{	match reader.get_key()
			{	b"x" => x = reader.read_prop("x")?,
				b"y" => y = reader.read_prop("y")?,
				_ => return Err(reader.format_error("unexpected key")),
			}
			Ok(())
		}
	).unwrap();
	assert_eq!((x, y), (10, 20));
}

#[test]
fn manual_read_array()
{	let mut reader = Reader::new(r#" ["One", "Two", "Three"] "#.bytes());
	let mut value: Vec<String> = Vec::new();
	reader.read_array
	(	|reader|
		{	value.push(reader.read_index()?);
			Ok(())
		}
	).unwrap();
	assert_eq!(value, vec!["One".to_string(), "Two".to_string(), "Three".to_string()]);
}

#[test]
fn read_object_returns_false_for_null()
{	let mut reader = Reader::new("null".bytes());
	let present = reader.read_object(|_reader, _key| Ok(())).unwrap();
	assert_eq!(present, false);
}

#[test]
fn error_message_includes_array_index_path()
{	let mut reader = Reader::new("[1, 2, 999]".bytes());
	let err = reader.read::<Vec<u8>>().unwrap_err();
	assert!(err.to_string().contains("[2]"), "error was: {err}");
}

#[test]
fn error_message_includes_property_path()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Bag {items: Vec<u8>}

	let mut reader = Reader::new(r#" {"items": [1, 999]} "#.bytes());
	let err = reader.read::<Bag>().unwrap_err();
	let msg = err.to_string();
	assert!(msg.contains("items"), "error was: {msg}");
	assert!(msg.contains("[1]"), "error was: {msg}");
}

#[test]
fn errors_on_malformed_top_level()
{	assert!(Reader::new("}".bytes()).read::<i32>().is_err());
	assert!(Reader::new(",".bytes()).read::<i32>().is_err());
	assert!(Reader::new(":".bytes()).read::<i32>().is_err());
	assert!(Reader::new("nul".bytes()).read::<Option<i32>>().is_err());
}
