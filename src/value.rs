use crate::nop_json::DebugToJson;

use std::char;
use std::fmt;
use std::collections::{HashMap, BTreeMap, LinkedList, VecDeque};
use std::convert::{TryInto, TryFrom};
use std::str::FromStr;
use std::ops::Index;
use numtoa::NumToA;

const FORMAT_NUM_WIDTH: usize = 10;
const FORMAT_NUM_WIDTH_Z: [u8; FORMAT_NUM_WIDTH] = [b'0'; FORMAT_NUM_WIDTH];
const FORMAT_NUM_WIDTH_0Z: &[u8] = b"0.0000000000";

#[derive(Clone, PartialEq)]
pub enum Value
{	Null,
	Bool(bool),
	Number(u64, i16, bool),
	String(String),
	Array(Vec<Value>),
	Object(HashMap<String, Value>)
}

impl Value
{	pub fn is_null(&self) -> bool
	{	match *self {Value::Null => true, _ => false}
	}

	pub fn is_bool(&self) -> bool
	{	match *self {Value::Bool(_) => true, _ => false}
	}

	pub fn is_number(&self) -> bool
	{	match *self {Value::Number(_, _, _) => true, _ => false}
	}

	pub fn is_string(&self) -> bool
	{	match *self {Value::String(_) => true, _ => false}
	}

	pub fn is_array(&self) -> bool
	{	match *self {Value::Array(_) => true, _ => false}
	}

	pub fn is_object(&self) -> bool
	{	match *self {Value::Object(_) => true, _ => false}
	}
}

impl fmt::Debug for Value
{	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{	DebugToJson::fmt(self, f)
	}
}

impl fmt::Display for Value
{	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{	DebugToJson::fmt(self, f)
	}
}

// 1. From value

macro_rules! impl_from_value_float
{	($ty:ty) =>
	{	impl TryFrom<Value> for $ty
		{	type Error = ();

			fn try_from(value: Value) -> Result<Self, Self::Error>
			{	match value
				{	Value::Null => Ok(0.0),
					Value::Bool(v) => Ok(if v {1.0} else {0.0}),
					Value::Number(mantissa, exponent, is_negative) =>
					{	let mut result = (mantissa as Self)*((10.0 as Self).powi(exponent as i32));
						if is_negative
						{	result = -result;
						}
						Ok(result)
					},
					Value::String(v) =>
					{	v.parse().map_err(|_| ())
					},
					Value::Array(_v) => Err(()),
					Value::Object(_v) => Err(()),
				}
			}
		}
	}
}

macro_rules! impl_from_value_signed
{	($ty:ty) =>
	{	impl TryFrom<Value> for $ty
		{	type Error = ();

			fn try_from(value: Value) -> Result<Self, Self::Error>
			{	match value
				{	Value::Null => Ok(0),
					Value::Bool(v) => Ok(if v {1} else {0}),
					Value::Number(mantissa, exponent, is_negative) =>
					{	let mut result = if is_negative
						{	Self::try_from(i64::try_from(mantissa).map_err(|_| ())?.wrapping_neg()).map_err(|_| ())?
						}
						else
						{	Self::try_from(mantissa).map_err(|_| ())?
						};
						if exponent != 0
						{	if exponent > 0
							{	result = result.checked_mul((10 as Self).checked_pow(exponent as u32).ok_or(())?).ok_or(())?;
							}
							else
							{	result = result / (10 as Self).checked_pow(exponent.checked_neg().ok_or(())? as u32).ok_or(())?;
							}
						}
						Ok(result)
					},
					Value::String(v) =>
					{	v.parse().map_err(|_| ())
					},
					Value::Array(_v) => Err(()),
					Value::Object(_v) => Err(()),
				}
			}
		}
	}
}

macro_rules! impl_from_value_unsigned
{	($ty:ty) =>
	{	impl TryFrom<Value> for $ty
		{	type Error = ();

			fn try_from(value: Value) -> Result<Self, Self::Error>
			{	match value
				{	Value::Null => Ok(0),
					Value::Bool(v) => Ok(if v {1} else {0}),
					Value::Number(mantissa, exponent, is_negative) =>
					{	if is_negative
						{	return Err(());
						}
						let mut result = Self::try_from(mantissa).map_err(|_| ())?;
						if exponent != 0
						{	if exponent > 0
							{	result = result.checked_mul((10 as Self).checked_pow(exponent as u32).ok_or(())?).ok_or(())?;
							}
							else
							{	result = result / (10 as Self).checked_pow(exponent.checked_neg().ok_or(())? as u32).ok_or(())?;
							}
						}
						Ok(result)
					},
					Value::String(v) =>
					{	v.parse().map_err(|_| ())
					},
					Value::Array(_v) => Err(()),
					Value::Object(_v) => Err(()),
				}
			}
		}
	}
}

