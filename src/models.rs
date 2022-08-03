#[derive(Debug)]
pub struct Mood {
    id: i32,
    timestamp: u32,
    value: f32,
    message: Option<String>,
}
