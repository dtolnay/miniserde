use miniserde::{json, MiniSerialize, MiniDeserialize};

#[derive(PartialEq, Debug, MiniSerialize, MiniDeserialize)]
struct Example {
    x: String,
    n: Nested,
}

#[derive(PartialEq, Debug, MiniSerialize, MiniDeserialize)]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[test]
fn test_de() {
    let j = r#" {"x": "X", "n": {"y": ["Y", "Y"]}} "#;
    let actual: Example = json::from_str(j).unwrap();
    let expected = Example {
        x: "X".to_owned(),
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
        n: Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        },
    };
    let actual = json::to_string(&example);
    let expected = r#"{"x":"X","n":{"y":["Y","Y"],"z":null}}"#;
    assert_eq!(actual, expected);
}