impl_from_value_float!(f64);
impl_from_value_float!(f32);
impl_from_value_signed!(isize);
impl_from_value_signed!(i128);
impl_from_value_signed!(i64);
impl_from_value_signed!(i32);
impl_from_value_signed!(i16);
impl_from_value_signed!(i8);
impl_from_value_unsigned!(usize);
impl_from_value_unsigned!(u128);
impl_from_value_unsigned!(u64);
impl_from_value_unsigned!(u32);
impl_from_value_unsigned!(u16);
impl_from_value_unsigned!(u8);

impl TryFrom<Value> for ()
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(()),
			Value::Bool(_v) => Err(()),
			Value::Number(_mantissa, _exponent, _is_negative) => Err(()),
			Value::String(_v) => Err(()),
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for bool
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(false),
			Value::Bool(v) => Ok(v),
			Value::Number(mantissa, _exponent, _is_negative) => Ok(mantissa != 0),
			Value::String(_v) => Ok(true),
			Value::Array(_v) => Ok(true),
			Value::Object(_v) => Ok(true),
		}
	}
}

impl TryFrom<Value> for char
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok('n'),
			Value::Bool(v) => Ok(if v {'t'} else {'f'}),
			Value::Number(mantissa, _exponent, is_negative) =>
			{	if is_negative
				{	Ok('-')
				}
				else if mantissa == 0
				{	Ok('0')
				}
				else
				{	let mut buffer = [0u8; 24];
					let s = String::from_utf8_lossy(mantissa.numtoa(10, &mut buffer));
					Ok(s.chars().next().unwrap())
				}
			},
			Value::String(v) => v.chars().next().ok_or(()),
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for String
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok("null".to_string()),
			Value::Bool(v) => Ok(if v {"true".to_string()} else {"false".to_string()}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	let mut buffer = [0u8; 24];
				let mantissa = mantissa.numtoa(10, &mut buffer);
				let len = mantissa.len();
				if exponent >= 0
				{	let e = exponent as usize;
					if len+e <= FORMAT_NUM_WIDTH
					{	// append zeroes according to exponent
						let mut vec = if !is_negative
						{	Vec::with_capacity(mantissa.len() + e)
						}
						else
						{	let mut vec = Vec::with_capacity(mantissa.len() + e + 1);
							vec.push(b'-');
							vec
						};
						vec.extend_from_slice(mantissa);
						vec.extend_from_slice(&FORMAT_NUM_WIDTH_Z[0 .. e]);
						return String::from_utf8(vec).map_err(|_| ());
					}
				}
				else
				{	let e = exponent.wrapping_neg() as usize;
					if e < len
					{	// insert dot in the middle of number
						let mut vec = if !is_negative
						{	Vec::with_capacity(mantissa.len() + 1)
						}
						else
						{	let mut vec = Vec::with_capacity(mantissa.len() + 2);
							vec.push(b'-');
							vec
						};
						vec.extend_from_slice(&mantissa[0 .. len-e]);
						vec.push(b'.');
						vec.extend_from_slice(&mantissa[len-e ..]);
						return String::from_utf8(vec).map_err(|_| ());
					}
					if e <= FORMAT_NUM_WIDTH
					{	// prepend with 0.000...
						let mut vec = if !is_negative
						{	Vec::with_capacity(mantissa.len() + e-len+2)
						}
						else
						{	let mut vec = Vec::with_capacity(mantissa.len() + e-len+3);
							vec.push(b'-');
							vec
						};
						vec.extend_from_slice(&FORMAT_NUM_WIDTH_0Z[0 .. e-len+2]);
						vec.extend_from_slice(mantissa);
						return String::from_utf8(vec).map_err(|_| ());
					}
				}
				let mut buffer = [0u8; 24];
				let exponent = exponent.numtoa(10, &mut buffer);
				let mut vec = if !is_negative
				{	Vec::with_capacity(mantissa.len() + exponent.len() + 1)
				}
				else
				{	let mut vec = Vec::with_capacity(mantissa.len() + exponent.len() + 2);
					vec.push(b'-');
					vec
				};
				vec.extend_from_slice(mantissa);
				vec.push(b'e');
				vec.extend_from_slice(exponent);
				String::from_utf8(vec).map_err(|_| ())
			},
			Value::String(v) => Ok(v),
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

