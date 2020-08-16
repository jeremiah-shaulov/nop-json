pub use nop_json_derive::*;
use crate::value::Value;

use std::{io, io::Write, char, fmt, borrow::Cow};
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, LinkedList, VecDeque};
use std::convert::TryInto;
use numtoa::NumToA;

const READER_BUFFER_SIZE: usize = 128;
const FORMAT_NUM_WIDTH: usize = 10;
const FORMAT_NUM_WIDTH_Z: [u8; FORMAT_NUM_WIDTH] = [b'0'; FORMAT_NUM_WIDTH];
const FORMAT_NUM_WIDTH_0Z: &[u8] = b"0.0000000000";
const HEX_DIGITS: [u8; 16] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F'];

macro_rules! read_int
{	($self:expr, $T:ty, $is_unsigned:expr) =>
	{	{	let mut is_in_string = false;
			let mut c = $self.lookahead;
			loop
			{	match c
				{	b' ' | b'\t' | b'\r' | b'\n' =>
					{	c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
					}
					b'n' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok(0);
						}
						if let Some(b'u') = $self.iter.next()
						{	if let Some(b'l') = $self.iter.next()
							{	if let Some(b'l') = $self.iter.next()
								{	if let Some(c) = $self.iter.next()
									{	if !c.is_ascii_alphanumeric() && c!=b'_'
										{	$self.lookahead = c;
											return Ok(0 as $T);
										}
									}
									else
									{	$self.lookahead = b' ';
										return Ok(0 as $T);
									}
								}
							}
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: unexpected identifier"));
					}
					b'f' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok(0);
						}
						if let Some(b'a') = $self.iter.next()
						{	if let Some(b'l') = $self.iter.next()
							{	if let Some(b's') = $self.iter.next()
								{	if let Some(b'e') = $self.iter.next()
									{	if let Some(c) = $self.iter.next()
										{	if !c.is_ascii_alphanumeric() && c!=b'_'
											{	$self.lookahead = c;
												return Ok(0 as $T);
											}
										}
										else
										{	$self.lookahead = b' ';
											return Ok(0 as $T);
										}
									}
								}
							}
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: unexpected identifier"));
					}
					b't' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok(0);
						}
						if let Some(b'r') = $self.iter.next()
						{	if let Some(b'u') = $self.iter.next()
							{	if let Some(b'e') = $self.iter.next()
								{	if let Some(c) = $self.iter.next()
									{	if !c.is_ascii_alphanumeric() && c!=b'_'
										{	$self.lookahead = c;
											return Ok(1 as $T);
										}
									}
									else
									{	$self.lookahead = b' ';
										return Ok(1 as $T);
									}
								}
							}
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: unexpected identifier"));
					}
					b'0'..=b'9' | b'-' | b'.' =>
					{	let mut is_negative = false;
						let mut exponent = 0i32;
						let mut is_after_dot = false;
						let mut result = 0 as $T;
						let mut ten = 10 as $T;
						let mut is_error = false;
						if c == b'-'
						{	is_negative = true;
							c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
						}
						loop
						{	match c
							{	b'0' =>
								{	ten = ten.checked_mul(10).unwrap_or_else(|| {if !is_after_dot {is_error = true}; 0});
								}
								b'1'..= b'9' =>
								{	if !is_after_dot
									{	result = result.checked_mul(ten).unwrap_or_else(|| {is_error = true; 0});
										result = result.checked_add(if $is_unsigned {(c - b'0') as $T} else {(b'0' as i8 - c as i8) as $T}).unwrap_or_else(|| {is_error = true; 0}); // if signed, make negative number (because wider range), and then negate (if not is_negative)
										ten = 10 as $T;
									}
								}
								b'.' => {is_after_dot = true}
								b'e' | b'E' =>
								{	c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
									let mut n_is_negative = false;
									match c
									{	b'+' => {c = b'0'}
										b'-' => {c = b'0'; n_is_negative = true}
										b'0' ..= b'9' => {}
										_ =>
										{	$self.lookahead = c;
											if is_in_string
											{	if c != b'"' {$self.skip_string()?} else {$self.lookahead = b' '};
												return Ok(0);
											}
											return Err($self.format_error("Invalid JSON input: invalid number format"));
										}
									};
									let mut n: i32 = 0;
									loop
									{	match c
										{	b'0' =>
											{	n = n.checked_mul(10).unwrap_or_else(|| {is_error = true; 0});
											}
											b'1'..=b'9' =>
											{	n = n.checked_mul(10).and_then(|n| n.checked_add((c-b'0') as i32)).unwrap_or_else(|| {is_error = true; 0});
											}
											_ =>
											{	$self.lookahead = c;
												break;
											}
										}
										if let Some(new_c) = $self.iter.next()
										{	c = new_c;
										}
										else
										{	$self.lookahead = b' ';
											if is_in_string
											{	return Err($self.format_error("Invalid JSON input: unexpected end of input"));
											}
											break;
										}
									}
									exponent = if n_is_negative {-n} else {n};
									break;
								}
								_ =>
								{	$self.lookahead = c;
									break;
								}
							}
							if let Some(new_c) = $self.iter.next()
							{	c = new_c;
							}
							else
							{	$self.lookahead = b' ';
								if is_in_string
								{	return Err($self.format_error("Invalid JSON input: unexpected end of input"));
								}
								break;
							}
						}
						if !is_after_dot && ten>(10 as $T)
						{	result = result.checked_mul(ten / (10 as $T)).unwrap_or_else(|| {is_error = true; 0});
						}
						if exponent != 0
						{	result = result.checked_mul((10 as $T).checked_pow(exponent as u32).unwrap_or_else(|| {is_error = true; 0})).unwrap_or_else(|| {is_error = true; 0});
						}
						if $is_unsigned
						{	if is_negative
							{	is_error = true;
							}
						}
						else if !is_negative // i built negative number (see above), so make it nonnegative
						{	result = result.checked_neg().unwrap_or_else(|| {is_error = true; 0});
						}
						if is_error
						{	if is_in_string
							{	if $self.lookahead!=b'"' {$self.skip_string()?} else {$self.lookahead = b' '};
							}
							return Err($self.number_error());
						}
						if is_in_string
						{	let mut c = $self.lookahead;
							$self.lookahead = b' ';
							while c.is_ascii_whitespace()
							{	match $self.iter.next()
								{	Some(new_c) => c = new_c,
									None => return Err($self.format_error("Invalid JSON: unexpected end of input"))
								}
							}
							if c != b'"'
							{	$self.skip_string()?;
								return Ok(0);
							}
						}
						return Ok(result);
					}
					b'"' =>
					{	if !is_in_string
						{	is_in_string = true;
							c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
						}
						else
						{	$self.lookahead = b' ';
							return Ok(0);
						}
					}
					b'[' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok(0);
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: value must be number, not array"));
					}
					b'{' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok(0);
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: value must be number, not object"));
					}
					_ =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok(0);
						}
						$self.lookahead = b' ';
						return Err($self.format_error_fmt(format_args!("Invalid JSON input: unexpected '{}'", String::from_utf8_lossy(&[c]))));
					}
				}
			}
		}
	}
}

