#![allow(clippy::uninlined_format_args)]

use indoc::indoc;
use miniserde::json::{self, Value};

#[test]
fn test_round_trip_deeply_nested() {
    let depth = if cfg!(miri) { 40 } else { 100_000 };

    let mut j = String::new();
    for _ in 0..depth {
        j.push_str("{\"x\":[");
    }
    for _ in 0..depth {
        j.push_str("]}");
    }

    let value: Value = json::from_str(&j).unwrap();
    let j2 = json::to_string(&value);
    assert_eq!(j, j2);
}

#[test]
fn test_debug() {
    let j = r#"
        {
            "Null": null,
            "Bool": true,
            "Number": 1,
            "String": "...",
            "Array": [true],
            "EmptyArray": [],
            "EmptyObject": {}
        }
    "#;

    let value: Value = json::from_str(j).unwrap();
    let debug = format!("{:#?}", value);

    let expected = indoc! {r#"
        Object {
            "Array": Array [
                Bool(true),
            ],
            "Bool": Bool(true),
            "EmptyArray": Array [],
            "EmptyObject": Object {},
            "Null": Null,
            "Number": Number(1),
            "String": String("..."),
        }"#
    };

    assert_eq!(debug, expected);
}
