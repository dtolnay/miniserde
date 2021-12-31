use crate::json::Value;
use alloc::vec::Vec;

pub fn safely(value: Value) {
    match value {
        Value::Array(_) | Value::Object(_) => {}
        _ => return,
    }

    let mut stack = Vec::new();
    stack.push(value);
    while let Some(value) = stack.pop() {
        match value {
            Value::Array(vec) => {
                for child in vec {
                    stack.push(child);
                }
            }
            Value::Object(map) => {
                for (_, child) in map {
                    stack.push(child);
                }
            }
            _ => {}
        }
    }
}
