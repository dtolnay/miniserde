#![allow(clippy::assertions_on_result_states)]

use miniserde::{json, Deserialize};

#[derive(Deserialize)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[test]
fn main() {
    let result = json::from_str::<Point>(r#"{"x": 1, "y": 2, "z": 3}"#);
    assert!(result.is_ok());
}
