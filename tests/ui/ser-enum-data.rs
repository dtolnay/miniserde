use miniserde::Serialize;

#[derive(Serialize)]
enum Enum {
    Variant(i32)
}

fn main() {}
