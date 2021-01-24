pub use nop_json_derive::*;
use crate::debug_to_json::DebugToJson;
use crate::value::Value;
use crate::nop_json::{escape};

use std::{char, fmt, f32, f64, io};
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, LinkedList, VecDeque};
use std::sync::{RwLock, Mutex, Arc};
use std::rc::Rc;


pub trait WriteToJson<W: io::Write>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>;
}

fn write_debug_to_json<W, T>(out: &mut W, value: &T) -> io::Result<()> where T: DebugToJson, W: io::Write
{	struct Wrapper<'a, T: DebugToJson>
	{	value: &'a T
	}
	impl<'a, T: DebugToJson> fmt::Display for Wrapper<'a, T>
	{	fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result
		{	DebugToJson::fmt(self.value, out)
		}
	}
	let w = Wrapper {value};
	write!(out, "{}", w)
}

impl<W: io::Write> WriteToJson<W> for ()     {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for isize  {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for i128   {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for i64    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for i32    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for i16    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for i8     {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for usize  {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for u128   {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for u64    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for u32    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for u16    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for u8     {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for f64    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for f32    {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for bool   {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for char   {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for String {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write> WriteToJson<W> for Value  {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}

impl<W: io::Write, T> WriteToJson<W> for Box<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let v: &T = &*self;
		v.write_to_json(out)
	}
}

impl<W: io::Write, T> WriteToJson<W> for RwLock<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	match self.read()
		{	Ok(v) => v.write_to_json(out),
			Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
		}
	}
}

impl<W: io::Write, T> WriteToJson<W> for Mutex<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	match self.lock()
		{	Ok(v) => v.write_to_json(out),
			Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
		}
	}
}

impl<W: io::Write, T> WriteToJson<W> for Rc<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let v: &T = &*self;
		v.write_to_json(out)
	}
}

impl<W: io::Write, T> WriteToJson<W> for Arc<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let v: &T = &*self;
		v.write_to_json(out)
	}
}

impl<W: io::Write, T> WriteToJson<W> for Option<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	match *self
		{	Some(ref v) => v.write_to_json(out),
			None => write!(out, "null"),
		}
	}
}

impl<W: io::Write, T> WriteToJson<W> for Vec<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			item.write_to_json(out)?;
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

impl<W: io::Write, T> WriteToJson<W> for HashSet<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			item.write_to_json(out)?;
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

impl<W: io::Write, T> WriteToJson<W> for LinkedList<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			item.write_to_json(out)?;
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

impl<W: io::Write, T> WriteToJson<W> for VecDeque<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			item.write_to_json(out)?;
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

impl<W: io::Write, T> WriteToJson<W> for BTreeSet<T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let mut c = '[';
		for item in self
		{	write!(out, "{}", c)?;
			item.write_to_json(out)?;
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

impl<W: io::Write, T> WriteToJson<W> for HashMap<String, T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let mut c = '{';
		for (key, item) in self
		{	write!(out, "{}\"{}\":", c, escape(key))?;
			item.write_to_json(out)?;
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

impl<W: io::Write, T> WriteToJson<W> for BTreeMap<String, T> where T: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	let mut c = '{';
		for (key, item) in self
		{	write!(out, "{}\"{}\":", c, escape(key))?;
			item.write_to_json(out)?;
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

impl<W: io::Write, A, B> WriteToJson<W> for (A, B) where A: WriteToJson<W>, B: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	write!(out, "[")?;
		self.0.write_to_json(out)?;
		write!(out, ",")?;
		self.1.write_to_json(out)?;
		write!(out, "]")
	}
}

impl<W: io::Write, A, B, C> WriteToJson<W> for (A, B, C) where A: WriteToJson<W>, B: WriteToJson<W>, C: WriteToJson<W>
{	fn write_to_json(&self, out: &mut W) -> io::Result<()>
	{	write!(out, "[")?;
		self.0.write_to_json(out)?;
		write!(out, ",")?;
		self.1.write_to_json(out)?;
		write!(out, ",")?;
		self.2.write_to_json(out)?;
		write!(out, "]")
	}
}