/*impl<T> TryFrom<Value> for Vec<T> where T: TryFrom<Value>
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(Vec::new()),
			Value::Bool(v) => Err(()),
			Value::Number(mantissa, _exponent, _is_negative) => Err(()),
			Value::String(_v) => Err(()),
			Value::Array(v) => Ok(v),
			Value::Object(_v) => Err(()),
		}
	}
}*/

// 2. To value

macro_rules! impl_from_value_signed
{	($ty:ty) =>
	{	impl TryFrom<$ty> for Value
		{	type Error = ();

			fn try_from(value: $ty) -> Result<Self, Self::Error>
			{	if value >= 0
				{	Ok(Value::Number(value.try_into().map_err(|_| ())?, 0, false))
				}
				else
				{	Ok(Value::Number(i64::try_from(value).map_err(|_| ())?.wrapping_neg() as u64, 0, true))
				}
			}
		}
	}
}

macro_rules! impl_from_value_unsigned
{	($ty:ty) =>
	{	impl TryFrom<$ty> for Value
		{	type Error = ();

			fn try_from(value: $ty) -> Result<Self, Self::Error>
			{	Ok(Value::Number(value.try_into().map_err(|_| ())?, 0, false))
			}
		}
	}
}

impl_from_value_signed!(isize);
impl_from_value_signed!(i64);
impl_from_value_signed!(i32);
impl_from_value_signed!(i16);
impl_from_value_signed!(i8);
impl_from_value_unsigned!(usize);
impl_from_value_unsigned!(u64);
impl_from_value_unsigned!(u32);
impl_from_value_unsigned!(u16);
impl_from_value_unsigned!(u8);

impl TryFrom<()> for Value
{	type Error = ();

	fn try_from(_value: ()) -> Result<Self, Self::Error>
	{	Ok(Value::Null)
	}
}

impl TryFrom<bool> for Value
{	type Error = ();

	fn try_from(value: bool) -> Result<Self, Self::Error>
	{	Ok(Value::Bool(value))
	}
}

impl TryFrom<char> for Value
{	type Error = ();

	fn try_from(value: char) -> Result<Self, Self::Error>
	{	Ok(Value::String(value.to_string()))
	}
}

impl TryFrom<String> for Value
{	type Error = ();

	fn try_from(value: String) -> Result<Self, Self::Error>
	{	Ok(Value::String(value))
	}
}

impl FromStr for Value
{	type Err = ();

	fn from_str(value: &str) -> Result<Self, Self::Err>
	{	Ok(Value::String(value.to_string()))
	}
}

impl TryFrom<Vec<Value>> for Value
{	type Error = ();

	fn try_from(value: Vec<Value>) -> Result<Self, Self::Error>
	{	Ok(Value::Array(value))
	}
}

macro_rules! impl_from_vec
{	($ty:ty) =>
	{	impl TryFrom<Vec<$ty>> for Value
		{	type Error = ();

			fn try_from(value: Vec<$ty>) -> Result<Self, Self::Error>
			{	let mut vec = Vec::with_capacity(value.len());
				for v in value
				{	vec.push(Value::try_from(v)?);
				}
				Ok(Value::Array(vec))
			}
		}
	}
}

macro_rules! impl_from_linked_list
{	($ty:ty) =>
	{	impl TryFrom<LinkedList<$ty>> for Value
		{	type Error = ();

			fn try_from(value: LinkedList<$ty>) -> Result<Self, Self::Error>
			{	let mut vec = Vec::with_capacity(value.len());
				for v in value
				{	vec.push(Value::try_from(v)?);
				}
				Ok(Value::Array(vec))
			}
		}
	}
}

macro_rules! impl_from_vec_deque
{	($ty:ty) =>
	{	impl TryFrom<VecDeque<$ty>> for Value
		{	type Error = ();

			fn try_from(value: VecDeque<$ty>) -> Result<Self, Self::Error>
			{	let mut vec = Vec::with_capacity(value.len());
				for v in value
				{	vec.push(Value::try_from(v)?);
				}
				Ok(Value::Array(vec))
			}
		}
	}
}

