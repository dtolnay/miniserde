use miniserde::Deserialize;

#[derive(Deserialize)]
union Union {
    x: i32,
}

fn main() {}
