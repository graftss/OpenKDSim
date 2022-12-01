#[derive(Debug)]
pub struct CameraParams {
    /// (??) A duration when the camera is zooming out.
    /// offset: 0x7a0b4
    pub scale_up_duration_long: i32,

    /// (??) A duration when the camera is zooming out.
    /// offset: 0x7a0b8
    pub scale_up_duration_short: i32,

    /// (??) The initial timer when changing to Shoot mode.
    pub shoot_timer_init: u16,

    /// (??) The initial timer when changing to Shootret mode.
    pub shoot_ret_timer_init: u16,

    /// (??)
    /// offset: 0xd345e8
    pub param_0xd345e8: f32,

    /// (??)
    /// offset: 0xd345ec
    pub param_0xd345ec: f32,

    /// A factor controlling how much the "special camera" move closer
    /// *laterally* to the katamari.
    pub special_camera_tighten: f32,

    /// The vertical speed of the L1 look camera.
    /// offset: 0x7a058
    pub l1_look_speed_y: f32,

    /// The horizontal speed of the L1 look camera.
    /// offset: 0x7a05c
    pub l1_look_speed_x: f32,

    /// The max y angle of the L1 look camera.
    /// offset: 0x7a050
    pub l1_look_max_y: f32,

    /// The min y angle of the L1 look camera.
    /// offset: 0x7a054
    pub l1_look_min_y: f32,

    /// (??) Presumably meant to be x-axis delay on camera movement, but that doesn't seem to be right.
    /// offset: 0xb23e0
    pub delay_x: f32,

    /// (??) Presumably meant to be y-axis delay on camera movement, but that doesn't seem to be right.
    /// offset: 0xb23e4
    pub delay_y: f32,

    /// (??) Presumably meant to be z-axis delay on camera movement, but that doesn't seem to be right.
    /// offset: 0xb23e8
    pub delay_z: f32,
}

impl Default for CameraParams {
    fn default() -> Self {
        Self {
            scale_up_duration_long: 100,
            scale_up_duration_short: 60,
            shoot_timer_init: 0x3c,
            shoot_ret_timer_init: 0x14,
            param_0xd345e8: f32::from_bits(0xff027d4b),
            param_0xd345ec: 0.0,
            special_camera_tighten: 0.75,
            l1_look_speed_x: f32::from_bits(0x3d0ef998),
            l1_look_speed_y: f32::from_bits(0x3cc82be9),
            l1_look_max_y: f32::from_bits(0x3f860a7c),
            l1_look_min_y: f32::from_bits(0x3e71465f),
            delay_x: 1.0,
            delay_y: 1.0,
            delay_z: 1.0,
        }
    }
}