macro_rules! read_float
{	($self:expr, $T:ty, $nan:expr, $infinity:expr, $neg_infinity:expr) =>
	{	{	let mut is_in_string = false;
			let mut c = $self.lookahead;
			loop
			{	match c
				{	b' ' | b'\t' | b'\r' | b'\n' =>
					{	c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
					}
					b'n' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok($nan);
						}
						if let Some(b'u') = $self.iter.next()
						{	if let Some(b'l') = $self.iter.next()
							{	if let Some(b'l') = $self.iter.next()
								{	if let Some(c) = $self.iter.next()
									{	if !c.is_ascii_alphanumeric() && c!=b'_'
										{	$self.lookahead = c;
											return Ok(0 as $T);
										}
									}
									else
									{	$self.lookahead = b' ';
										return Ok(0 as $T);
									}
								}
							}
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: unexpected identifier"));
					}
					b'f' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok($nan);
						}
						if let Some(b'a') = $self.iter.next()
						{	if let Some(b'l') = $self.iter.next()
							{	if let Some(b's') = $self.iter.next()
								{	if let Some(b'e') = $self.iter.next()
									{	if let Some(c) = $self.iter.next()
										{	if !c.is_ascii_alphanumeric() && c!=b'_'
											{	$self.lookahead = c;
												return Ok(0 as $T);
											}
										}
										else
										{	$self.lookahead = b' ';
											return Ok(0 as $T);
										}
									}
								}
							}
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: unexpected identifier"));
					}
					b't' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok($nan);
						}
						if let Some(b'r') = $self.iter.next()
						{	if let Some(b'u') = $self.iter.next()
							{	if let Some(b'e') = $self.iter.next()
								{	if let Some(c) = $self.iter.next()
									{	if !c.is_ascii_alphanumeric() && c!=b'_'
										{	$self.lookahead = c;
											return Ok(1 as $T);
										}
									}
									else
									{	$self.lookahead = b' ';
										return Ok(1 as $T);
									}
								}
							}
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: unexpected identifier"));
					}
					b'0'..=b'9' | b'-' | b'.' =>
					{	let mut is_negative = false;
						let mut exponent = 0i32;
						let mut is_after_dot = 0;
						let mut result = 0 as $T;
						let mut ten = 10 as $T;
						let mut is_error = false;
						if c == b'-'
						{	is_negative = true;
							c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
							if is_in_string && c==b'I' // -Infinity?
							{	$self.read_string_contents_as_bytes()?;
								if $self.buffer_len >= 7 && &$self.buffer[.. 7] == b"nfinity"
								{	if $self.buffer_len > 7 || $self.buffer[7 .. $self.buffer_len].iter().position(|c| !c.is_ascii_whitespace()).is_none()
									{	return Ok($neg_infinity);
									}
								}
								return Ok($nan);
							}
						}
						loop
						{	match c
							{	b'0' =>
								{	exponent += is_after_dot;
									ten *= 10 as $T;
								}
								b'1'..= b'9' =>
								{	exponent += is_after_dot;
									result *= ten;
									result += (c - b'0') as $T;
									ten = 10 as $T;
								}
								b'.' => {is_after_dot = -1}
								b'e' | b'E' =>
								{	c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
									let mut n_is_negative = false;
									match c
									{	b'+' => {c = b'0'}
										b'-' => {c = b'0'; n_is_negative = true}
										b'0' ..= b'9' => {}
										_ =>
										{	$self.lookahead = c;
											if is_in_string
											{	if c != b'"' {$self.skip_string()?} else {$self.lookahead = b' '};
												return Ok($nan);
											}
											return Err($self.format_error("Invalid JSON input: invalid number format"));
										}
									};
									let mut n: i32 = 0;
									loop
									{	match c
										{	b'0' =>
											{	n = n.checked_mul(10).unwrap_or_else(|| {is_error = true; 0});
											}
											b'1'..=b'9' =>
											{	n = n.checked_mul(10).and_then(|n| n.checked_add((c-b'0') as i32)).unwrap_or_else(|| {is_error = true; 0});
											}
											_ =>
											{	$self.lookahead = c;
												break;
											}
										}
										if let Some(new_c) = $self.iter.next()
										{	c = new_c;
										}
										else
										{	$self.lookahead = b' ';
											if is_in_string
											{	return Err($self.format_error("Invalid JSON input: unexpected end of input"));
											}
											break;
										}
									}
									match exponent.checked_add(if n_is_negative {-n} else {n})
									{	Some(new_exponent) => exponent = new_exponent,
										None => is_error = true
									}
									break;
								}
								_ =>
								{	$self.lookahead = c;
									break;
								}
							}
							if let Some(new_c) = $self.iter.next()
							{	c = new_c;
							}
							else
							{	$self.lookahead = b' ';
								if is_in_string
								{	return Err($self.format_error("Invalid JSON input: unexpected end of input"));
								}
								break;
							}
						}
						if is_error
						{	if is_in_string
							{	if $self.lookahead!=b'"' {$self.skip_string()?} else {$self.lookahead = b' '};
							}
							return Ok($nan);
						}
						if is_after_dot==0 && ten>10.0
						{	result *= ten / 10.0;
						}
						if exponent != 0
						{	result *= (10 as $T).powi(exponent);
						}
						if is_negative
						{	result = -result;
						}
						if is_in_string
						{	let mut c = $self.lookahead;
							$self.lookahead = b' ';
							while c.is_ascii_whitespace()
							{	match $self.iter.next()
								{	Some(new_c) => c = new_c,
									None => return Err($self.format_error("Invalid JSON: unexpected end of input"))
								}
							}
							if c != b'"'
							{	$self.skip_string()?;
								return Ok($nan);
							}
						}
						return Ok(result);
					}
					b'"' =>
					{	if !is_in_string
						{	is_in_string = true;
							c = $self.iter.next().ok_or_else(|| $self.format_error("Invalid JSON: unexpected end of input"))?;
						}
						else
						{	$self.lookahead = b' ';
							return Ok($nan);
						}
					}
					b'[' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok($nan);
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: value must be number, not array"));
					}
					b'{' =>
					{	if is_in_string
						{	$self.skip_string()?;
							return Ok($nan);
						}
						$self.lookahead = b' ';
						return Err($self.format_error("Invalid JSON input: value must be number, not object"));
					}
					_ =>
					{	if is_in_string
						{	if c == b'I'
							{	$self.read_string_contents_as_bytes()?;
								if $self.buffer_len >= 7 && &$self.buffer[.. 7] == b"nfinity"
								{	if $self.buffer_len > 7 || $self.buffer[7 .. $self.buffer_len].iter().position(|c| !c.is_ascii_whitespace()).is_none()
									{	return Ok($infinity);
									}
								}
							}
							else
							{	$self.skip_string()?;
							}
							return Ok($nan);
						}
						$self.lookahead = b' ';
						return Err($self.format_error_fmt(format_args!("Invalid JSON input: unexpected '{}'", String::from_utf8_lossy(&[c]))));
					}
				}
			}
		}
	}
}

// pub OptionalDefault (i only wanted to export OrDefault, but compiler wants me to export this as well)

pub trait OptionalDefault: Sized
{	fn optional_default() -> Option<Self>;
}
impl<T> OptionalDefault for T
{	default fn optional_default() -> Option<Self>
	{	None
	}
}
impl<T> OptionalDefault for T where T: Default
{	fn optional_default() -> Option<Self>
	{	Some(Self::default())
	}
}


/// This is like std::default::Default, but optional. I auto-implement this trait for Option<T> for all types.
/// Types that implement std::default::Default return Some(default), and others return None.
/// I need this to provide default value for omitted fields during JSON deserialization.
///
/// # Examples
///
/// ```
/// use nop_json::OrDefault;
///
/// #[derive(PartialEq, Debug)]
/// struct NoFallback {x: i32, y: i32}
///
/// #[derive(PartialEq, Debug, Default)]
///	struct Fallback {x: i32, y: i32}
///
/// let no: Option<NoFallback> = None;
/// let yes: Option<Fallback> = None;
/// assert_eq!(no.or_default(), None);
/// assert_eq!(yes.or_default(), Some(Fallback {x: 0, y: 0}));
/// ```
pub trait OrDefault
{	fn or_default(self) -> Self;
}
impl<T> OrDefault for Option<T> where T: OptionalDefault
{	fn or_default(self) -> Self
	{	self.or_else(|| T::optional_default())
	}
}


/// I auto-implement this trait for all types. During deserialization process, you may want to complain
/// on invalid fields combination. I call ok_from_json() right after i deserialized some object with `#[derive(TryFromJson)]`.
/// You can implement this function on your structs/enums.
///
/// # Examples
///
/// ```
/// use nop_json::{Reader, TryFromJson};
/// use std::io;
///
/// #[derive(TryFromJson, Debug)]
/// struct FromTo {from: i32, to: i32}
///
/// impl FromTo
/// {	fn ok_from_json(self) -> Result<Self, String>
/// 	{	if self.from <= self.to
/// 		{	Ok(self)
/// 		}
/// 		else
/// 		{	Err("to must be after from".to_string())
/// 		}
/// 	}
/// }
///
/// let mut reader = Reader::new(r#" {"from": 0, "to": 10}   {"from": 3, "to": -1} "#.bytes());
/// let obj_0: io::Result<FromTo> = reader.read();
/// let obj_1: io::Result<FromTo> = reader.read();
/// assert!(obj_0.is_ok());
/// assert!(obj_1.is_err());
/// ```
pub trait OkFromJson: Sized
{	fn ok_from_json(self) -> Result<Self, String>;
}
impl<T> OkFromJson for T
{	default fn ok_from_json(self) -> Result<Self, String>
	{	Ok(self)
	}
}


/// Implementing this trait makes possible to any type (except unions) to be JSON deserializable. The common technique
/// to implement this trait is automatically through `#[derive(TryFromJson)]`.
///
/// # Examples
///
/// ```
/// use nop_json::{Reader, TryFromJson, DebugToJson};
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// struct Point {x: i32, y: i32}
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// #[json(type)]
/// enum Geometry
/// {	#[json(point)] Point(Point),
/// 	#[json(cx, cy, r)] Circle(i32, i32, i32),
/// 	Nothing,
/// }
///
/// let mut reader = Reader::new(r#" {"type": "Point", "point": {"x": 0, "y": 0}} "#.bytes());
/// let obj: Geometry = reader.read().unwrap();
/// assert_eq!(obj, Geometry::Point(Point {x: 0, y: 0}));
/// ```
/// Here we deserialize a struct, and an enum. Struct `Point {x: 0, y: 0}` will be written as `{"x": 0, "y": 0}`.
///
/// We can use different names for "x" and "y". Every struct field can be optionally annotated with `#[json(field_name)]` attribute,
/// or `#[json("field_name")]`.
///
/// For enums we need to give names to each field, plus to "variant" field. The name of the "variant" field is specified at enum level.
/// In the example above, it's "type" (`#[json(type)]`). So `Geometry::Circle(0, 0, 1)` will be written as
/// `{"type": "Circle", "cx": 0, "cy": 0, "r": 1}`.
///
/// Variant name is printed as it's called in enum ("Point", "Circle", "Nothing"). We can rename them if specify `#[json(variant_name(field_name_0, field_name_1, ...))]`.
/// ```
/// use nop_json::{Reader, TryFromJson, DebugToJson};
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// struct Point {x: i32, y: i32}
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// #[json(var)]
/// enum Geometry
/// {	#[json(pnt(point))] Point(Point),
/// 	#[json(cir(cx, cy, r))] Circle(i32, i32, i32),
/// 	Nothing,
/// }
///
/// let mut reader = Reader::new(r#" {"var": "pnt", "point": {"x": 0, "y": 0}} "#.bytes());
/// let obj: Geometry = reader.read().unwrap();
/// assert_eq!(obj, Geometry::Point(Point {x: 0, y: 0}));
/// ```
/// There's also another option: to choose variant according to content. To do so, we ommit `#[json(...)]` at enum level.
/// This is only possible if variants have non-overlapping members.
/// ```
/// use nop_json::{Reader, TryFromJson, DebugToJson};
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// struct Point {x: i32, y: i32}
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// enum Geometry
/// {	#[json(point)] Point(Point),
/// 	#[json(cx, cy, r)] Circle(i32, i32, i32),
/// 	Nothing,
/// }
///
/// let mut reader = Reader::new(r#" {"point": {"x": 0, "y": 0}} "#.bytes());
/// let obj: Geometry = reader.read().unwrap();
/// assert_eq!(obj, Geometry::Point(Point {x: 0, y: 0}));
/// ```
///
/// To exclude a field from deserialization, and use default value for it, specify empty name (`#[json("")]`).
///
/// ```
/// use nop_json::{Reader, TryFromJson, DebugToJson};
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// struct Point {x: i32, y: i32, #[json("")] comments: String}
///
/// #[derive(TryFromJson, DebugToJson, PartialEq)]
/// enum Geometry
/// {	#[json(point)] Point(Point),
/// 	#[json(cx, cy, r)] Circle(i32, i32, i32),
/// 	Nothing,
/// }
///
/// let mut reader = Reader::new(r#" {"point": {"x": 0, "y": 0, "comments": "hello"}} "#.bytes());
/// let obj_0: Geometry = reader.read().unwrap();
/// assert_eq!(obj_0, Geometry::Point(Point {x: 0, y: 0, comments: String::new()}));
///
/// let mut reader = Reader::new(r#" {"point": {"x": 0, "y": 0}} "#.bytes());
/// let obj_0: Geometry = reader.read().unwrap();
/// assert_eq!(obj_0, Geometry::Point(Point {x: 0, y: 0, comments: String::new()}));
/// ```
/// It's possible to validate object right after deserialization. To do so implement `ok_from_json()` function.
/// ```
/// use nop_json::{Reader, TryFromJson};
///
/// #[derive(TryFromJson, Debug)]
/// struct FromTo {from: i32, to: i32}
///
/// impl FromTo
/// {	fn ok_from_json(self) -> Result<Self, String>
/// 	{	if self.from <= self.to
/// 		{	Ok(self)
/// 		}
/// 		else
/// 		{	Err("to must be after from".to_string())
/// 		}
/// 	}
/// }
/// ```
/// Automatic implementation through `#[derive(TryFromJson)]` has 1 limitation: object key string must be not longer
/// than 128 bytes, or it will be cut.
///
/// Sometimes that can be different reasons to implement `TryFromJson` manually.
/// Let's see how the automatic implementation looks like.
/// ```
/// struct Point {x: i32, y: i32}
///
/// impl nop_json::TryFromJson for Point
/// {	fn try_from_json<T>(reader: &mut nop_json::Reader<T>) -> std::io::Result<Self> where T: Iterator<Item=u8>
/// 	{	use nop_json::OrDefault;
/// 		use nop_json::OkFromJson;
///
/// 		let mut x = None;
/// 		let mut y = None;
///
/// 		reader.read_object_use_buffer
/// 		(	|reader|
/// 			{	match reader.get_key()
/// 				{	b"x" => x = reader.read_prop("x")?,
/// 					b"y" => y = reader.read_prop("y")?,
/// 					_ => return Err(reader.format_error_fmt(format_args!("Invalid property: {}", String::from_utf8_lossy(reader.get_key()))))
/// 				}
/// 				Ok(())
/// 			}
/// 		)?;
///
/// 		let result = Self
/// 		{	x: x.or_default().ok_or_else(|| reader.format_error("Member \"x\" doesn't have default value. To make optional #[derive(Default)]"))?,
/// 			y: y.or_default().ok_or_else(|| reader.format_error("Member \"y\" doesn't have default value. To make optional #[derive(Default)]"))?,
/// 		};
/// 		result.ok_from_json().map_err(|msg| reader.format_error(&msg))
/// 	}
/// }
/// ```
/// This implementation uses `reader.read_object_use_buffer()` which reads object keys to internal buffer which is 128 bytes, without memory allocation.
/// You can use `reader.read_object()` instead. Also you can do different things in this implementation function.
pub trait TryFromJson: Sized
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>;
}

