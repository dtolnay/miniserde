use miniserde::Serialize;

#[derive(Serialize)]
struct Struct {
    #[serde(rename = "A", rename = "B")]
    x: i32,
}

fn main() {}
