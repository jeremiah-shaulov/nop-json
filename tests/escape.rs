//! The `escape` / `escape_bytes` helpers.

use nop_json::{escape, escape_bytes};
use std::borrow::Cow;

#[test]
fn no_special_chars_borrows()
{	match escape("plain text 123")
	{	Cow::Borrowed(s) => assert_eq!(s, "plain text 123"),
		Cow::Owned(_) => panic!("should not allocate when nothing needs escaping"),
	}
}

#[test]
fn escapes_quote_and_backslash()
{	assert_eq!(escape("a\\b\"c"), "a\\\\b\\\"c");
	assert_eq!(escape("\""), "\\\"");
	assert_eq!(escape("\\"), "\\\\");
}

#[test]
fn escapes_named_control_chars()
{	assert_eq!(escape("\n"), "\\n");
	assert_eq!(escape("\r"), "\\r");
	assert_eq!(escape("\t"), "\\t");
	assert_eq!(escape("\u{0008}"), "\\b");
	assert_eq!(escape("\u{000C}"), "\\f");
}

#[test]
fn escapes_other_control_chars_as_u00xx_uppercase()
{	assert_eq!(escape("\u{0000}"), "\\u0000");
	assert_eq!(escape("\u{0001}"), "\\u0001");
	assert_eq!(escape("\u{001F}"), "\\u001F");  // uppercase hex digits
}

#[test]
fn does_not_escape_high_unicode()
{	// non-control, non-quote/backslash characters pass through untouched
	assert_eq!(escape("\u{05E9}\u{20AC}\u{1F600}"), "\u{05E9}\u{20AC}\u{1F600}");
}

#[test]
fn mixed_content()
{	assert_eq!(escape("line1\nline2\t\"quoted\""), "line1\\nline2\\t\\\"quoted\\\"");
}

#[test]
fn escape_bytes_basic_and_passthrough()
{	assert_eq!(escape_bytes(b"abc").as_ref(), b"abc");
	assert_eq!(escape_bytes(b"a\\b\"c\n").as_ref(), b"a\\\\b\\\"c\\n");
	// bytes 0x80-0xFF are passed through unchanged (used for binary blobs)
	assert_eq!(escape_bytes(b"\x80\xFF").as_ref(), b"\x80\xFF");
	// control bytes still escaped
	assert_eq!(escape_bytes(b"\x00\x1F").as_ref(), b"\\u0000\\u001F");
}

#[test]
fn escape_bytes_borrows_when_clean()
{	match escape_bytes(b"\x80\x81clean")
	{	Cow::Borrowed(b) => assert_eq!(b, b"\x80\x81clean"),
		Cow::Owned(_) => panic!("high bytes alone should not trigger allocation"),
	}
}