impl TryFromJson for ()
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_and_discard()}
}

impl TryFromJson for isize
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_isize()}
}

impl TryFromJson for i128
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_i128()}
}

impl TryFromJson for i64
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_i64()}
}

impl TryFromJson for i32
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_i32()}
}

impl TryFromJson for i16
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_i16()}
}

impl TryFromJson for i8
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_i8()}
}

impl TryFromJson for usize
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_usize()}
}

impl TryFromJson for u128
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_u128()}
}

impl TryFromJson for u64
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_u64()}
}

impl TryFromJson for u32
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_u32()}
}

impl TryFromJson for u16
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_u16()}
}

impl TryFromJson for u8
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_u8()}
}

impl TryFromJson for f64
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_f64()}
}

impl TryFromJson for f32
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_f32()}
}

impl TryFromJson for bool
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_bool()}
}

impl TryFromJson for char
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_char()}
}

impl TryFromJson for String
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_string()}
}

impl TryFromJson for Value
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8> {reader.read_value()}
}

impl<U> TryFromJson for Box<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	Ok(Box::new(U::try_from_json(reader)?))
	}
}

impl<U> TryFromJson for std::sync::RwLock<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	Ok(std::sync::RwLock::new(U::try_from_json(reader)?))
	}
}

impl<U> TryFromJson for std::sync::Mutex<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	Ok(std::sync::Mutex::new(U::try_from_json(reader)?))
	}
}

impl<U> TryFromJson for std::rc::Rc<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	Ok(std::rc::Rc::new(U::try_from_json(reader)?))
	}
}

impl<U> TryFromJson for std::sync::Arc<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	Ok(std::sync::Arc::new(U::try_from_json(reader)?))
	}
}

impl<U> TryFromJson for Option<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	if reader.get_next_char() != b'n'
		{	Ok(Some(U::try_from_json(reader)?))
		}
		else // null or invalid
		{	reader.read_and_discard()?; // Err if invalid
			Ok(None)
		}
	}
}

impl<U> TryFromJson for Vec<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let mut result = Vec::new();
		reader.read_array
		(	|reader|
			{	result.push(reader.read_index()?);
				Ok(())
			}
		)?;
		Ok(result)
	}
}

impl<U> TryFromJson for HashSet<U> where U: Eq + std::hash::Hash + TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let mut result = HashSet::new();
		reader.read_array
		(	|reader|
			{	result.insert(reader.read_index()?);
				Ok(())
			}
		)?;
		Ok(result)
	}
}

impl<U> TryFromJson for LinkedList<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let mut result = LinkedList::new();
		reader.read_array
		(	|reader|
			{	result.push_back(reader.read_index()?);
				Ok(())
			}
		)?;
		Ok(result)
	}
}

impl<U> TryFromJson for VecDeque<U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let mut result = VecDeque::new();
		reader.read_array
		(	|reader|
			{	result.push_back(reader.read_index()?);
				Ok(())
			}
		)?;
		Ok(result)
	}
}

impl<U> TryFromJson for BTreeSet<U> where U: std::cmp::Ord + TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let mut result = BTreeSet::new();
		reader.read_array
		(	|reader|
			{	result.insert(reader.read_index()?);
				Ok(())
			}
		)?;
		Ok(result)
	}
}

impl<U> TryFromJson for HashMap<String, U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let mut result = HashMap::new();
		reader.read_object
		(	|reader, key|
			{	result.insert(key, reader.read_index()?);
				Ok(())
			}
		)?;
		Ok(result)
	}
}

impl<U> TryFromJson for BTreeMap<String, U> where U: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let mut result = BTreeMap::new();
		reader.read_object
		(	|reader, key|
			{	result.insert(key, reader.read_index()?);
				Ok(())
			}
		)?;
		Ok(result)
	}
}

