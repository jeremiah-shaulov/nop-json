//! `#[derive(TryFromJson, ValidateJson, DebugToJson)]` for structs and enums:
//! field renaming, exclusion, ignoring, enum variant selection, validation, nesting and round-trips.

use nop_json::{Reader, TryFromJson, ValidateJson, DebugToJson};
use std::io;

fn read<T: TryFromJson>(json: &str) -> io::Result<T>
{	Reader::new(json.bytes()).read()
}

#[test]
fn struct_basic_round_trip()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Point {x: i32, y: i32}

	let p: Point = read(r#"{"x": 3, "y": 4}"#).unwrap();
	assert_eq!(p, Point {x: 3, y: 4});

	let json = p.to_json_string();
	assert_eq!(json, r#"{"x":3,"y":4}"#);
	assert_eq!(read::<Point>(&json).unwrap(), p);
}

#[test]
fn struct_field_rename()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct User
	{	id: usize,
		#[json(all_posts)] posts: Vec<i32>,
	}
	let u: User = read(r#"{"id": 1, "all_posts": [10, 20]}"#).unwrap();
	assert_eq!(u, User {id: 1, posts: vec![10, 20]});
	// serialized back with the renamed key
	assert_eq!(u.to_json_string(), r#"{"id":1,"all_posts":[10,20]}"#);
}

#[test]
fn struct_excluded_field_uses_default()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Point {x: i32, y: i32, #[json("")] comments: String}

	// "comments" in the input is ignored because the field is excluded with #[json("")]
	let p: Point = read(r#"{"x": 0, "y": 0, "comments": "hello"}"#).unwrap();
	assert_eq!(p, Point {x: 0, y: 0, comments: String::new()});

	let p: Point = read(r#"{"x": 1, "y": 2}"#).unwrap();
	assert_eq!(p, Point {x: 1, y: 2, comments: String::new()});
}

#[test]
fn struct_unknown_field_is_error_by_default()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Point {x: i32, y: i32}
	assert!(read::<Point>(r#"{"x": 0, "y": 1, "z": 2}"#).is_err());
}

#[test]
fn struct_ignore_all_unknown()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	#[json_ignore]
	struct Point {x: i32, y: i32}
	let p: Point = read(r#"{"x": 0, "y": 1, "comments": "no comments", "extra": [1,2,3]}"#).unwrap();
	assert_eq!(p, Point {x: 0, y: 1});
}

#[test]
fn struct_ignore_specific()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	#[json_ignore(comments)]
	struct Point {x: i32, y: i32}
	assert!(read::<Point>(r#"{"x": 0, "y": 1, "comments": "ok"}"#).is_ok());
	// a different unknown field is still an error
	assert!(read::<Point>(r#"{"x": 0, "y": 1, "other": "no"}"#).is_err());
}

#[test]
fn struct_missing_fields_use_default()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Config {a: i32, b: i32, c: i32}
	let c: Config = read(r#"{"b": 5}"#).unwrap();
	assert_eq!(c, Config {a: 0, b: 5, c: 0});
}

#[test]
fn enum_typed_discriminator()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Point {x: i32, y: i32}

	#[derive(PartialEq, TryFromJson, ValidateJson, DebugToJson)]
	#[json(type)]
	enum Geometry
	{	#[json(point)] Point(Point),
		#[json(cx, cy, r)] Circle(i32, i32, i32),
		Nothing,
	}

	let g: Geometry = read(r#"{"type": "Point", "point": {"x": 1, "y": 2}}"#).unwrap();
	assert_eq!(g, Geometry::Point(Point {x: 1, y: 2}));

	let g: Geometry = read(r#"{"type": "Circle", "cx": 1, "cy": 2, "r": 3}"#).unwrap();
	assert_eq!(g, Geometry::Circle(1, 2, 3));

	let g: Geometry = read(r#"{"type": "Nothing"}"#).unwrap();
	assert_eq!(g, Geometry::Nothing);

	// round trip
	let g = Geometry::Circle(1, 2, 3);
	assert_eq!(read::<Geometry>(&g.to_json_string()).unwrap(), g);
}

#[test]
fn enum_renamed_variant()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Point {x: i32, y: i32}

	#[derive(PartialEq, TryFromJson, ValidateJson, DebugToJson)]
	#[json(var)]
	enum Geometry
	{	#[json(pnt(point))] Point(Point),
		#[json(cir(cx, cy, r))] Circle(i32, i32, i32),
		Nothing,
	}
	let g: Geometry = read(r#"{"var": "pnt", "point": {"x": 0, "y": 0}}"#).unwrap();
	assert_eq!(g, Geometry::Point(Point {x: 0, y: 0}));

	let g: Geometry = read(r#"{"var": "cir", "cx": 1, "cy": 2, "r": 3}"#).unwrap();
	assert_eq!(g, Geometry::Circle(1, 2, 3));
}

#[test]
fn enum_by_content_no_discriminator()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Point {x: i32, y: i32}

	#[derive(PartialEq, TryFromJson, ValidateJson, DebugToJson)]
	enum Geometry
	{	#[json(point)] Point(Point),
		#[json(cx, cy, r)] Circle(i32, i32, i32),
		Nothing,
	}
	let g: Geometry = read(r#"{"point": {"x": 5, "y": 6}}"#).unwrap();
	assert_eq!(g, Geometry::Point(Point {x: 5, y: 6}));

	let g: Geometry = read(r#"{"cx": 1, "cy": 2, "r": 3}"#).unwrap();
	assert_eq!(g, Geometry::Circle(1, 2, 3));
}

#[test]
fn enum_ignore_lists()
{	#[derive(PartialEq, TryFromJson, ValidateJson, DebugToJson)]
	#[json(type)]
	#[json_ignore(comments)]
	enum Geometry
	{	#[json(point(x, y))]
		#[json_ignore(point_comments)]
		Point(i32, i32),

		#[json(circle(cx, cy, r))]
		#[json_ignore(circle_comments)]
		Circle(i32, i32, i32),
	}
	let g: Geometry = read(r#"{"type": "point", "x": 0, "y": 1, "comments": "c", "point_comments": "pc"}"#).unwrap();
	assert_eq!(g, Geometry::Point(0, 1));

	// circle_comments is not valid for the Point variant
	assert!(read::<Geometry>(r#"{"type": "point", "x": 0, "y": 1, "circle_comments": "x"}"#).is_err());
}

#[test]
fn validate_json_custom()
{	#[derive(PartialEq, Default, TryFromJson, DebugToJson)]
	struct FromTo {from: i32, to: i32}
	impl ValidateJson for FromTo
	{	fn validate_json(self) -> Result<Self, String>
		{	if self.from <= self.to {Ok(self)} else {Err("to must be >= from".to_string())}
		}
	}
	assert!(read::<FromTo>(r#"{"from": 0, "to": 10}"#).is_ok());
	let err = read::<FromTo>(r#"{"from": 3, "to": -1}"#).unwrap_err();
	assert!(err.to_string().contains("to must be >= from"), "{err}");
}

#[test]
fn nested_structures_round_trip()
{	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Post {id: usize, title: String, is_published: bool}

	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct User {id: usize, name: String, posts: Vec<Post>}

	let input = r#"{"id": 1, "name": "John", "posts": [{"id": 1, "title": "Hello", "is_published": true}, {"id": 2, "title": "Subj", "is_published": false}]}"#;
	let u: User = read(input).unwrap();
	assert_eq!
	(	u,
		User
		{	id: 1,
			name: "John".to_string(),
			posts: vec!
			[	Post {id: 1, title: "Hello".to_string(), is_published: true},
				Post {id: 2, title: "Subj".to_string(), is_published: false},
			],
		}
	);
	// round trip through serialization
	assert_eq!(read::<User>(&u.to_json_string()).unwrap(), u);
}

#[test]
fn numeric_string_coercion_in_struct()
{	// the "id" values are JSON strings but the field is usize
	#[derive(PartialEq, Default, TryFromJson, ValidateJson, DebugToJson)]
	struct Item {id: usize, name: String}
	let it: Item = read(r#"{"id": "42", "name": "widget"}"#).unwrap();
	assert_eq!(it, Item {id: 42, name: "widget".to_string()});
}
