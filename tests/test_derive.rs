#![allow(clippy::derive_partial_eq_without_eq)]

use miniserde::{json, Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum Tag {
    A,
    #[serde(rename = "renamedB")]
    B,
    #[allow(non_camel_case_types)]
    r#enum,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Example {
    x: String,
    t1: Tag,
    t2: Box<Tag>,
    t3: [Tag; 1],
    r#struct: Box<Nested>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[test]
fn test_de() {
    let j =
        r#" {"x": "X", "t1": "A", "t2": "renamedB", "t3": ["enum"], "struct": {"y": ["Y", "Y"]}} "#;
    let actual: Example = json::from_str(j).unwrap();
    let expected = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        t3: [Tag::r#enum],
        r#struct: Box::new(Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        }),
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_ser() {
    let example = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        t3: [Tag::r#enum],
        r#struct: Box::new(Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        }),
    };
    let actual = json::to_string(&example);
    let expected =
        r#"{"x":"X","t1":"A","t2":"renamedB","t3":["enum"],"struct":{"y":["Y","Y"],"z":null}}"#;
    assert_eq!(actual, expected);
}
