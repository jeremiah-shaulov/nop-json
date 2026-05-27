//! ReaderBuilder and the configurable parsing limits (depth_limit, value_size_limit).

use nop_json::{Reader, ReaderBuilder, Value, TryFromJson, ValidateJson, DebugToJson};

fn nested_array(depth: usize) -> String
{	let mut s = String::with_capacity(depth * 2 + 1);
	for _ in 0 .. depth {s.push('[')}
	s.push('1');
	for _ in 0 .. depth {s.push(']')}
	s
}

#[test]
fn builder_builds_a_working_reader()
{	let mut reader = ReaderBuilder::new().build(r#" [1, 2, 3] "#.bytes());
	assert_eq!(reader.read::<Vec<i32>>().unwrap(), vec![1, 2, 3]);
}

#[test]
fn builder_is_chainable_and_copy()
{	let builder = ReaderBuilder::new().depth_limit(10).value_size_limit(1000);
	// Copy: the builder can be reused
	let mut r1 = builder.build("[1]".bytes());
	let mut r2 = builder.build("[2]".bytes());
	assert_eq!(r1.read::<Vec<i32>>().unwrap(), vec![1]);
	assert_eq!(r2.read::<Vec<i32>>().unwrap(), vec![2]);
}

#[test]
fn depth_limit_boundary()
{	// depth_limit(N): nesting exactly N is accepted, N+1 is rejected
	let build = |json: &str| ReaderBuilder::new().depth_limit(3).build(json.bytes()).read::<Value>();
	assert!(build(&nested_array(1)).is_ok());
	assert!(build(&nested_array(3)).is_ok());
	assert!(build(&nested_array(4)).is_err());
}

#[test]
fn depth_limit_rejects_deep_input_without_crashing()
{	// Previously this overflowed the stack; now it must return an error with the default limit.
	let deep = nested_array(200_000);
	let err = Reader::new(deep.bytes()).read::<Value>().unwrap_err();
	assert!(err.to_string().contains("too deep"), "{err}");
}

#[test]
fn depth_limit_applies_to_objects()
{	let nested_obj = |depth: usize| -> String
	{	let mut s = String::new();
		for _ in 0 .. depth {s.push_str("{\"a\":")}
		s.push('1');
		for _ in 0 .. depth {s.push('}')}
		s
	};
	assert!(ReaderBuilder::new().depth_limit(5).build(nested_obj(5).bytes()).read::<Value>().is_ok());
	assert!(ReaderBuilder::new().depth_limit(5).build(nested_obj(6).bytes()).read::<Value>().is_err());
}

#[test]
fn depth_limit_applies_to_typed_vec_and_skip()
{	// reading into Vec<Vec<...>> goes through read_array
	let deep = nested_array(50);
	assert!(ReaderBuilder::new().depth_limit(10).build(deep.bytes()).read::<Value>().is_err());

	// skipping (read into ()) also honors the limit
	let deep = nested_array(50);
	let res: std::io::Result<()> = ReaderBuilder::new().depth_limit(10).build(deep.bytes()).read();
	assert!(res.is_err());
}

#[test]
fn depth_counter_resets_between_values()
{	// each value is within the limit; reading several in a row must not accumulate depth
	let mut reader = ReaderBuilder::new().depth_limit(3).build("[[1]] [[2]] [[3]]".bytes());
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "[[1]]");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "[[2]]");
	assert_eq!(reader.read::<Value>().unwrap().to_string(), "[[3]]");
}

#[test]
fn empty_containers_do_not_leak_depth()
{	// empty arrays/objects use a fast path for the closing bracket; depth must still balance
	let mut reader = ReaderBuilder::new().depth_limit(2).build("[] {} [[]] []".bytes());
	assert!(reader.read::<Value>().is_ok());
	assert!(reader.read::<Value>().is_ok());
	assert!(reader.read::<Value>().is_ok()); // [[]] is depth 2, still within limit
	assert!(reader.read::<Value>().is_ok());
}

#[test]
fn value_size_limit_on_string()
{	let json = format!("\"{}\"", "a".repeat(100));
	assert!(ReaderBuilder::new().value_size_limit(10).build(json.bytes()).read::<String>().is_err());
	assert!(ReaderBuilder::new().value_size_limit(1000).build(json.bytes()).read::<String>().is_ok());

	// a string at/under the limit is accepted
	let small = format!("\"{}\"", "x".repeat(10));
	assert_eq!(ReaderBuilder::new().value_size_limit(10).build(small.bytes()).read::<String>().unwrap(), "x".repeat(10));
}

#[test]
fn value_size_limit_on_blob()
{	let mut json = vec![b'"'];
	json.extend(std::iter::repeat(b'x').take(100));
	json.push(b'"');
	let err = ReaderBuilder::new().value_size_limit(16).build(json.into_iter()).read_blob().unwrap_err();
	assert!(err.to_string().contains("too large"), "{err}");
}

#[test]
fn default_limits_allow_normal_input()
{	// sanity: a derived struct with reasonable nesting parses fine with defaults
	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Inner {v: i32}
	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Outer {items: Vec<Inner>, name: String}

	let o: Outer = Reader::new(r#"{"items": [{"v": 1}, {"v": 2}], "name": "ok"}"#.bytes()).read().unwrap();
	assert_eq!(o, Outer {items: vec![Inner {v: 1}, Inner {v: 2}], name: "ok".to_string()});
}
