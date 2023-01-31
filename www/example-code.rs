use serde_json;

fn gav(x: i32, y: i32) -> i64 {
    (x - y) * (x + y)
}

fn main() {
    let x = serde_json::from_str::<i32>("{}").unwrap();
}
