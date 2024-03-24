use miniserde::{json, Deserialize};

#[derive(Deserialize, PartialEq, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[test]
fn main() {
    let actual = json::from_str::<Point>(r#"{"x": 1, "y": 2, "z": 3}"#).unwrap();
    let expected = Point { x: 1, y: 2 };
    assert_eq!(actual, expected);
}
