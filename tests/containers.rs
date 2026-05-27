//! Deserialization of the standard container/wrapper types, plus serialize -> read round-trips.

use nop_json::{Reader, DebugToJson};
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, LinkedList, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::Rc;

fn read<T: nop_json::TryFromJson>(json: &str) -> std::io::Result<T>
{	Reader::new(json.bytes()).read()
}

#[test]
fn vectors()
{	assert_eq!(read::<Vec<i32>>("[1, 2, 3]").unwrap(), vec![1, 2, 3]);
	assert_eq!(read::<Vec<String>>(r#"["a", "b"]"#).unwrap(), vec!["a".to_string(), "b".to_string()]);
	assert_eq!(read::<Vec<Vec<i32>>>("[[1], [2, 3], []]").unwrap(), vec![vec![1], vec![2, 3], vec![]]);
	assert_eq!(read::<Vec<i32>>("[]").unwrap(), Vec::<i32>::new());
	// null reads as an empty collection
	assert_eq!(read::<Vec<i32>>("null").unwrap(), Vec::<i32>::new());
}

#[test]
fn sets()
{	let hs: HashSet<i32> = read("[1, 2, 3, 2, 1]").unwrap();
	assert_eq!(hs, [1, 2, 3].iter().copied().collect());

	let bs: BTreeSet<i32> = read("[3, 1, 2, 1]").unwrap();
	assert_eq!(bs, [1, 2, 3].iter().copied().collect());
}

#[test]
fn lists_and_deques()
{	let ll: LinkedList<i32> = read("[1, 2, 3]").unwrap();
	assert_eq!(ll.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);

	let dq: VecDeque<i32> = read("[1, 2, 3]").unwrap();
	assert_eq!(dq.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
}

#[test]
fn maps()
{	let hm: HashMap<String, i32> = read(r#"{"a": 1, "b": 2}"#).unwrap();
	assert_eq!(hm.get("a"), Some(&1));
	assert_eq!(hm.get("b"), Some(&2));
	assert_eq!(hm.len(), 2);

	let bm: BTreeMap<String, i32> = read(r#"{"x": 10, "y": 20}"#).unwrap();
	assert_eq!(bm.into_iter().collect::<Vec<_>>(), vec![("x".to_string(), 10), ("y".to_string(), 20)]);

	let empty: HashMap<String, i32> = read("{}").unwrap();
	assert!(empty.is_empty());
}

#[test]
fn nested_map_of_vec()
{	let m: BTreeMap<String, Vec<i32>> = read(r#"{"a": [1, 2], "b": []}"#).unwrap();
	assert_eq!(m.get("a"), Some(&vec![1, 2]));
	assert_eq!(m.get("b"), Some(&vec![]));
}

#[test]
fn tuples()
{	assert_eq!(read::<(i32, String)>(r#"[1, "a"]"#).unwrap(), (1, "a".to_string()));
	assert_eq!(read::<(i32, i32, i32)>("[1, 2, 3]").unwrap(), (1, 2, 3));
	// wrong arity
	assert!(read::<(i32, i32)>("[1]").is_err());
	assert!(read::<(i32, i32)>("[1, 2, 3]").is_err());
	assert!(read::<(i32, i32)>("5").is_err());
}

#[test]
fn options()
{	assert_eq!(read::<Option<i32>>("5").unwrap(), Some(5));
	assert_eq!(read::<Option<i32>>("null").unwrap(), None);
	assert_eq!(read::<Option<String>>(r#""hi""#).unwrap(), Some("hi".to_string()));
	assert_eq!(read::<Vec<Option<i32>>>("[1, null, 3]").unwrap(), vec![Some(1), None, Some(3)]);
}

#[test]
fn smart_pointers()
{	assert_eq!(*read::<Box<i32>>("7").unwrap(), 7);
	assert_eq!(*read::<Rc<i32>>("7").unwrap(), 7);
	assert_eq!(*read::<Arc<i32>>("7").unwrap(), 7);
	assert_eq!(*read::<RwLock<i32>>("7").unwrap().read().unwrap(), 7);
	assert_eq!(*read::<Mutex<i32>>("7").unwrap().lock().unwrap(), 7);
}

#[test]
fn round_trip_vec_and_maps()
{	let original = vec![1, 2, 3, 100, -5];
	let json = original.to_json_string();
	assert_eq!(read::<Vec<i32>>(&json).unwrap(), original);

	let mut m = BTreeMap::new();
	m.insert("alpha".to_string(), vec![1, 2]);
	m.insert("beta".to_string(), vec![]);
	let json = m.to_json_string();
	assert_eq!(read::<BTreeMap<String, Vec<i32>>>(&json).unwrap(), m);

	let hs: HashSet<i32> = [5, 9, 13].iter().copied().collect();
	let json = hs.to_json_string();
	assert_eq!(read::<HashSet<i32>>(&json).unwrap(), hs);
}

#[test]
fn wrong_container_type_errors()
{	assert!(read::<Vec<i32>>(r#"{"a": 1}"#).is_err());     // object, not array
	assert!(read::<HashMap<String, i32>>("[1, 2]").is_err()); // array, not object
	assert!(read::<Vec<i32>>("true").is_err());
}
