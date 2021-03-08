use std::borrow::Cow;

const HEX_DIGITS: [u8; 16] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F'];

/// Adds slashes before `"` and `\` characters, converts `\t`, `\r`, `\n`, `\b`, `\f` characters as needed,
/// and encodes characters with codes less than space (32) with `\u00XX`.
/// If input string doesn't contain something that JSON standard wants us to escape, it just returns the input string
/// without memory allocation.
///
/// # Examples
///
/// ```
/// let orig = "Some \"quote\" and some \\.";
/// let json_encoded = format!("{{\"value\": \"{}\"}}", nop_json::escape(orig));
/// assert_eq!(json_encoded, "{\"value\": \"Some \\\"quote\\\" and some \\\\.\"}");
/// ```
pub fn escape(s: &str) -> Cow<str>
{	let bytes = s.as_bytes();
	if let Some(pos) = bytes.iter().position(|c| match *c {b'"' | b'\\' | 0..=31 => true, _ => false})
	{	Cow::Owned(String::from_utf8(do_escape_bytes(bytes, pos)).unwrap())
	}
	else
	{	Cow::Borrowed(s)
	}
}

/// Like [escape](fn.escape.html), but for `&[u8]`.
pub fn escape_bytes(bytes: &[u8]) -> Cow<[u8]>
{	if let Some(pos) = bytes.iter().position(|c| match *c {b'"' | b'\\' | 0..=31 => true, _ => false})
	{	Cow::Owned(do_escape_bytes(bytes, pos))
	}
	else
	{	Cow::Borrowed(bytes)
	}
}

fn do_escape_bytes(bytes: &[u8], mut pos: usize) -> Vec<u8>
{	let mut buffer = Vec::with_capacity(bytes.len() + 8);
	let mut from = 0;
	loop
	{	buffer.extend_from_slice(&bytes[from .. pos]);
		let c = bytes[pos];
		if c >= 32
		{	buffer.push(b'\\');
			buffer.push(c);
		}
		else
		{	match c
			{	9 =>
				{	buffer.push(b'\\');
					buffer.push(b't');
				}
				13 =>
				{	buffer.push(b'\\');
					buffer.push(b'r');
				}
				10 =>
				{	buffer.push(b'\\');
					buffer.push(b'n');
				}
				8 =>
				{	buffer.push(b'\\');
					buffer.push(b'b');
				}
				12 =>
				{	buffer.push(b'\\');
					buffer.push(b'f');
				}
				_ =>
				{	buffer.push(b'\\');
					buffer.push(b'u');
					buffer.push(b'0');
					buffer.push(b'0');
					buffer.push(HEX_DIGITS[(c >> 4) as usize]);
					buffer.push(HEX_DIGITS[(c & 0xF) as usize]);
				}
			}
		}
		from = pos + 1;
		if let Some(new_pos) = &bytes[from ..].iter().position(|c| match *c {b'"' | b'\\' | 0..=31 => true, _ => false})
		{	pos = from + *new_pos;
		}
		else
		{	buffer.extend_from_slice(&bytes[from .. ]);
			break;
		}
	}
	buffer
}
