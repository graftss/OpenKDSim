use std::fmt::Display;

use gl_matrix::common::{Vec3, Vec4};

use crate::{
    katamari::mesh::KatMesh,
    mission::{GameMode, Mission},
    stage::Stage,
};

/// Miscellaneous global game state.
#[derive(Debug, Default)]
pub struct GlobalState {
    /// Set to true after `MonoInitStart` is called.
    /// offset: 0xff0f0
    pub did_init_start: bool,

    /// The player whose state is being updated.
    /// offset: 0xff0f4
    pub updating_player: u8,

    /// The current mission.
    /// offset: 0xff104
    pub mission: Option<Mission>,

    /// The *unique* id of the currently loaded area among all other areas in the game.
    /// The House stage has ids 0-4, Town has 5-8, and World has 9+.
    /// offset: 0xff106
    pub stage_area: u8,

    /// The current stage (which is the map - house, town, world, etc.)
    /// offset: 0xff108
    pub stage: Option<Stage>,

    /// The current loaded area of the current stage, where the smallest
    /// area of each stage is 0.
    /// offset: 0xff109
    pub area: Option<u32>,

    /// If true, the current mission is in VS mode.
    /// offset: 0xff0f1
    pub is_vs_mode: bool,

    /// The number of ticks before the mission timer expires.
    /// offset: 0xff120
    pub remain_time_ticks: i32,

    /// The current game time (in *real time*, not ticks).
    /// offset: 0xff12c
    pub game_time_ms: i32,

    /// The current game mode.
    pub gamemode: Option<GameMode>,

    /// (??) Set by `SetStoreFlag`.
    /// offset: 0x10dab8
    pub kat_diam_int_on_store_flag: i32,

    /// If true, ticking the physics engine has no effect (it's "frozen").
    /// offset: 0x10daea
    pub freeze: bool,

    /// (??) true when map is being changed (i.e. during a new area load).
    /// Presumably nothing should be moving while this is on.
    /// offset: 0x10daec
    pub map_change_mode: bool,

    /// (??) Set to true after `MonoInitEnd` is finished to signify that all
    /// props have been initialized.
    /// offset: 0x10daed
    pub props_initialized: bool,

    /// (??) too lazy to document this right now
    /// offset: 0x10daf9
    pub vs_mission_idx: u32,

    /// The number of ticks that have been completed.
    /// offset: 0x10ea50
    pub ticks: u32,

    /// (??) Set by `SetStoreFlag`.
    /// offset: 0x10eace
    pub store_flag: bool,

    /// If true, the simulation is currently in the process of detaching
    /// props from the katamari. (It's set to true while happening, and set back
    /// to false after everything has been detached.)
    /// offset: 0x10eadb
    pub detaching_props_from_kat: bool,

    /// All builtin katamari meshes. The game only uses the mesh at index 1.
    /// offset: 0x10eae8
    pub katamari_meshes: Vec<KatMesh>,

    /// Props with a diameter ratio to the player less
    /// than this value will be destroyed when they reach an alpha of 0.
    /// offset: 0x1339fc
    pub prop_diam_ratio_destroy_when_invis: f32,

    /// The number of loaded theme props.
    /// offset: 0x153198
    pub num_theme_props: u16,

    /// The number of loaded twin props.
    /// offset: 0x155290
    pub num_twin_props: u16,

    /// The number of loaded tree root props.
    /// offset: 0x155294
    pub num_root_props: u16,

    /// The control index of the next prop
    /// offset: 0xd35325
    pub next_ctrl_idx: u16,

    /// The "theme object" score in constellation levels (e.g. number of crabs in Make Cancer).
    pub catch_count_b: i32,

    /// Global forward movement speed multiplier.
    pub forwards_speed: f32,

    /// Global sideways movement speed multiplier.
    pub sideways_speed: f32,

    /// Global backwards movement speed multiplier.
    pub backwards_speed: f32,

    /// Global boost movement speed multiplier.
    pub boost_speed: f32,

    /// Global forward movement acceleration multiplier.
    pub forwards_accel: f32,

    /// Global sideways movement acceleration multiplier.
    pub sideways_accel: f32,

    /// Global backwards movement acceleration multiplier.
    pub backwards_accel: f32,

    /// Global boost movement acceleration multiplier.
    pub boost_accel: f32,

    /// Global multiplier on the speed the prince rotates around the katamari.
    pub prince_turn_speed_mult: f32,

    /// (??) Global camera delay along x, y, and z axes.
    pub camera_delay: Vec3,

    /// The y-coordinate of the death plane. Anything below this value should be warped.
    pub royal_warp_plane_y: f32,

    /// Gravity direction.
    pub gravity: Vec4,

    /// Negative gravity direction.
    pub neg_gravity: Vec4,

    /// (??)
    /// offset: 0xd35580
    pub map_loop_rate: f32,
}

impl Display for GlobalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl GlobalState {
    pub fn set_speeds(
        &mut self,
        forw_s: f32,
        side_s: f32,
        back_s: f32,
        boost_s: f32,
        forw_a: f32,
        side_a: f32,
        back_a: f32,
        boost_a: f32,
        rot_s: f32,
        dp_y: f32,
        cam_x: f32,
        cam_y: f32,
        cam_z: f32,
    ) {
        self.forwards_speed = forw_s;
        self.sideways_speed = side_s;
        self.backwards_speed = back_s;
        self.boost_speed = boost_s;

        self.forwards_accel = forw_a;
        self.sideways_accel = side_a;
        self.backwards_accel = back_a;
        self.boost_accel = boost_a;

        self.prince_turn_speed_mult = rot_s;
        self.royal_warp_plane_y = dp_y;

        self.camera_delay[0] = cam_x;
        self.camera_delay[1] = cam_y;
        self.camera_delay[2] = cam_z;
    }

    pub fn set_gravity(&mut self, x: f32, y: f32, z: f32) {
        self.gravity[0] = x;
        self.gravity[1] = y;
        self.gravity[2] = z;

        self.neg_gravity[0] = -x;
        self.neg_gravity[1] = -y;
        self.neg_gravity[2] = -z;
    }

    pub fn set_gamemode(&mut self, gamemode: u32) {
        self.gamemode = Some(gamemode.into());
    }

    pub fn mono_init_start(&mut self, mission: u32, area: u32, stage: u32) {
        self.stage = Some(stage.into());
        self.did_init_start = true;

        self.mission = Some(mission.into());
        self.area = Some(area);
        self.stage = Some(stage.into());

        self.num_theme_props = 0;
        self.num_twin_props = 0;
        self.num_root_props = 0;
        self.next_ctrl_idx = 0;
    }

    pub fn get_next_ctrl_idx(&mut self) -> u16 {
        let result = self.next_ctrl_idx;
        self.next_ctrl_idx += 1;
        result
    }
}
