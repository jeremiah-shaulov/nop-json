use std::char;
use std::collections::HashMap;
use std::convert::TryFrom;

pub const VALUE_NUM_MANTISSA_BYTES: usize = 24;

macro_rules! get_num
{	($mantissa:expr, $exponent:expr, $is_negative:expr, $T:ty, $is_unsigned:expr) =>
	{	{	let mut buffer_len = $mantissa.len();
			if $is_unsigned && $is_negative
			{	return Err(());
			}
			if $exponent < 0
			{	let minus_exponent = $exponent.wrapping_neg() as usize;
				if minus_exponent >= buffer_len
				{	return Ok(0);
				}
				buffer_len -= minus_exponent;
			}
			let mut result = 0 as $T;
			if let Some(pos) = $mantissa.iter().position(|c| *c!=b'0')
			{	if pos < buffer_len
				{	result = ($mantissa[pos] - b'0') as $T;
					if !$is_unsigned && $is_negative
					{	result = result.wrapping_neg();
						for c in ($mantissa[pos+1 .. buffer_len]).iter()
						{	result = result.checked_mul(10).ok_or(())?;
							result = result.checked_sub((c - b'0') as $T).ok_or(())?;
						}
					}
					else
					{	for c in ($mantissa[pos+1 .. buffer_len]).iter()
						{	result = result.checked_mul(10).ok_or(())?;
							result = result.checked_add((c - b'0') as $T).ok_or(())?;
						}
					}
					if $exponent > 0
					{	result = result.checked_mul((10 as $T).checked_pow($exponent as u32).ok_or(())?).ok_or(())?
					}
				}
			}
			Ok(result)
		}
	}
}

// pub Value

#[derive(Debug, Clone, PartialEq)]
pub enum Value
{	Null, Bool(bool), Number([u8; VALUE_NUM_MANTISSA_BYTES], i32, bool), String(String), Array(Vec<Value>), Object(HashMap<String, Value>)
}

impl TryFrom<Value> for isize
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, isize, false)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for i128
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, i128, false)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for i64
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, i64, false)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for i32
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, i32, false)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for i16
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, i16, false)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for i8
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, i8, false)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for usize
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, usize, true)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for u128
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, u128, true)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for u64
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, u64, true)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for u32
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, u32, true)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for u16
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, u16, true)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for u8
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0),
			Value::Bool(v) => Ok(if v {1} else {0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	get_num!(mantissa, exponent, is_negative, u8, true)
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for f64
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0.0),
			Value::Bool(v) => Ok(if v {1.0} else {0.0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	if let Some(pos) = mantissa.iter().position(|c| *c!=b'0')
				{	let mut result = (mantissa[pos] - b'0') as f64;
					for c in (mantissa[pos+1 .. ]).iter()
					{	result *= 10.0;
						result += (c - b'0') as f64;
					}
					if is_negative
					{	result = -result;
					}
					result *= 10f64.powi(exponent);
					Ok(result)
				}
				else
				{	Ok(if is_negative {-0.0} else {0.0})
				}
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

impl TryFrom<Value> for f32
{	type Error = ();

	fn try_from(value: Value) -> Result<Self, Self::Error>
	{	match value
		{	Value::Null => Ok(0.0),
			Value::Bool(v) => Ok(if v {1.0} else {0.0}),
			Value::Number(mantissa, exponent, is_negative) =>
			{	if let Some(pos) = mantissa.iter().position(|c| *c!=b'0')
				{	let mut result = (mantissa[pos] - b'0') as f32;
					for c in (mantissa[pos+1 .. ]).iter()
					{	result *= 10.0;
						result += (c - b'0') as f32;
					}
					if is_negative
					{	result = -result;
					}
					result *= 10f32.powi(exponent);
					Ok(result)
				}
				else
				{	Ok(if is_negative {-0.0} else {0.0})
				}
			},
			Value::String(v) =>
			{	v.parse().map_err(|_| ())
			},
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}

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
			Value::Number(mantissa, _exponent, _is_negative) => Ok(mantissa.iter().position(|c| *c!=b'0').is_some()),
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
			Value::Number(mantissa, _exponent, _is_negative) =>
			{	if let Some(pos) = mantissa.iter().position(|c| *c!=b'0')
				{	Ok(mantissa[pos] as char)
				}
				else
				{	Ok('0')
				}
			},
			Value::String(v) => v.chars().next().ok_or(()),
			Value::Array(_v) => Err(()),
			Value::Object(_v) => Err(()),
		}
	}
}
