use miniserde::Serialize;

#[derive(Serialize)]
union Union {
    x: i32,
}

fn main() {}
