extern crate miniserde;

use miniserde::json::{self, Value};

#[test]
fn test_round_trip_deeply_nested() {
    let mut j = String::new();
    for _ in 0..100_000 {
        j.push_str("{\"x\":[");
    }
    for _ in 0..100_000 {
        j.push_str("]}");
    }

    let value: Value = json::from_str(&j).unwrap();
    let j2 = json::to_string(&value);
    assert_eq!(j, j2);
}