impl_from_vec!(isize);
impl_from_vec!(i64);
impl_from_vec!(i32);
impl_from_vec!(i16);
impl_from_vec!(i8);
impl_from_vec!(usize);
impl_from_vec!(u64);
impl_from_vec!(u32);
impl_from_vec!(u16);
impl_from_vec!(u8);
impl_from_vec!(bool);
impl_from_vec!(char);
impl_from_vec!(());
impl_from_vec!(String);

impl_from_linked_list!(isize);
impl_from_linked_list!(i64);
impl_from_linked_list!(i32);
impl_from_linked_list!(i16);
impl_from_linked_list!(i8);
impl_from_linked_list!(usize);
impl_from_linked_list!(u64);
impl_from_linked_list!(u32);
impl_from_linked_list!(u16);
impl_from_linked_list!(u8);
impl_from_linked_list!(bool);
impl_from_linked_list!(char);
impl_from_linked_list!(());
impl_from_linked_list!(String);

impl_from_vec_deque!(isize);
impl_from_vec_deque!(i64);
impl_from_vec_deque!(i32);
impl_from_vec_deque!(i16);
impl_from_vec_deque!(i8);
impl_from_vec_deque!(usize);
impl_from_vec_deque!(u64);
impl_from_vec_deque!(u32);
impl_from_vec_deque!(u16);
impl_from_vec_deque!(u8);
impl_from_vec_deque!(bool);
impl_from_vec_deque!(char);
impl_from_vec_deque!(());
impl_from_vec_deque!(String);


impl TryFrom<HashMap<String, Value>> for Value
{	type Error = ();

	fn try_from(value: HashMap<String, Value>) -> Result<Self, Self::Error>
	{	Ok(Value::Object(value))
	}
}

macro_rules! impl_from_hash_map
{	($ty:ty) =>
	{	impl TryFrom<HashMap<String, $ty>> for Value
		{	type Error = ();

			fn try_from(value: HashMap<String, $ty>) -> Result<Self, Self::Error>
			{	let mut obj = HashMap::with_capacity(value.len());
				for (key, v) in value
				{	obj.insert(key, Value::try_from(v)?);
				}
				Ok(Value::Object(obj))
			}
		}
	}
}

macro_rules! impl_from_btree_map
{	($ty:ty) =>
	{	impl TryFrom<BTreeMap<String, $ty>> for Value
		{	type Error = ();

			fn try_from(value: BTreeMap<String, $ty>) -> Result<Self, Self::Error>
			{	let mut obj = HashMap::with_capacity(value.len());
				for (key, v) in value
				{	obj.insert(key, Value::try_from(v)?);
				}
				Ok(Value::Object(obj))
			}
		}
	}
}

impl_from_hash_map!(isize);
impl_from_hash_map!(i64);
impl_from_hash_map!(i32);
impl_from_hash_map!(i16);
impl_from_hash_map!(i8);
impl_from_hash_map!(usize);
impl_from_hash_map!(u64);
impl_from_hash_map!(u32);
impl_from_hash_map!(u16);
impl_from_hash_map!(u8);
impl_from_hash_map!(bool);
impl_from_hash_map!(char);
impl_from_hash_map!(());
impl_from_hash_map!(String);

impl_from_btree_map!(isize);
impl_from_btree_map!(i64);
impl_from_btree_map!(i32);
impl_from_btree_map!(i16);
impl_from_btree_map!(i8);
impl_from_btree_map!(usize);
impl_from_btree_map!(u64);
impl_from_btree_map!(u32);
impl_from_btree_map!(u16);
impl_from_btree_map!(u8);
impl_from_btree_map!(bool);
impl_from_btree_map!(char);
impl_from_btree_map!(());
impl_from_btree_map!(String);


impl<'a> Index<&'a str> for Value
{	type Output = Value;

	fn index(&self, index: &'a str) -> &Self::Output
	{	match *self
		{	Value::Null => &Value::Null,
			Value::Bool(ref _v) => &Value::Null,
			Value::Number(ref _mantissa, ref _exponent, ref _is_negative) => &Value::Null,
			Value::String(ref _v) => &Value::Null,
			Value::Array(ref _v) => &Value::Null,
			Value::Object(ref v) =>
			{	v.get(index).unwrap_or(&Value::Null)
			}
		}
	}
}
