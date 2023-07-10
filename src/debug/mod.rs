pub mod draw;

pub struct DebugConfig {
    pub log_tick: bool,

    // TODO_DEBUG: make this distinguish between katamari collision rays and prop rays, once
    // the latter are added.
    /// Draw points where collision rays intersect with the bounding boxes of props.
    pub draw_collided_prop_aabb_hits: bool,

    /// Draw the meshes of collided props.
    pub draw_collided_prop_mesh: bool,

    /// TODO_DOC
    pub draw_collided_prop_tris: bool,

    /// Draw katamari collision rays.
    pub kat_draw_collision_rays: bool,

    /// Draw katamari shell rays.
    pub kat_draw_shell_rays: bool,
}

pub const DEBUG_CONFIG: DebugConfig = DebugConfig {
    log_tick: true,
    draw_collided_prop_aabb_hits: false,
    draw_collided_prop_mesh: false,
    draw_collided_prop_tris: false,
    kat_draw_collision_rays: true,
    kat_draw_shell_rays: true,
};