impl<U, V> TryFromJson for (U, V) where U: TryFromJson, V: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let (a, b) = match reader.next_token()?
		{	Token::Null => return Err(reader.format_error("Value must be array[2], not null")),
			Token::False => return Err(reader.format_error("Value must be array[2], not boolean")),
			Token::True => return Err(reader.format_error("Value must be array[2], not boolean")),
			Token::Number(_e, _n) => return Err(reader.format_error("Value must be array[2], not number")),
			Token::Quote => return Err(reader.format_error("Value must be array[2], not string")),
			Token::ArrayBegin =>
			{	// begin read tuple
				reader.path.push(PathItem::Index(0));
				// .0
				let a = U::try_from_json(reader)?;
				match reader.next_token()?
				{	Token::Null => return Err(reader.format_error("Invalid JSON input: expected ',', got null")),
					Token::False => return Err(reader.format_error("Invalid JSON input: expected ',', got false")),
					Token::True => return Err(reader.format_error("Invalid JSON input: expected ',', got true")),
					Token::Number(_e, _n) => return Err(reader.format_error("Invalid JSON input: expected ',', got number")),
					Token::Quote => return Err(reader.format_error("Invalid JSON input: expected ',', got string")),
					Token::ArrayBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '['")),
					Token::ArrayEnd => return Err(reader.format_error("Value must be array[2], not array[1]")),
					Token::ObjectBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '{'")),
					Token::ObjectEnd => return Err(reader.format_error("Invalid JSON input: expected ',', got '}'")),
					Token::Comma => {},
					Token::Colon => return Err(reader.format_error("Invalid JSON input: expected ',', got ':'")),
				}
				// next
				if let Some(p) = reader.path.last_mut()
				{	*p = PathItem::Index(1);
				}
				// .1
				let b = V::try_from_json(reader)?;
				match reader.next_token()?
				{	Token::Null => return Err(reader.format_error("Invalid JSON input: expected ',', got null")),
					Token::False => return Err(reader.format_error("Invalid JSON input: expected ',', got false")),
					Token::True => return Err(reader.format_error("Invalid JSON input: expected ',', got true")),
					Token::Number(_e, _n) => return Err(reader.format_error("Invalid JSON input: expected ',', got number")),
					Token::Quote => return Err(reader.format_error("Invalid JSON input: expected ',', got string")),
					Token::ArrayBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '['")),
					Token::ArrayEnd => {},
					Token::ObjectBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '{'")),
					Token::ObjectEnd => return Err(reader.format_error("Invalid JSON input: expected ',', got '}'")),
					Token::Comma => return Err(reader.format_error("Expected array with 2 elements, got more")),
					Token::Colon => return Err(reader.format_error("Invalid JSON input: expected ',', got ':'")),
				}
				// end read tuple
				reader.path.pop();
				(a, b)
			}
			Token::ArrayEnd => return Err(reader.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => return Err(reader.format_error("Value must be array[2], not object")),
			Token::ObjectEnd => return Err(reader.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => return Err(reader.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => return Err(reader.format_error("Invalid JSON input: unexpected ':'")),
		};
		Ok((a, b))
	}
}

impl<U, V, W> TryFromJson for (U, V, W) where U: TryFromJson, V: TryFromJson, W: TryFromJson
{	fn try_from_json<T>(reader: &mut Reader<T>) -> io::Result<Self> where T: Iterator<Item=u8>
	{	let (a, b, c) = match reader.next_token()?
		{	Token::Null => return Err(reader.format_error("Value must be array[3], not null")),
			Token::False => return Err(reader.format_error("Value must be array[3], not boolean")),
			Token::True => return Err(reader.format_error("Value must be array[3], not boolean")),
			Token::Number(_e, _n) => return Err(reader.format_error("Value must be array[3], not number")),
			Token::Quote => return Err(reader.format_error("Value must be array[3], not string")),
			Token::ArrayBegin =>
			{	// begin read tuple
				reader.path.push(PathItem::Index(0));
				// .0
				let a = U::try_from_json(reader)?;
				match reader.next_token()?
				{	Token::Null => return Err(reader.format_error("Invalid JSON input: expected ',', got null")),
					Token::False => return Err(reader.format_error("Invalid JSON input: expected ',', got false")),
					Token::True => return Err(reader.format_error("Invalid JSON input: expected ',', got true")),
					Token::Number(_e, _n) => return Err(reader.format_error("Invalid JSON input: expected ',', got number")),
					Token::Quote => return Err(reader.format_error("Invalid JSON input: expected ',', got string")),
					Token::ArrayBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '['")),
					Token::ArrayEnd => return Err(reader.format_error("Value must be array[3], not array[1]")),
					Token::ObjectBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '{'")),
					Token::ObjectEnd => return Err(reader.format_error("Invalid JSON input: expected ',', got '}'")),
					Token::Comma => {},
					Token::Colon => return Err(reader.format_error("Invalid JSON input: expected ',', got ':'")),
				}
				// next
				if let Some(p) = reader.path.last_mut()
				{	*p = PathItem::Index(1);
				}
				// .1
				let b = V::try_from_json(reader)?;
				match reader.next_token()?
				{	Token::Null => return Err(reader.format_error("Invalid JSON input: expected ',', got null")),
					Token::False => return Err(reader.format_error("Invalid JSON input: expected ',', got false")),
					Token::True => return Err(reader.format_error("Invalid JSON input: expected ',', got true")),
					Token::Number(_e, _n) => return Err(reader.format_error("Invalid JSON input: expected ',', got number")),
					Token::Quote => return Err(reader.format_error("Invalid JSON input: expected ',', got string")),
					Token::ArrayBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '['")),
					Token::ArrayEnd => return Err(reader.format_error("Value must be array[3], not array[2]")),
					Token::ObjectBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '{'")),
					Token::ObjectEnd => return Err(reader.format_error("Invalid JSON input: expected ',', got '}'")),
					Token::Comma => {},
					Token::Colon => return Err(reader.format_error("Invalid JSON input: expected ',', got ':'")),
				}
				// next
				if let Some(p) = reader.path.last_mut()
				{	*p = PathItem::Index(2);
				}
				// .2
				let c = W::try_from_json(reader)?;
				match reader.next_token()?
				{	Token::Null => return Err(reader.format_error("Invalid JSON input: expected ',', got null")),
					Token::False => return Err(reader.format_error("Invalid JSON input: expected ',', got false")),
					Token::True => return Err(reader.format_error("Invalid JSON input: expected ',', got true")),
					Token::Number(_e, _n) => return Err(reader.format_error("Invalid JSON input: expected ',', got number")),
					Token::Quote => return Err(reader.format_error("Invalid JSON input: expected ',', got string")),
					Token::ArrayBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '['")),
					Token::ArrayEnd => {},
					Token::ObjectBegin => return Err(reader.format_error("Invalid JSON input: expected ',', got '{'")),
					Token::ObjectEnd => return Err(reader.format_error("Invalid JSON input: expected ',', got '}'")),
					Token::Comma => return Err(reader.format_error("Expected array with 3 elements, got more")),
					Token::Colon => return Err(reader.format_error("Invalid JSON input: expected ',', got ':'")),
				}
				// end read tuple
				reader.path.pop();
				(a, b, c)
			}
			Token::ArrayEnd => return Err(reader.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => return Err(reader.format_error("Value must be array[3], not object")),
			Token::ObjectEnd => return Err(reader.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => return Err(reader.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => return Err(reader.format_error("Invalid JSON input: unexpected ':'")),
		};
		Ok((a, b, c))
	}
}


// pub DebugToJson

pub trait DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl DebugToJson for ()
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	write!(f, "null")
	}
}

impl DebugToJson for isize {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for i128  {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for i64   {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for i32   {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for i16   {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for i8    {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for usize {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for u128  {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for u64   {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for u32   {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for u16   {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for u8    {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for f64    {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}
impl DebugToJson for f32    {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {write!(f, "{}", self)}}

impl DebugToJson for bool
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	write!(f, "{}", if *self {"true"} else {"false"})
	}
}

impl DebugToJson for char
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	match *self
		{	'"' => write!(f, stringify!("\"")),
			'\\' => write!(f, stringify!("\\")),
			_ => write!(f, "\"{}\"", self),
		}
	}
}

impl DebugToJson for String
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	write!(f, "\"{}\"", escape(&self))
	}
}

impl DebugToJson for Value
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	match *self
		{	Value::Null => write!(f, "null"),
			Value::Bool(v) => if v {write!(f, "true")} else {write!(f, "false")},
			Value::Number(mantissa, exponent, is_negative) =>
			{	let mut buffer = [0u8; 24];
				let mantissa = mantissa.numtoa(10, &mut buffer);
				let mut buffer = [0u8; READER_BUFFER_SIZE];
				&mut buffer[0 .. mantissa.len()].copy_from_slice(&mantissa);
				let len = number_to_string(&mut buffer, mantissa.len(), exponent, is_negative).map_err(|_| fmt::Error {})?;
				write!(f, "{}", String::from_utf8_lossy(&buffer[0 .. len]))
			},
			Value::String(ref v) => write!(f, "\"{}\"", escape(v)),
			Value::Array(ref v) =>
			{	let mut c = '[';
				for item in v
				{	write!(f, "{}", c)?;
					DebugToJson::fmt(item, f)?;
					c = ',';
				}
				if c == '['
				{	write!(f, "[]")
				}
				else
				{	write!(f, "]")
				}
			}
			Value::Object(ref v) =>
			{	let mut c = '{';
				for (key, item) in v
				{	write!(f, "{}\"{}\":", c, escape(key))?;
					DebugToJson::fmt(item, f)?;
					c = ',';
				}
				if c == '{'
				{	write!(f, "{{}}")
				}
				else
				{	write!(f, "}}")
				}
			}
		}
	}
}

impl<T> DebugToJson for Box<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let v: &T = &*self;
		DebugToJson::fmt(v, f)
	}
}

impl<T> DebugToJson for std::sync::RwLock<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	match self.read()
		{	Ok(v) => DebugToJson::fmt(&*v, f),
			Err(_e) => Err(fmt::Error {})
		}
	}
}

impl<T> DebugToJson for std::sync::Mutex<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	match self.lock()
		{	Ok(v) => DebugToJson::fmt(&*v, f),
			Err(_e) => Err(fmt::Error {})
		}
	}
}

impl<T> DebugToJson for std::rc::Rc<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let v: &T = &*self;
		DebugToJson::fmt(v, f)
	}
}

impl<T> DebugToJson for std::sync::Arc<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let v: &T = &*self;
		DebugToJson::fmt(v, f)
	}
}

impl<T> DebugToJson for Option<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	match *self
		{	Some(ref v) => DebugToJson::fmt(v, f),
			None => write!(f, "null"),
		}
	}
}

impl<T> DebugToJson for Vec<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(f, "{}", c)?;
			DebugToJson::fmt(item, f)?;
			c = ',';
		}
		if c == '['
		{	write!(f, "[]")
		}
		else
		{	write!(f, "]")
		}
	}
}

impl<T> DebugToJson for HashSet<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(f, "{}", c)?;
			DebugToJson::fmt(item, f)?;
			c = ',';
		}
		if c == '['
		{	write!(f, "[]")
		}
		else
		{	write!(f, "]")
		}
	}
}

impl<T> DebugToJson for LinkedList<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(f, "{}", c)?;
			DebugToJson::fmt(item, f)?;
			c = ',';
		}
		if c == '['
		{	write!(f, "[]")
		}
		else
		{	write!(f, "]")
		}
	}
}

impl<T> DebugToJson for VecDeque<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(f, "{}", c)?;
			DebugToJson::fmt(item, f)?;
			c = ',';
		}
		if c == '['
		{	write!(f, "[]")
		}
		else
		{	write!(f, "]")
		}
	}
}

impl<T> DebugToJson for BTreeSet<T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(f, "{}", c)?;
			DebugToJson::fmt(item, f)?;
			c = ',';
		}
		if c == '['
		{	write!(f, "[]")
		}
		else
		{	write!(f, "]")
		}
	}
}

impl<T> DebugToJson for HashMap<String, T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '{';
		for (key, item) in self
		{	write!(f, "{}\"{}\":", c, escape(key))?;
			DebugToJson::fmt(item, f)?;
			c = ',';
		}
		if c == '{'
		{	write!(f, "{{}}")
		}
		else
		{	write!(f, "}}")
		}
	}
}

impl<T> DebugToJson for BTreeMap<String, T> where T: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '{';
		for (key, item) in self
		{	write!(f, "{}\"{}\":", c, escape(key))?;
			DebugToJson::fmt(item, f)?;
			c = ',';
		}
		if c == '{'
		{	write!(f, "{{}}")
		}
		else
		{	write!(f, "}}")
		}
	}
}

impl<U, V> DebugToJson for (U, V) where U: DebugToJson, V: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	write!(f, "[")?;
		DebugToJson::fmt(&self.0, f)?;
		write!(f, ",")?;
		DebugToJson::fmt(&self.1, f)?;
		write!(f, "]")
	}
}

impl<U, V, W> DebugToJson for (U, V, W) where U: DebugToJson, V: DebugToJson, W: DebugToJson
{	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{	write!(f, "[")?;
		DebugToJson::fmt(&self.0, f)?;
		write!(f, ",")?;
		DebugToJson::fmt(&self.1, f)?;
		write!(f, ",")?;
		DebugToJson::fmt(&self.2, f)?;
		write!(f, "]")
	}
}


