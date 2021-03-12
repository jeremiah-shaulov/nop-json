pub use nop_json_derive::*;
use crate::nop_json::{number_to_string, READER_BUFFER_SIZE};
use crate::value::Value;
use crate::escape::escape;

use std::{char, fmt, f32, f64};
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, LinkedList, VecDeque};
use numtoa::NumToA;


/// Trait that can be automatically derived for structs and enums. When it's derived, a [Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html) implementation is also added,
/// so the object can be printed with `println!`.
///
/// Example:
///
/// ```
/// use nop_json::DebugToJson;
///
/// #[derive(DebugToJson)]
/// struct Point {x: i32, y: i32}
///
/// let point = Point {x: 1, y: 2};
/// println!("{:?}", point);
/// ```
///
/// Here is what automatic implementation expands to. If you write:
///
/// ```
/// use nop_json::DebugToJson;
///
/// #[derive(DebugToJson)]
/// struct Point {x: i32, y: i32}
/// ```
///
/// The implementation will be:
///
/// ```
/// use nop_json::DebugToJson;
///
/// struct Point {x: i32, y: i32}
///
/// impl DebugToJson for Point
/// {	fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result
/// 	{	write!(out, "{{\"x\":")?;
/// 		DebugToJson::fmt(&self.x, out)?;
/// 		write!(out, ",\"y\":")?;
/// 		DebugToJson::fmt(&self.y, out)?;
/// 		write!(out, "}}")
/// 	}
/// }
/// impl std::fmt::Debug for Point
/// {	fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result
/// 	{	DebugToJson::fmt(self, out)
/// 	}
/// }
/// ```
///
/// For more information, see [main page](index.html).
pub trait DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result;

	/// Any type that implements `DebugToJson` can be converted to JSON string. For example `DebugToJson` is implemented for primitive types:
	/// ```
	/// use nop_json::DebugToJson;
	///
	/// let smth = true;
	/// assert_eq!(smth.to_json_string(), "true".to_string());
	/// ```
	/// For custom types:
	/// ```
	/// use nop_json::DebugToJson;
	///
	/// #[derive(DebugToJson)]
	/// struct Something {value: i32}
	///
	/// let smth = Something {value: 123};
	/// assert_eq!(smth.to_json_string(), "{\"value\":123}".to_string());
	/// ```
	fn to_json_string(&self) -> String where Self: std::marker::Sized
	{	struct Wrapper<'a, T: DebugToJson>
		{	value: &'a T
		}
		impl<'a, T: DebugToJson> fmt::Display for Wrapper<'a, T>
		{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
			{	DebugToJson::fmt(self.value, out)
			}
		}
		let w = Wrapper {value: self};
		w.to_string()
	}
}

impl DebugToJson for ()
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	write!(out, "null")
	}
}

impl DebugToJson for isize {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for i128  {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for i64   {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for i32   {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for i16   {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for i8    {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for usize {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for u128  {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for u64   {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for u32   {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for u16   {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}
impl DebugToJson for u8    {fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {write!(out, "{}", self)}}

impl DebugToJson for f64
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	if *self == f64::INFINITY
		{	write!(out, "Infinity")
		}
		else if *self == f64::NEG_INFINITY
		{	write!(out, "-Infinity")
		}
		else if *self == f64::NAN
		{	write!(out, "\"NaN\"")
		}
		else
		{	write!(out, "{}", self)
		}
	}
}
impl DebugToJson for f32
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	if *self == f32::INFINITY
		{	write!(out, "\"Infinity\"")
		}
		else if *self == f32::NEG_INFINITY
		{	write!(out, "\"-Infinity\"")
		}
		else if *self == f32::NAN
		{	write!(out, "\"NaN\"")
		}
		else
		{	write!(out, "{}", self)
		}
	}
}

impl DebugToJson for bool
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	write!(out, "{}", if *self {"true"} else {"false"})
	}
}

impl DebugToJson for char
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	match *self
		{	'"' => write!(out, stringify!("\"")),
			'\\' => write!(out, stringify!("\\")),
			_ => write!(out, "\"{}\"", self),
		}
	}
}

impl DebugToJson for String
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	write!(out, "\"{}\"", escape(&self))
	}
}

