#![allow(dead_code)]
use miniserde::{json, Deserialize};

#[derive(Deserialize)]
struct Point {
    x: u32,
    y: u32,
}

#[test]
fn main() {
    let result = json::from_str::<Point>(r#"{"x": 1, "y": 2, "z": 3}"#);
    assert!(result.is_ok());
}