// pub escape

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
	if let Some(mut pos) = bytes.iter().position(|c| match *c {b'"' | b'\\' | 0..=31 => true, _ => false})
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
		Cow::Owned(String::from_utf8(buffer).unwrap())
	}
	else
	{	Cow::Borrowed(s)
	}
}


// pub Reader

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token
{	Null, False, True, Number(i16, bool), Quote, ArrayBegin, ArrayEnd, ObjectBegin, ObjectEnd, Comma, Colon
}

enum PathItem
{	Prop(&'static str),
	Index(usize),
}

fn number_to_string(buffer: &mut [u8; READER_BUFFER_SIZE], mut len: usize, mut exponent: i16, is_negative: bool) -> Result<usize, ()>
{	if len == 0
	{	buffer[0] = b'0';
		return Ok(1);
	}
	let mut pos = 0;
	if is_negative
	{	if len == buffer.len()
		{	len -= 1;
			exponent = exponent.checked_add(1).ok_or(())?;
		}
		buffer.copy_within(0..len, 1);
		buffer[0] = b'-';
		pos = 1;
	}
	if exponent >= 0
	{	let e = exponent as usize;
		if len+e <= FORMAT_NUM_WIDTH
		{	// append zeroes according to exponent
			&mut buffer[pos+len .. pos+len+e].copy_from_slice(&FORMAT_NUM_WIDTH_Z[0 .. e]);
			return Ok(pos + len + e);
		}
	}
	else
	{	let e = exponent.wrapping_neg() as usize;
		if e < len && len < buffer.len()-1
		{	// insert dot in the middle of number
			buffer.copy_within(pos+len-e .. pos+len, pos+len-e+1);
			buffer[pos+len-e] = b'.';
			return Ok(pos + len + 1);
		}
		if e <= FORMAT_NUM_WIDTH
		{	// prepend with 0.000...
			buffer.copy_within(pos .. pos+len, pos+e-len+2);
			&mut buffer[pos .. pos+e-len+2].copy_from_slice(&FORMAT_NUM_WIDTH_0Z[0 .. e-len+2]);
			return Ok(e + 2);
		}
	}
	len += pos;
	let mut buffer_2 = [0u8; 24];
	loop
	{	let exponent_str = exponent.numtoa(10, &mut buffer_2);
		if len+1+exponent_str.len() > buffer.len()
		{	let overflow = len+1+exponent_str.len() - buffer.len();
			exponent = exponent.checked_add(overflow as i16).ok_or(())?;
			len -= overflow;
		}
		else
		{	buffer[len] = b'e';
			len += 1;
			&mut buffer[len .. len+exponent_str.len()].copy_from_slice(exponent_str);
			len += exponent_str.len();
			return Ok(len);
		}
	}
}

pub struct ReadToIterator<T> where T: io::Read
{	reader: T,
}

impl<T> ReadToIterator<T> where T: io::Read
{	pub fn new(reader: T) -> Self
	{	Self {reader}
	}

	pub fn unwrap(self) -> T
	{	self.reader
	}
}

impl<T> Iterator for ReadToIterator<T> where T: io::Read
{	type Item = u8;

	fn next(&mut self) -> Option<Self::Item>
	{	let mut buffer = [0u8];
		match self.reader.read(&mut buffer)
		{	Ok(_) => Some(buffer[0]),
			Err(_) => None,
		}
	}
}

pub struct Reader<T> where T: Iterator<Item=u8>
{	iter: T,
	lookahead: u8,
	path: Vec<PathItem>,
	last_index: usize,
	buffer_len: usize,
	buffer: [u8; READER_BUFFER_SIZE], // must be at least 48 bytes for correct number reading
}
impl<T> Reader<T> where T: Iterator<Item=u8>
{	/// Construct new reader object, that can read values from a JSON stream, passing an object that implements `Iterator<u8>`.
	/// This allows to use `&str` as data source like this:
	/// ```
	/// # use nop_json::Reader;
	/// let source = "\"Data\"";
	/// let mut reader = Reader::new(source.bytes());
	/// ```
	/// To use `&[u8]` do this:
	/// ```
	/// # use nop_json::Reader;
	/// let source: &[u8] = b"\"Data\"";
	/// let mut reader = Reader::new(source.iter().map(|i| *i));
	/// ```
	/// To use `std::io::Read` as source, this crate has adapter object that converts `std::io::Read` to `Iterator<u8>`.
	/// ```
	/// use nop_json::{Reader, ReadToIterator};
	/// let source = std::io::stdin();
	/// let source = source.lock(); // this implements std::io::Read
	/// let mut reader = Reader::new(ReadToIterator::new(source));
	/// ```
	pub fn new(iter: T) -> Reader<T>
	{	Reader
		{	iter,
			lookahead: b' ',
			path: Vec::new(),
			last_index: 0,
			buffer_len: 0,
			buffer: [0u8; READER_BUFFER_SIZE],
		}
	}

	/// Destroy this reader, unwrapping the underlying iterator.
	pub fn unwrap(self) -> T
	{	self.iter
	}

	pub fn read<U>(&mut self) -> io::Result<U> where U: TryFromJson
	{	U::try_from_json(self)
	}

