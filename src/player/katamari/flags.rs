use super::collision::ray::{KatCollisionRayType, ShellRay};

/// The direction the katamari is moving relative to the slope of
/// the surface it's moving on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KatInclineMoveType {
    MoveFlatground,
    MoveUphill,
    MoveDownhill,
}

impl Default for KatInclineMoveType {
    fn default() -> Self {
        Self::MoveFlatground
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KatPhysicsFlags {
    /// If true, the katamari has no surface contacts.
    /// offset: 0x0
    pub airborne: bool,

    /// If true, the katamari is climbing a wall.
    /// offset: 0x1
    pub climbing_wall: bool,

    /// If true, the katamari is at its maximum climb height (so it can't climb higher).
    /// offset: 0x2
    pub at_max_climb_height: bool,

    /// If true, the katamari is braking.
    /// offset: 0x3
    pub braking: bool,

    /// If true, the katamari just bonked something (only true the frame it bonks).
    /// offset: 0x4
    pub bonked: bool,

    /// If true, the katamari is contacting a wall.
    /// offset: 0x5
    pub contacts_wall: bool,

    /// If true, the katamari is contacting a wall.
    /// offset: 0x6
    pub contacts_floor: bool,

    /// If true, the katamari is in water.
    /// offset: 0x7
    pub in_water: bool,

    /// (??) copy of `in_water`
    /// offset: 0x8
    pub in_water_copy: bool,

    /// (??) If true, the katamari was hit by a moving prop.
    /// offset: 0x9
    pub hit_by_moving_prop: bool,

    /// (??) If true, the katamari is contacting a prop.
    /// offset: 0xa
    pub contacts_prop_0xa: bool,

    /// (??) If true, the katamari is contacting the bottom of a downward-slanting surface.
    /// (e.g. can be triggered under mas1 table by setting simulation+0x71614 to 3, which
    /// changes the definition of how downward-slanting such a surface needs to be)
    /// offset: 0xb
    pub contacts_down_slanted_ceiling: bool,

    /// (??) If true, the katamari moved more than its own radius during the last tick.
    /// offset: 0xc
    pub moved_more_than_rad_0xc: bool,

    /// If true, the katamari is contacting a prop.
    /// offset: 0xd
    pub contacts_prop_0xd: bool,

    /// (??) A shell ray which collided with something
    /// offset: 0xe
    pub hit_shell_ray: Option<ShellRay>,

    /// If true, the katamari is completely submerged underwater.
    /// offset: 0xf
    pub under_water: bool,

    /// If true, the katamari is not moving.
    /// offset: 0x10
    pub immobile: bool,

    /// (??) The type of boundary ray currently acting as the pivot.
    /// offset: 0x11
    pub vault_ray_type: Option<KatCollisionRayType>,

    /// If true, the katamari is contacting a non-flat floor (normal < 0.9999).
    /// offset: 0x12
    pub on_sloped_floor: bool,

    /// If true, the katamari is contacting a flat floor (normal >= 0.9999).
    /// offset: 0x13
    pub on_flat_floor: bool,

    /// (??)
    /// offset: 0x14
    pub moved_too_much_0x14: bool,

    /// (??)
    /// offset: 0x15
    pub incline_move_type: KatInclineMoveType,

    /// If true, the katamari is spinning
    /// offset: 0x16
    pub wheel_spin: bool,

    /// True if not boosting AND input below the "min push" threshold.
    /// offset: 0x17
    pub no_input_push: bool,

    /// True if the katamari moved more than its radius on the previous tick.
    /// offset: 0x19
    pub moved_more_than_rad: bool,

    /// True if the katamari is considered stuck between walls.
    /// offset: 0x1a
    pub stuck_between_walls: bool,

    /// (??)
    /// offset: 0x1b
    pub is_colliding_0x1b: bool,

    /// True if the katamari should emit the "puff of smoke" vfx as it moves.
    /// By default this occurs when it's over 12m in the World stage.
    /// offset: 0x1c
    pub should_emit_smoke: bool,

    /// (??)
    /// offset: 0x1d
    pub moved_more_than_rad_0x1d: bool,

    /// (??)
    /// offset: 0x1e
    pub vs_attack: u8,

    /// (??)
    /// offset: 0x1f
    pub vs_mode_some_state: u8,
}

impl KatPhysicsFlags {
    /// Reset some flags at the start of `Katamari::update_collision`
    pub fn reset_for_collision_update(&mut self) {
        self.contacts_down_slanted_ceiling = false;
        self.moved_more_than_rad_0xc = false;
        self.contacts_prop_0xd = false;
        self.hit_shell_ray = None;
        self.moved_more_than_rad_0x1d = false;
        self.hit_by_moving_prop = false;
        self.contacts_prop_0xa = false;
        self.stuck_between_walls = false;
        self.is_colliding_0x1b = false;
        self.contacts_wall = false;
        self.contacts_floor = false;
    }
}

/// A group of flags which mostly record if the katamari is contacting certain special types of surfaces
/// with non-standard `HitAttribute` values.
#[derive(Debug, Default, Clone, Copy)]
pub struct KatHitFlags {
    /// If true, ignores "pushing downward" incline effect (e.g. on park entrance steps)
    /// offset: 0x0
    pub force_flatground: bool,

    /// (??) True while climbing a prop, and also certain surfaces e.g. park steps.
    /// offset: 0x1
    pub wall_climb_free: bool,

    /// (??)
    /// offset: 0x2
    pub small_ledge_climb: bool,

    /// True when speed should be uncapped while moving downhill (e.g. big hill in Town stage)
    /// offset: 0x3
    pub speed_check_off: bool,

    /// (??)
    /// offset: 0x4
    pub flag_0x4: bool,

    /// (??)
    /// offset: 0x5
    pub flag_0x5: bool,

    /// True when the camera should be zoomed in (e.g. under House stage table, under trees outside World park).
    /// offset: 0x6
    pub special_camera: bool,

    /// (??) Applies when contacting a "NoReactionNoSlope" surface
    /// offset: 0x7
    pub no_reaction_no_slope: bool,

    /// True if the "turntable" spin effect should be applied.
    /// offset: 0x8
    pub on_turntable: bool,

    /// True when contacting a "WallClimbDisabled" surface.
    /// If true, climbing is disabled. (e.g. the legs under the mas1 table)
    /// offset: 0x9
    pub wall_climb_disabled: bool,

    /// (??) True when contacting a "MapSemiTranslucent" surface.
    /// offset: 0xa
    pub map_semi_translucent: bool,
}