#![allow(clippy::assertions_on_result_states)]

use miniserde::json;

#[test]
fn main() {
    let result = json::from_str::<bool>(" true && false ");
    assert!(result.is_err());
}
