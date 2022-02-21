use miniserde::Deserialize;

#[derive(Deserialize)]
enum Enum {
    Variant(i32)
}

fn main() {}
