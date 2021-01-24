pub use nop_json_derive::*;
use crate::debug_to_json::DebugToJson;
use crate::value::Value;

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

impl<W: io::Write, T> WriteToJson<W> for Box<T>              where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for RwLock<T>           where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for Mutex<T>            where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for Rc<T>               where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for Arc<T>              where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for Option<T>           where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for Vec<T>              where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for HashSet<T>          where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for LinkedList<T>       where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for VecDeque<T>         where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for BTreeSet<T>         where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for HashMap<String, T>  where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T> WriteToJson<W> for BTreeMap<String, T> where T: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}

impl<W: io::Write, T, U> WriteToJson<W> for (T, U)           where T: DebugToJson, U: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
impl<W: io::Write, T, U, V> WriteToJson<W> for (T, U, V)     where T: DebugToJson, U: DebugToJson, V: DebugToJson {fn write_to_json(&self, out: &mut W) -> io::Result<()> {write_debug_to_json(out, self)}}