	pub fn read_prop<U>(&mut self, prop: &'static str) -> io::Result<U> where U: TryFromJson
	{	self.path.push(PathItem::Prop(prop));
		let result = self.read();
		self.path.pop();
		result
	}

	pub fn read_index<U>(&mut self) -> io::Result<U> where U: TryFromJson
	{	if let Some(p) = self.path.last_mut()
		{	*p = PathItem::Index(self.last_index);
			self.last_index += 1;
		}
		self.read()
	}

	fn get_path_str(&self) -> String
	{	let mut s = "$".to_string();
		for i in &self.path
		{	match i
			{	PathItem::Prop(prop) => {s.push('.'); s.push_str(prop)}
				PathItem::Index(index) => s.push_str(&format!("[{}]", index))
			}
		}
		s
	}

	pub fn format_error(&self, msg: &str) -> io::Error
	{	let mut s = self.get_path_str();
		s.push_str(": ");
		s.push_str(msg);
		io::Error::new(io::ErrorKind::Other, s)
	}

	pub fn format_error_fmt(&self, args: fmt::Arguments) -> io::Error
	{	let mut s = self.get_path_str();
		s.push_str(": ");
		use fmt::Write;
		s.write_fmt(args).ok();
		io::Error::new(io::ErrorKind::Other, s)
	}

	fn number_error(&self) -> io::Error
	{	self.format_error("Invalid JSON input: Number is too big")
	}

	fn get_next_char(&mut self) -> u8
	{	while self.lookahead.is_ascii_whitespace()
		{	match self.iter.next()
			{	Some(c) => self.lookahead = c,
				None => self.lookahead = b' ',
			}
		}
		self.lookahead
	}

	fn next_token(&mut self) -> io::Result<Token>
	{	let mut c = self.lookahead;
		loop
		{	match c
			{	b' ' | b'\t' | b'\r' | b'\n' =>
				{	c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
				}
				b'n' =>
				{	if let Some(b'u') = self.iter.next()
					{	if let Some(b'l') = self.iter.next()
						{	if let Some(b'l') = self.iter.next()
							{	if let Some(c) = self.iter.next()
								{	if !c.is_ascii_alphanumeric() && c!=b'_'
									{	self.lookahead = c;
										return Ok(Token::Null);
									}
								}
								else
								{	self.lookahead = b' ';
									return Ok(Token::Null);
								}
							}
						}
					}
					self.lookahead = b' ';
					return Err(self.format_error("Invalid JSON input: unexpected identifier"));
				}
				b'f' =>
				{	if let Some(b'a') = self.iter.next()
					{	if let Some(b'l') = self.iter.next()
						{	if let Some(b's') = self.iter.next()
							{	if let Some(b'e') = self.iter.next()
								{	if let Some(c) = self.iter.next()
									{	if !c.is_ascii_alphanumeric() && c!=b'_'
										{	self.lookahead = c;
											return Ok(Token::False);
										}
									}
									else
									{	self.lookahead = b' ';
										return Ok(Token::False);
									}
								}
							}
						}
					}
					self.lookahead = b' ';
					return Err(self.format_error("Invalid JSON input: unexpected identifier"));
				}
				b't' =>
				{	if let Some(b'r') = self.iter.next()
					{	if let Some(b'u') = self.iter.next()
						{	if let Some(b'e') = self.iter.next()
							{	if let Some(c) = self.iter.next()
								{	if !c.is_ascii_alphanumeric() && c!=b'_'
									{	self.lookahead = c;
										return Ok(Token::True);
									}
								}
								else
								{	self.lookahead = b' ';
									return Ok(Token::True);
								}
							}
						}
					}
					self.lookahead = b' ';
					return Err(self.format_error("Invalid JSON input: unexpected identifier"));
				}
				b'0'..=b'9' | b'-' | b'.' =>
				{	let mut is_negative = false;
					let mut exponent = 0i32;
					let mut is_after_dot = 0;
					let mut pos = 0;
					let mut n_trailing_zeroes = 0;
					if c == b'-'
					{	is_negative = true;
						c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
					}
					loop
					{	match c
						{	b'0' =>
							{	exponent += is_after_dot;
								if pos > 0
								{	n_trailing_zeroes += 1;
									if pos < self.buffer.len()
									{	self.buffer[pos] = b'0';
										pos += 1;
									}
								}
							}
							b'1'..= b'9' =>
							{	exponent += is_after_dot;
								n_trailing_zeroes = 0;
								if pos < self.buffer.len()
								{	self.buffer[pos] = c;
									pos += 1;
								}
							}
							b'.' => {is_after_dot = -1}
							b'e' | b'E' =>
							{	c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
								let mut n_is_negative = false;
								match c
								{	b'+' => {c = b'0'}
									b'-' => {c = b'0'; n_is_negative = true}
									b'0' ..= b'9' => {}
									_ =>
									{	self.lookahead = c;
										return Err(self.format_error("Invalid JSON input: invalid number format"));
									}
								};
								let mut n: i32 = 0;
								let mut is_error = false;
								loop
								{	match c
									{	b'0' =>
										{	n = match n.checked_mul(10)
											{	Some(n) => n,
												None => {is_error = true; n_trailing_zeroes = 1; 0} // i check for error inside if n_trailing_zeroes (for optimization)
											}
										}
										b'1'..=b'9' =>
										{	n = match n.checked_mul(10).and_then(|n| n.checked_add((c-b'0') as i32))
											{	Some(n) => n,
												None => {is_error = true; n_trailing_zeroes = 1; 0} // i check for error inside if n_trailing_zeroes (for optimization)
											}
										}
										_ =>
										{	self.lookahead = c;
											break;
										}
									}
									if let Some(new_c) = self.iter.next()
									{	c = new_c;
									}
									else
									{	self.lookahead = b' ';
										break;
									}
								}
								if n_trailing_zeroes > 0
								{	if is_error
									{	self.lookahead = b' ';
										return Err(self.number_error());
									}
									if is_after_dot == 0
									{	exponent += n_trailing_zeroes;
									}
									pos -= n_trailing_zeroes as usize;
								}
								match exponent.checked_add(if n_is_negative {-n} else {n})
								{	Some(new_exponent) => exponent = new_exponent,
									None =>
									{	self.lookahead = b' ';
										return Err(self.number_error());
									}
								}
								break;
							}
							_ =>
							{	self.lookahead = c;
								if n_trailing_zeroes > 0
								{	if is_after_dot == 0
									{	exponent += n_trailing_zeroes;
									}
									pos -= n_trailing_zeroes as usize;
								}
								break;
							}
						}
						if let Some(new_c) = self.iter.next()
						{	c = new_c;
						}
						else
						{	self.lookahead = b' ';
							break;
						}
					}
					self.buffer_len = pos;
					return match exponent.try_into()
					{	Ok(exponent) => Ok(Token::Number(exponent, is_negative)),
						Err(_) => Err(self.number_error())
					};
				}
				b'"' =>
				{	// no need for: self.lookahead = ... because i will call read_string_contents() or read_string_contents_as_bytes() or skip_string() then
					return Ok(Token::Quote);
				}
				b'[' =>
				{	self.lookahead = b' ';
					return Ok(Token::ArrayBegin);
				}
				b']' =>
				{	self.lookahead = b' ';
					return Ok(Token::ArrayEnd);
				}
				b'{' =>
				{	self.lookahead = b' ';
					return Ok(Token::ObjectBegin);
				}
				b'}' =>
				{	self.lookahead = b' ';
					return Ok(Token::ObjectEnd);
				}
				b',' =>
				{	self.lookahead = b' ';
					return Ok(Token::Comma);
				}
				b':' =>
				{	self.lookahead = b' ';
					return Ok(Token::Colon);
				}
				_ =>
				{	self.lookahead = b' ';
					return Err(self.format_error_fmt(format_args!("Invalid JSON input: unexpected '{}'", String::from_utf8_lossy(&[c]))));
				}
			}
		}
	}

	fn skip_string(&mut self) -> io::Result<()>
	{	self.lookahead = b' ';
		loop
		{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			match c
			{	b'"' =>
				{	break;
				}
				b'\\' =>
				{	self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
				}
				_ => {}
			}
		}
		Ok(())
	}

	fn u_escape_to_utf8(&mut self, buf_pos: usize) -> io::Result<usize>
	{	let c0 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
		let c1 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
		let c2 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
		let c3 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
		let c = (self.hex_to_u32(c0)? << 12) | (self.hex_to_u32(c1)? << 8) | (self.hex_to_u32(c2)? << 4) | self.hex_to_u32(c3)?;
		if c <= 0x7F
		{	if buf_pos == self.buffer.len()
			{	Ok(0)
			}
			else
			{	self.buffer[buf_pos] = c as u8;
				Ok(1)
			}
		}
		else if c <= 0x7FF
		{	if buf_pos+1 >= self.buffer.len()
			{	Ok((&mut self.buffer[buf_pos ..]).write(&[(0xC0 | (c >> 6)) as u8, (0x80 | (c & 0x3F)) as u8]).unwrap())
			}
			else
			{	self.buffer[buf_pos] = (0xC0 | (c >> 6)) as u8;
				self.buffer[buf_pos+1] = (0x80 | (c & 0x3F)) as u8;
				Ok(2)
			}
		}
		else if c <= 0xD7FF || c >= 0xE000
		{	if buf_pos+2 >= self.buffer.len()
			{	Ok((&mut self.buffer[buf_pos ..]).write(&[(0xE0 | (c >> 12)) as u8, (0x80 | ((c >> 6) & 0x3F)) as u8, (0x80 | (c & 0x3F)) as u8]).unwrap())
			}
			else
			{	self.buffer[buf_pos] = (0xE0 | (c >> 12)) as u8;
				self.buffer[buf_pos+1] = (0x80 | ((c >> 6) & 0x3F)) as u8;
				self.buffer[buf_pos+2] = (0x80 | (c & 0x3F)) as u8;
				Ok(2)
			}
		}
		else if c <= 0xDBFF
		{	// UTF-16 surrogate pairs
			let c0 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			let c1 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			let c2 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			let c3 = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			let cc = (self.hex_to_u32(c0)? << 12) | (self.hex_to_u32(c1)? << 8) | (self.hex_to_u32(c2)? << 4) | self.hex_to_u32(c3)?;
			if cc >= 0xDC00 && cc <= 0xDFFF
			{	let c = 0x10000 + (((c-0xD800) << 10) | (cc-0xDC00));
				Ok((&mut self.buffer[buf_pos ..]).write(&[0xFFu8, (c >> 18) as u8, (0x80 | ((c >> 12) & 0x3F)) as u8, (0x80 | ((c >> 6) & 0x3F)) as u8, (0x80 | (c & 0x3F)) as u8]).unwrap())
			}
			else
			{	Err(self.format_error("Invalid UTF-16 surrogate pair"))
			}
		}
		else
		{	Err(self.format_error("Escape sequence doesn't map to UTF-8"))
		}
	}

	#[inline]
	fn hex_to_u32(&self, c: u8) -> io::Result<u32>
	{	match c
		{	b'0' ..= b'9' => Ok((c as u32) - (b'0' as u32)),
			b'a' ..= b'f' => Ok((c as u32) - ((b'a' - 10) as u32)),
			b'A' ..= b'F' => Ok((c as u32) - ((b'A' - 10) as u32)),
			_ => return Err(self.format_error("Invalid JSON input: error in escape sequence"))
		}
	}

	fn read_string_contents(&mut self) -> io::Result<String>
	{	String::from_utf8(self.read_blob_contents()?).map_err(|_| self.format_error("Invalid UTF-8 string"))
	}

	fn read_blob_contents(&mut self) -> io::Result<Vec<u8>>
	{	let mut bytes = Vec::new();
		loop
		{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			match c
			{	b'"' => break,
				b'\\' =>
				{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
					match c
					{	b'r' => bytes.push(b'\r'),
						b'n' => bytes.push(b'\n'),
						b't' => bytes.push(b'\t'),
						b'b' => bytes.push(8),
						b'f' => bytes.push(12),
						b'u' =>
						{	let len = self.u_escape_to_utf8(0)?;
							bytes.extend_from_slice(&self.buffer[0 .. len]);
						},
						_ => bytes.push(c)
					}
				}
				_ => bytes.push(c)
			}
		}
		self.lookahead = b' ';
		Ok(bytes)
	}

	fn read_string_contents_as_bytes(&mut self) -> io::Result<()>
	{	let mut len = 0;
		loop
		{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			match c
			{	b'"' =>
				{	self.lookahead = b' ';
					break;
				}
				b'\\' =>
				{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
					match c
					{	b'r' =>
						{	if len < self.buffer.len() {self.buffer[len] = b'\r'; len += 1}
						},
						b'n' =>
						{	if len < self.buffer.len() {self.buffer[len] = b'\n'; len += 1}
						},
						b't' =>
						{	if len < self.buffer.len() {self.buffer[len] = b'\t'; len += 1}
						},
						b'b' =>
						{	if len < self.buffer.len() {self.buffer[len] = 8; len += 1}
						},
						b'f' =>
						{	if len < self.buffer.len() {self.buffer[len] = 12; len += 1}
						},
						b'u' =>
						{	len += self.u_escape_to_utf8(len)?
						},
						_ =>
						{	if len < self.buffer.len() {self.buffer[len] = c; len += 1}
						}
					}
				}
				_ => len += (&mut self.buffer[len ..]).write(&[c]).unwrap()
			}
		}
		self.buffer_len = len;
		Ok(())
	}

	fn pipe_blob_contents<U>(&mut self, writer: &mut U) -> io::Result<()> where U: io::Write
	{	let mut len = 0;
		loop
		{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
			let c = match c
			{	b'"' =>
				{	self.lookahead = b' ';
					break;
				}
				b'\\' =>
				{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
					match c
					{	b'r' => b'\r',
						b'n' => b'\n',
						b't' => b'\t',
						b'b' => 8,
						b'f' => 12,
						b'u' =>
						{	if len+4 >= self.buffer.len() {writer.write_all(&self.buffer[0 .. len])?; len = 0}
							len += self.u_escape_to_utf8(len)?;
							continue;
						},
						_ => c
					}
				}
				_ => c
			};
			if len >= self.buffer.len() {writer.write_all(&self.buffer[0 .. len])?; len = 0}
			self.buffer[len] = c;
			len += 1;
		}
		if len > 0
		{	writer.write_all(&self.buffer[0 .. len])?;
		}
		Ok(())
	}

	fn skip_array(&mut self) -> io::Result<()>
	{	enum State {AtValueOrEnd, AtValue, AtCommaOrEnd}
		let mut state = State::AtValueOrEnd;
		loop
		{	state = match state
			{	State::AtValueOrEnd =>
				{	match self.next_token()?
					{	Token::Null => State::AtCommaOrEnd,
						Token::False => State::AtCommaOrEnd,
						Token::True => State::AtCommaOrEnd,
						Token::Number(_e, _n) => State::AtCommaOrEnd,
						Token::Quote => {self.skip_string()?; State::AtCommaOrEnd },
						Token::ArrayBegin => {self.skip_array()?; State::AtCommaOrEnd },
						Token::ArrayEnd => break,
						Token::ObjectBegin => {self.skip_object()?; State::AtCommaOrEnd },
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: unexpected '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: unexpected ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: unexpected ':'")),
					}
				}
				State::AtValue =>
				{	match self.next_token()?
					{	Token::Null => State::AtCommaOrEnd,
						Token::False => State::AtCommaOrEnd,
						Token::True => State::AtCommaOrEnd,
						Token::Number(_e, _n) => State::AtCommaOrEnd,
						Token::Quote => {self.skip_string()?; State::AtCommaOrEnd },
						Token::ArrayBegin => {self.skip_array()?; State::AtCommaOrEnd },
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: unexpected ']'")),
						Token::ObjectBegin => {self.skip_object()?; State::AtCommaOrEnd },
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: unexpected '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: unexpected ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: unexpected ':'")),
					}
				}
				State::AtCommaOrEnd =>
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: unexpected null literal")),
						Token::False => return Err(self.format_error("Invalid JSON input: unexpected false literal")),
						Token::True => return Err(self.format_error("Invalid JSON input: unexpected true literal")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: unexpected number literal")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: unexpected string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: unexpected '['")),
						Token::ArrayEnd => break,
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: unexpected '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: unexpected '}'")),
						Token::Comma => State::AtValue,
						Token::Colon => return Err(self.format_error("Invalid JSON input: unexpected ':'")),
					}
				}
			};
		}
		Ok(())
	}

	fn skip_object(&mut self) -> io::Result<()>
	{	enum State {AtKeyOrEnd, AtKey, AtColon, AtValue, AtCommaOrEnd}
		let mut state = State::AtKeyOrEnd;
		loop
		{	state = match state
			{	State::AtKeyOrEnd =>
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: unexpected null literal")),
						Token::False => return Err(self.format_error("Invalid JSON input: unexpected false literal")),
						Token::True => return Err(self.format_error("Invalid JSON input: unexpected true literal")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: unexpected number literal")),
						Token::Quote => {self.skip_string()?; State::AtColon},
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: unexpected '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: unexpected ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: unexpected '{'")),
						Token::ObjectEnd => break,
						Token::Comma => return Err(self.format_error("Invalid JSON input: unexpected ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: unexpected ':'")),
					}
				}
				State::AtKey =>
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: unexpected null literal")),
						Token::False => return Err(self.format_error("Invalid JSON input: unexpected false literal")),
						Token::True => return Err(self.format_error("Invalid JSON input: unexpected true literal")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: unexpected number literal")),
						Token::Quote => {self.skip_string()?; State::AtColon},
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: unexpected '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: unexpected ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: unexpected '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: unexpected '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: unexpected ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: unexpected ':'")),
					}
				}
				State::AtColon =>
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: unexpected null literal")),
						Token::False => return Err(self.format_error("Invalid JSON input: unexpected false literal")),
						Token::True => return Err(self.format_error("Invalid JSON input: unexpected true literal")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: unexpected number literal")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: unexpected string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: unexpected '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: unexpected ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: unexpected '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: unexpected '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: unexpected ','")),
						Token::Colon => State::AtValue,
					}
				}
				State::AtValue =>
				{	match self.next_token()?
					{	Token::Null => State::AtCommaOrEnd,
						Token::False => State::AtCommaOrEnd,
						Token::True => State::AtCommaOrEnd,
						Token::Number(_e, _n) => State::AtCommaOrEnd,
						Token::Quote => {self.skip_string()?; State::AtCommaOrEnd },
						Token::ArrayBegin => {self.skip_array()?; State::AtCommaOrEnd },
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: unexpected ']'")),
						Token::ObjectBegin => {self.skip_object()?; State::AtCommaOrEnd },
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: unexpected '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: unexpected ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: unexpected ':'")),
					}
				}
				State::AtCommaOrEnd =>
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: unexpected null literal")),
						Token::False => return Err(self.format_error("Invalid JSON input: unexpected false literal")),
						Token::True => return Err(self.format_error("Invalid JSON input: unexpected true literal")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: unexpected number literal")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: unexpected string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: unexpected '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: unexpected ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: unexpected '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: unexpected '}'")),
						Token::Comma => State::AtKey,
						Token::Colon => return Err(self.format_error("Invalid JSON input: unexpected ':'")),
					}
				}
			};
		}
		Ok(())
	}

	fn read_bool(&mut self) -> io::Result<bool>
	{	match self.next_token()?
		{	Token::Null => Ok(false),
			Token::False => Ok(false),
			Token::True => Ok(true),
			Token::Number(_e, _n) => Ok(self.buffer_len != 0),
			Token::Quote =>
			{	let c = self.iter.next().ok_or_else(|| self.format_error("Invalid JSON: unexpected end of input"))?;
				if c == b'"'
				{	self.lookahead = b' ';
					Ok(false)
				}
				else
				{	self.skip_string()?;
					Ok(true)
				}
			},
			Token::ArrayBegin => {self.skip_array()?; Ok(true)},
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => {self.skip_object()?; Ok(true)},
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	fn read_isize(&mut self) -> io::Result<isize>
	{	read_int!(self, isize, false)
	}

	fn read_i128(&mut self) -> io::Result<i128>
	{	read_int!(self, i128, false)
	}

	fn read_i64(&mut self) -> io::Result<i64>
	{	read_int!(self, i64, false)
	}

	fn read_i32(&mut self) -> io::Result<i32>
	{	read_int!(self, i32, false)
	}

	fn read_i16(&mut self) -> io::Result<i16>
	{	read_int!(self, i16, false)
	}

	fn read_i8(&mut self) -> io::Result<i8>
	{	read_int!(self, i8, false)
	}

	fn read_usize(&mut self) -> io::Result<usize>
	{	read_int!(self, usize, true)
	}

	fn read_u128(&mut self) -> io::Result<u128>
	{	read_int!(self, u128, true)
	}

	fn read_u64(&mut self) -> io::Result<u64>
	{	read_int!(self, u64, true)
	}

	fn read_u32(&mut self) -> io::Result<u32>
	{	read_int!(self, u32, true)
	}

	fn read_u16(&mut self) -> io::Result<u16>
	{	read_int!(self, u16, true)
	}

	fn read_u8(&mut self) -> io::Result<u8>
	{	read_int!(self, u8, true)
	}

	fn read_f64(&mut self) -> io::Result<f64>
	{	read_float!(self, f64, std::f64::NAN, std::f64::INFINITY, std::f64::NEG_INFINITY)
	}

	fn read_f32(&mut self) -> io::Result<f32>
	{	read_float!(self, f32, std::f32::NAN, std::f32::INFINITY, std::f32::NEG_INFINITY)
	}

	fn read_and_discard(&mut self) -> io::Result<()>
	{	match self.next_token()?
		{	Token::Null => Ok(()),
			Token::False => Ok(()),
			Token::True => Ok(()),
			Token::Number(_exponent, _is_negative) => Ok(()),
			Token::Quote => {self.skip_string()?; Ok(())},
			Token::ArrayBegin => {self.skip_array()?; Ok(())},
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => {self.skip_object()?; Ok(())},
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	fn read_string(&mut self) -> io::Result<String>
	{	match self.next_token()?
		{	Token::Null => Ok("null".to_string()),
			Token::False => Ok("false".to_string()),
			Token::True => Ok("true".to_string()),
			Token::Number(exponent, is_negative) =>
			{	let len = number_to_string(&mut self.buffer, self.buffer_len, exponent, is_negative).map_err(|_| self.number_error())?;
				Ok(String::from_utf8_lossy(&self.buffer[0 .. len]).into_owned())
			},
			Token::Quote => self.read_string_contents(),
			Token::ArrayBegin => Err(self.format_error("Value must be string, not array")),
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => Err(self.format_error("Value must be string, not object")),
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	pub fn read_bytes(&mut self) -> io::Result<&[u8]>
	{	match self.next_token()?
		{	Token::Null =>
			{	&mut self.buffer[0 .. 4].copy_from_slice(b"null");
				Ok(&self.buffer[0 .. 4])
			},
			Token::False =>
			{	&mut self.buffer[0 .. 5].copy_from_slice(b"false");
				Ok(&self.buffer[0 .. 5])
			},
			Token::True =>
			{	&mut self.buffer[0 .. 4].copy_from_slice(b"true");
				Ok(&self.buffer[0 .. 4])
			},
			Token::Number(exponent, is_negative) =>
			{	let len = number_to_string(&mut self.buffer, self.buffer_len, exponent, is_negative).map_err(|_| self.number_error())?;
				Ok(&self.buffer[0 .. len])
			},
			Token::Quote =>
			{	self.read_string_contents_as_bytes()?;
				Ok(&self.buffer[0 .. self.buffer_len])
			},
			Token::ArrayBegin => Err(self.format_error("Value must be string, not array")),
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => Err(self.format_error("Value must be string, not object")),
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	/// This function allows us to exploit standard JSON containers to pass binary data (BLOBs, binary large objects, or `Vec<u8>`).
	/// Does JSON standard strictly prohibit us to do so? Lets understand the principle behind this criminal operation.
	///
	/// Mr. JSON wants us to interchange only unicode strings. But he doesn't specify what unicode it must be: utf-8, utf-16, or other.
	/// What is invalid utf-8 can be valid utf-16, and what is invalid utf-16 can be valid something else. Furthermore, passing invalid strings is normal, it happens every day.
	/// The receiver of invalid string must reject it immediately, and not use it for his profit.
	/// This library obeys to this requirement, and invalid utf-8 (only this encoding is supported) will fail conversion from `Vec<u8>` to `String`.
	/// So reading an invalid utf-8 byte sequence to a `String` variable will really return error.
	/// But `nop-json` library can optionally return you the `Vec<u8>` and tell you to convert it to `String` by yourself.
	/// And you may decide not to do so, and get your hands on this useful byte sequence.
	///
	/// The trouble here is only with bytes in range `80 - FF`. Here is how we can encode our binary object:
	/// 1) `00 - 1F` - we can encode with the `\u00xx` encoding - this is the only option.
	/// 2) `20 - 7F` except `"` and `\` - we can leave intact - they are valid utf-8 JSON, or optionally we can encode them with `\u00xx`.
	/// 3) `"` and `\` - escape with a slash.
	/// 4) `80 - FF` - if we leave them as they are, this will make our string invalid utf-8 (but valid JSON container).
	/// Ask yourself can you live with this. Another option could be to encode these characters with `\u00xx` sequences, but `\u0080` expands to 2-byte utf-8 character.
	/// ```
	/// use nop_json::Reader;
	///
	/// let mut reader = Reader::new(b" \"\x80\x81\" ".iter().map(|i| *i));
	///
	/// let data = reader.read_blob().unwrap();
	/// assert_eq!(data, b"\x80\x81");
	/// ```
	pub fn read_blob(&mut self) -> io::Result<Vec<u8>>
	{	match self.next_token()?
		{	Token::Null => Ok(Vec::new()),
			Token::False => Err(self.format_error("Value must be string, not boolean")),
			Token::True => Err(self.format_error("Value must be string, not boolean")),
			Token::Number(_exponent, _is_negative) => Err(self.format_error("Value must be string, not number")),
			Token::Quote => self.read_blob_contents(),
			Token::ArrayBegin => Err(self.format_error("Value must be string, not array")),
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => Err(self.format_error("Value must be string, not object")),
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	pub fn pipe_blob<U>(&mut self, writer: &mut U) -> io::Result<()> where U: io::Write
	{	match self.next_token()?
		{	Token::Null => Ok(()),
			Token::False => Err(self.format_error("Value must be string, not boolean")),
			Token::True => Err(self.format_error("Value must be string, not boolean")),
			Token::Number(_exponent, _is_negative) => Err(self.format_error("Value must be string, not number")),
			Token::Quote => self.pipe_blob_contents(writer),
			Token::ArrayBegin => Err(self.format_error("Value must be string, not array")),
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => Err(self.format_error("Value must be string, not object")),
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	fn read_char(&mut self) -> io::Result<char>
	{	self.read_bytes()?;
		if self.buffer_len == 0
		{	return Err(self.format_error("Expected a character, got empty string"));
		}
		let c = self.buffer[0] as u32;
		if c&0x80 == 0 // 0xxxxxxx
		{	return Ok(self.buffer[0] as char);
		}
		else if c&0xE0 == 0xC0 // 110xxxxx
		{	if self.buffer_len >= 2
			{	let c = (self.buffer[1] as u32) & 0x3F | ((c & 0x1F) << 6);
				return Ok(char::from_u32(c).unwrap());
			}
		}
		else if c&0xF0 == 0xE0 // 1110xxxx
		{	if self.buffer_len >= 3
			{	let c = (self.buffer[2] as u32) & 0x3F | (((self.buffer[1] as u32) & 0x3F) << 6) | ((c & 0xF) << 12);
				return Ok(char::from_u32(c).unwrap());
			}
		}
		else if c&0xF8 == 0xF0 // 11110xxx
		{	if self.buffer_len >= 4
			{	let c = (self.buffer[3] as u32) & 0x3F | (((self.buffer[2] as u32) & 0x3F) << 6) | (((self.buffer[1] as u32) & 0x3F) << 12) | ((c & 0x7) << 18);
				return Ok(char::from_u32(c).unwrap());
			}
		}
		return Err(self.format_error("Invalid UTF-8 string"));
	}

	pub fn read_object<F>(&mut self, mut on_value: F) -> io::Result<bool> where F: FnMut(&mut Self, String) -> io::Result<()>
	{	match self.next_token()?
		{	Token::Null => Ok(false),
			Token::False => Err(self.format_error("Value must be object, not boolean")),
			Token::True => Err(self.format_error("Value must be object, not boolean")),
			Token::Number(_e, _n) => Err(self.format_error("Value must be object, not number")),
			Token::Quote => Err(self.format_error("Value must be object, not string")),
			Token::ArrayBegin => Err(self.format_error("Value must be object, not array")),
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin =>
			{	loop
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected key, got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected key, got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected key, got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected key, got number")),
						Token::Quote => {},
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected key, got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected key, got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected key, got '{'")),
						Token::ObjectEnd => break,
						Token::Comma => return Err(self.format_error("Invalid JSON input: expected key, got ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected key, got ':'")),
					}
					let key = self.read_string_contents()?;
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ':', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ':', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ':', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ':', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ':', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ':', got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected ':', got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ':', got '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: expected ':', got '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: expected ':', got ','")),
						Token::Colon => {},
					}
					on_value(self, key)?;
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got '{'")),
						Token::ObjectEnd => break,
						Token::Comma => {},
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got ':'")),
					}
				}
				Ok(true)
			}
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	pub fn read_object_use_buffer<F>(&mut self, mut on_value: F) -> io::Result<bool> where F: FnMut(&mut Self) -> io::Result<()>
	{	match self.next_token()?
		{	Token::Null => Ok(false),
			Token::False => Err(self.format_error("Value must be object, not boolean")),
			Token::True => Err(self.format_error("Value must be object, not boolean")),
			Token::Number(_e, _n) => Err(self.format_error("Value must be object, not number")),
			Token::Quote => Err(self.format_error("Value must be object, not string")),
			Token::ArrayBegin => Err(self.format_error("Value must be object, not array")),
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin =>
			{	loop
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected key, got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected key, got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected key, got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected key, got number")),
						Token::Quote => {},
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected key, got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected key, got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected key, got '{'")),
						Token::ObjectEnd => break,
						Token::Comma => return Err(self.format_error("Invalid JSON input: expected key, got ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected key, got ':'")),
					}
					self.read_string_contents_as_bytes()?;
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ':', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ':', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ':', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ':', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ':', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ':', got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected ':', got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ':', got '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: expected ':', got '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: expected ':', got ','")),
						Token::Colon => {},
					}
					on_value(self)?;
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got '{'")),
						Token::ObjectEnd => break,
						Token::Comma => {},
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got ':'")),
					}
				}
				Ok(true)
			}
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	pub fn get_key(&self) -> &[u8]
	{	&self.buffer[0 .. self.buffer_len]
	}

	pub fn read_array<F>(&mut self, mut on_value: F) -> io::Result<bool> where F: FnMut(&mut Self) -> io::Result<()>
	{	match self.next_token()?
		{	Token::Null => Ok(false),
			Token::False => Err(self.format_error("Value must be array, not boolean")),
			Token::True => Err(self.format_error("Value must be array, not boolean")),
			Token::Number(_e, _n) => Err(self.format_error("Value must be array, not number")),
			Token::Quote => Err(self.format_error("Value must be array, not string")),
			Token::ArrayBegin =>
			{	self.path.push(PathItem::Index(0));
				self.last_index = 0;
				loop
				{	on_value(self)?;
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got '['")),
						Token::ArrayEnd => break,
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got '}'")),
						Token::Comma => {},
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got ':'")),
					}
				}
				self.path.pop();
				Ok(true)
			}
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin => Err(self.format_error("Value must be array, not object")),
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}

	fn read_value(&mut self) -> io::Result<Value>
	{	match self.next_token()?
		{	Token::Null => Ok(Value::Null),
			Token::False => Ok(Value::Bool(false)),
			Token::True => Ok(Value::Bool(true)),
			Token::Number(exponent, is_negative) =>
			{	let mut mantissa = 0u64;
				for c in &self.buffer[.. self.buffer_len]
				{	mantissa = mantissa.checked_mul(10).ok_or_else(|| self.number_error())?;
					mantissa = mantissa.checked_add((*c - b'0') as u64).ok_or_else(|| self.number_error())?;
				}
				Ok(Value::Number(mantissa, exponent, is_negative))
			},
			Token::Quote => Ok(Value::String(self.read_string_contents()?)),
			Token::ArrayBegin =>
			{	let mut vec = Vec::new();
				loop
				{	vec.push(self.read_value()?);
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got '['")),
						Token::ArrayEnd => break,
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got '}'")),
						Token::Comma => {},
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected ',' or ']', got ':'")),
					}
				}
				Ok(Value::Array(vec))
			}
			Token::ArrayEnd => Err(self.format_error("Invalid JSON input: unexpected ']'")),
			Token::ObjectBegin =>
			{	let mut obj = HashMap::new();
				loop
				{	match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected key, got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected key, got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected key, got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected key, got number")),
						Token::Quote => {},
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected key, got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected key, got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected key, got '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: expected key, got '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: expected key, got ','")),
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected key, got ':'")),
					}
					let key = self.read_string_contents()?;
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ':', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ':', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ':', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ':', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ':', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ':', got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected ':', got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ':', got '{'")),
						Token::ObjectEnd => return Err(self.format_error("Invalid JSON input: expected ':', got '}'")),
						Token::Comma => return Err(self.format_error("Invalid JSON input: expected ':', got ','")),
						Token::Colon => {},
					}
					obj.insert(key, self.read_value()?);
					match self.next_token()?
					{	Token::Null => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got null")),
						Token::False => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got false")),
						Token::True => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got true")),
						Token::Number(_e, _n) => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got number")),
						Token::Quote => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got string")),
						Token::ArrayBegin => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got '['")),
						Token::ArrayEnd => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got ']'")),
						Token::ObjectBegin => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got '{'")),
						Token::ObjectEnd => break,
						Token::Comma => {},
						Token::Colon => return Err(self.format_error("Invalid JSON input: expected ',' or '}', got ':'")),
					}
				}
				Ok(Value::Object(obj))
			},
			Token::ObjectEnd => Err(self.format_error("Invalid JSON input: unexpected '}'")),
			Token::Comma => Err(self.format_error("Invalid JSON input: unexpected ','")),
			Token::Colon => Err(self.format_error("Invalid JSON input: unexpected ':'")),
		}
	}
}
