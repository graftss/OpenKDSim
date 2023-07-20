pub mod draw;

pub struct DebugConfig {
    /// If `false`, the `crate::util::debug_log` function becomes a no-op.
    pub allow_debug_logs: bool,

    /// Writes `tick` to the debug log every frame.
    pub log_tick: bool,

    /// Writes a log whenever a triangle with a nonzero hit attribute contacts the katamari.
    pub log_nonzero_hit_attribute_hits: bool,

    /// Writes a log whenever a prop is destroyed via the `Prop::prop_destroy` method.
    pub log_destroyed_props: bool,

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

// TODO_DEBUG: this should be made editable mid-execution.
pub const DEBUG_CONFIG: DebugConfig = DebugConfig {
    allow_debug_logs: true,
    log_tick: false,
    log_nonzero_hit_attribute_hits: false,
    log_destroyed_props: true,
    draw_collided_prop_aabb_hits: true,
    draw_collided_prop_mesh: true,
    draw_collided_prop_tris: true,
    kat_draw_collision_rays: true,
    kat_draw_shell_rays: true,
};