impl DebugToJson for Value
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	match *self
		{	Value::Null => write!(out, "null"),
			Value::Bool(v) => if v {write!(out, "true")} else {write!(out, "false")},
			Value::Number(mantissa, exponent, is_negative) =>
			{	let mut buffer = [0u8; 24];
				let mantissa = mantissa.numtoa(10, &mut buffer);
				let mut buffer = [0u8; READER_BUFFER_SIZE];
				&mut buffer[0 .. mantissa.len()].copy_from_slice(&mantissa);
				let len = number_to_string(&mut buffer, mantissa.len(), exponent, is_negative).map_err(|_| fmt::Error {})?;
				write!(out, "{}", String::from_utf8_lossy(&buffer[0 .. len]))
			},
			Value::String(ref v) => write!(out, "\"{}\"", escape(v)),
			Value::Array(ref v) =>
			{	let mut c = '[';
				for item in v
				{	write!(out, "{}", c)?;
					DebugToJson::fmt(item, out)?;
					c = ',';
				}
				if c == '['
				{	write!(out, "[]")
				}
				else
				{	write!(out, "]")
				}
			}
			Value::Object(ref v) =>
			{	let mut c = '{';
				for (key, item) in v
				{	write!(out, "{}\"{}\":", c, escape(key))?;
					DebugToJson::fmt(item, out)?;
					c = ',';
				}
				if c == '{'
				{	write!(out, "{{}}")
				}
				else
				{	write!(out, "}}")
				}
			}
		}
	}
}

impl<T> DebugToJson for Box<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let v: &T = &*self;
		DebugToJson::fmt(v, out)
	}
}

impl<T> DebugToJson for std::sync::RwLock<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	match self.read()
		{	Ok(v) => DebugToJson::fmt(&*v, out),
			Err(_e) => Err(fmt::Error {})
		}
	}
}

impl<T> DebugToJson for std::sync::Mutex<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	match self.lock()
		{	Ok(v) => DebugToJson::fmt(&*v, out),
			Err(_e) => Err(fmt::Error {})
		}
	}
}

impl<T> DebugToJson for std::rc::Rc<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let v: &T = &*self;
		DebugToJson::fmt(v, out)
	}
}

impl<T> DebugToJson for std::sync::Arc<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let v: &T = &*self;
		DebugToJson::fmt(v, out)
	}
}

impl<T> DebugToJson for Option<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	match *self
		{	Some(ref v) => DebugToJson::fmt(v, out),
			None => write!(out, "null"),
		}
	}
}

impl<T> DebugToJson for Vec<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			DebugToJson::fmt(item, out)?;
			c = ',';
		}
		if c == '['
		{	write!(out, "[]")
		}
		else
		{	write!(out, "]")
		}
	}
}

impl<T> DebugToJson for HashSet<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			DebugToJson::fmt(item, out)?;
			c = ',';
		}
		if c == '['
		{	write!(out, "[]")
		}
		else
		{	write!(out, "]")
		}
	}
}

impl<T> DebugToJson for LinkedList<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			DebugToJson::fmt(item, out)?;
			c = ',';
		}
		if c == '['
		{	write!(out, "[]")
		}
		else
		{	write!(out, "]")
		}
	}
}

impl<T> DebugToJson for VecDeque<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			DebugToJson::fmt(item, out)?;
			c = ',';
		}
		if c == '['
		{	write!(out, "[]")
		}
		else
		{	write!(out, "]")
		}
	}
}

impl<T> DebugToJson for BTreeSet<T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			DebugToJson::fmt(item, out)?;
			c = ',';
		}
		if c == '['
		{	write!(out, "[]")
		}
		else
		{	write!(out, "]")
		}
	}
}

impl<T> DebugToJson for HashMap<String, T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '{';
		for (key, item) in self
		{	write!(out, "{}\"{}\":", c, escape(key))?;
			DebugToJson::fmt(item, out)?;
			c = ',';
		}
		if c == '{'
		{	write!(out, "{{}}")
		}
		else
		{	write!(out, "}}")
		}
	}
}

impl<T> DebugToJson for BTreeMap<String, T> where T: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	let mut c = '{';
		for (key, item) in self
		{	write!(out, "{}\"{}\":", c, escape(key))?;
			DebugToJson::fmt(item, out)?;
			c = ',';
		}
		if c == '{'
		{	write!(out, "{{}}")
		}
		else
		{	write!(out, "}}")
		}
	}
}

impl<A, B> DebugToJson for (A, B) where A: DebugToJson, B: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	write!(out, "[")?;
		DebugToJson::fmt(&self.0, out)?;
		write!(out, ",")?;
		DebugToJson::fmt(&self.1, out)?;
		write!(out, "]")
	}
}

impl<A, B, C> DebugToJson for (A, B, C) where A: DebugToJson, B: DebugToJson, C: DebugToJson
{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
	{	write!(out, "[")?;
		DebugToJson::fmt(&self.0, out)?;
		write!(out, ",")?;
		DebugToJson::fmt(&self.1, out)?;
		write!(out, ",")?;
		DebugToJson::fmt(&self.2, out)?;
		write!(out, "]")
	}
}
