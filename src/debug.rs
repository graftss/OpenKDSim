/// The draw commands performed by the `debug_draw` delegate.
#[repr(C)]
pub enum DebugDrawType {
    Line = 0,
    Box = 1,
    Point = 2,
}

impl From<DebugDrawType> for i32 {
    fn from(value: DebugDrawType) -> Self {
        match value {
            DebugDrawType::Line => 0,
            DebugDrawType::Box => 1,
            DebugDrawType::Point => 2,
        }
    }
}

pub struct DebugConfig {
    pub log_tick: bool,

    /// Draw points where the katamari's collision rays intersect
    /// the bounding boxes of props.
    pub kat_draw_prop_aabb_collision: bool,

    /// Draw katamari collision rays.
    pub kat_draw_collision_rays: bool,

    /// Draw katamari shell rays.
    pub kat_draw_shell_rays: bool,
}

pub const DEBUG_CONFIG: DebugConfig = DebugConfig {
    log_tick: true,

    kat_draw_prop_aabb_collision: false,
    kat_draw_collision_rays: true,
    kat_draw_shell_rays: true,
};
