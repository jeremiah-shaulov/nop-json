//! String reading: escapes, \u BMP escapes (1/2/3-byte), UTF-16 surrogate pairs,
//! raw UTF-8 passthrough, read_char, read_bytes, and malformed-escape errors.
//!
//! JSON `\uXXXX` escapes are written as Rust `"\\uXXXX"` (the `\\` is one backslash byte).

use nop_json::Reader;

fn read_str(json: &str) -> std::io::Result<String>
{	Reader::new(json.bytes()).read()
}

#[test]
fn plain_and_long()
{	assert_eq!(read_str("\"abc\"").unwrap(), "abc");
	assert_eq!(read_str("\"\"").unwrap(), "");
	assert_eq!(read_str("\"abcdefghijklmnopqrstuvwxyz\"").unwrap(), "abcdefghijklmnopqrstuvwxyz");
}

#[test]
fn simple_escapes()
{	assert_eq!(read_str("\"a\\\"b\"").unwrap(), "a\"b");   // \"
	assert_eq!(read_str("\"a\\\\b\"").unwrap(), "a\\b");   // \\
	assert_eq!(read_str("\"a\\/b\"").unwrap(), "a/b");     // \/
	assert_eq!(read_str("\"\\n\"").unwrap(), "\n");
	assert_eq!(read_str("\"\\r\"").unwrap(), "\r");
	assert_eq!(read_str("\"\\t\"").unwrap(), "\t");
	assert_eq!(read_str("\"\\b\"").unwrap(), "\u{0008}");
	assert_eq!(read_str("\"\\f\"").unwrap(), "\u{000C}");
	assert_eq!(read_str("\"\\\\ \\n\"").unwrap(), "\\ \n");
}

#[test]
fn u_escape_1byte()
{	assert_eq!(read_str("\"\\u0041\\u0042\\u0043\"").unwrap(), "ABC");
	assert_eq!(read_str("\"\\u0000\"").unwrap(), "\u{0000}");
}

#[test]
fn u_escape_2byte()
{	// Hebrew U+05D0..: each is 2-byte UTF-8
	assert_eq!(read_str("\"\\u05E9\\u05DC\\u05D5\\u05DD\"").unwrap(), "\u{05E9}\u{05DC}\u{05D5}\u{05DD}");
}

#[test]
fn u_escape_3byte()
{	assert_eq!(read_str("\"\\u20AC\"").unwrap(), "\u{20AC}");          // EURO SIGN
	assert_eq!(read_str("\"\\u4E2D\\u6587\"").unwrap(), "\u{4E2D}\u{6587}"); // CJK
	assert_eq!(read_str("\"a\\u20ACb\"").unwrap(), "a\u{20AC}b");
}

#[test]
fn u_escape_surrogate_pair()
{	assert_eq!(read_str("\"\\uD83D\\uDE00\"").unwrap(), "\u{1F600}");        // grinning face
	assert_eq!(read_str("\"\\uD83D\\uDE00\\uD83D\\uDE01\"").unwrap(), "\u{1F600}\u{1F601}");
	assert_eq!(read_str("\"x\\uD834\\uDD1Ey\"").unwrap(), "x\u{1D11E}y");    // musical symbol G clef
}

#[test]
fn lowercase_and_uppercase_hex()
{	assert_eq!(read_str("\"\\u00e9\"").unwrap(), "\u{00E9}");
	assert_eq!(read_str("\"\\u00E9\"").unwrap(), "\u{00E9}");
}

#[test]
fn raw_utf8_passthrough()
{	// feed the actual multi-byte UTF-8 bytes (no escapes)
	let input = format!("\"{}{}{}\"", '\u{05E9}', '\u{4E2D}', '\u{1F600}');
	assert_eq!(read_str(&input).unwrap(), "\u{05E9}\u{4E2D}\u{1F600}");
}

#[test]
fn string_from_scalars()
{	assert_eq!(read_str("null").unwrap(), "null");
	assert_eq!(read_str("true").unwrap(), "true");
	assert_eq!(read_str("false").unwrap(), "false");
	assert_eq!(read_str("123").unwrap(), "123");
	assert_eq!(read_str("12.3").unwrap(), "12.3");
}

#[test]
fn read_char_various_widths()
{	assert_eq!(Reader::new("\"a\"".bytes()).read::<char>().unwrap(), 'a');
	assert_eq!(Reader::new("\"\\u05E9\"".bytes()).read::<char>().unwrap(), '\u{05E9}'); // 2-byte
	assert_eq!(Reader::new("\"\\u20AC\"".bytes()).read::<char>().unwrap(), '\u{20AC}'); // 3-byte
	assert_eq!(Reader::new("\"\\uD83D\\uDE00\"".bytes()).read::<char>().unwrap(), '\u{1F600}'); // 4-byte
	// first char of a longer string
	assert_eq!(Reader::new("\"abc\"".bytes()).read::<char>().unwrap(), 'a');
}

#[test]
fn read_char_empty_is_error()
{	assert!(Reader::new("\"\"".bytes()).read::<char>().is_err());
}

#[test]
fn read_bytes_returns_raw()
{	let mut reader = Reader::new("\"abc\"".bytes());
	assert_eq!(reader.read_bytes().unwrap(), b"abc");

	let mut reader = Reader::new("\"\\u20AC\"".bytes());
	assert_eq!(reader.read_bytes().unwrap(), "\u{20AC}".as_bytes()); // 3 bytes

	let mut reader = Reader::new("123".bytes());
	assert_eq!(reader.read_bytes().unwrap(), b"123");

	let mut reader = Reader::new("null".bytes());
	assert_eq!(reader.read_bytes().unwrap(), b"null");
}

#[test]
fn malformed_escapes_error()
{	assert!(read_str("\"\\uZZZZ\"").is_err());            // bad hex
	assert!(read_str("\"\\uD83D\"").is_err());            // high surrogate, no low surrogate
	assert!(read_str("\"\\uD83Dx\"").is_err());           // high surrogate not followed by \u
	assert!(read_str("\"\\uDE00\"").is_err());            // lone low surrogate
	assert!(read_str("\"unterminated").is_err());         // no closing quote
}
