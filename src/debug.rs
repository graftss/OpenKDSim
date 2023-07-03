#[repr(C)]
pub enum DebugDrawType {
    Line = 0,
    Box = 1,
}

impl From<DebugDrawType> for i32 {
    fn from(value: DebugDrawType) -> Self {
        match value {
            DebugDrawType::Line => 0,
            DebugDrawType::Box => 1,
        }
    }
}
