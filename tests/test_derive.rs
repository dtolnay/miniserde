use miniserde::{json, Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum Tag {
    A,
    #[serde(rename = "renamedB")]
    B,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Example {
    x: String,
    t1: Tag,
    t2: Tag,
    n: Nested,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[test]
fn test_de() {
    let j = r#" {"x": "X", "t1": "A", "t2": "renamedB", "n": {"y": ["Y", "Y"]}} "#;
    let actual: Example = json::from_str(j).unwrap();
    let expected = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Tag::B,
        n: Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        },
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_ser() {
    let example = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Tag::B,
        n: Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        },
    };
    let actual = json::to_string(&example);
    let expected = r#"{"x":"X","t1":"A","t2":"renamedB","n":{"y":["Y","Y"],"z":null}}"#;
    assert_eq!(actual, expected);
}
