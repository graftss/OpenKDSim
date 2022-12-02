use crate::{
    macros::{inv_lerp, lerp},
    mission::config::KatScaledParamsCtrlPt,
    player::prince::PushDir,
};

/// Katamari parameters which vary based on the katamari's current size.
#[derive(Debug, Default, Clone, Copy)]
pub struct KatScaledParams {
    /// Base max speed which acts as a multiplier on the speeds of all movement types.
    /// offset: 0x0
    pub base_max_speed: f32,

    /// Downward acceleration from gravity.
    /// offset: 0x4
    pub accel_grav: f32,

    /// (??) The force exerted when braking with forwards movement.
    /// offset: 0x8
    pub brake_forwards_force: f32,

    /// (??) The force exerted when braking with backwards movement.
    /// offset: 0xc
    pub brake_backwards_force: f32,

    /// (??) The force exerted when braking with sideways movement.
    /// offset: 0x10
    pub brake_sideways_force: f32,

    /// (??) The force exerted when braking boost movement.
    /// offset: 0x14
    pub brake_boost_force: f32,

    /// Max speed with forwards movement.
    /// offset: 0x18
    pub max_forwards_speed: f32,

    /// Max speed with backwards movement.
    /// offset: 0x1c
    pub max_backwards_speed: f32,

    /// Max speed with sideways movement.
    /// offset: 0x20
    pub max_sideways_speed: f32,

    /// Max speed with boost movement.
    /// offset: 0x24
    pub max_boost_speed: f32,

    /// (??)
    /// offset: 0x28
    pub push_uphill_accel: f32,

    /// (??)
    /// offset: 0x2c
    pub not_push_uphill_accel: f32,

    /// Acceleration during forwards movement.
    /// offset: 0x30
    pub forwards_accel: f32,

    /// Acceleration during backwards movement.
    /// offset: 0x34
    pub backwards_accel: f32,

    /// Acceleration during sideways movement.
    /// offset: 0x38
    pub sideways_accel: f32,

    /// Acceleration during boost movement.
    /// offset: 0x3c
    pub boost_accel: f32,

    /// The prince's lateral distance from the katamari center.
    /// offset: 0x40
    pub prince_offset: f32,

    /// (??)
    /// offset: 0x44
    pub alarm_closest_range: f32,

    /// (??)
    /// offset: 0x48
    pub alarm_closer_range: f32,

    /// (??)
    /// offset: 0x4c
    pub alarm_close_range: f32,
}

macro_rules! lerp_param {
    ($self: ident, $min: ident, $max: ident, $t: ident, $param: ident) => {
        $self.$param = lerp!($t, $min.params.$param, $max.params.$param);
    };
}

impl KatScaledParams {
    /// Interpolates the values of this struct between the param control points
    /// `min` and `max` using the katamari size `diam_cm`.
    pub fn interpolate_from(
        &mut self,
        diam_cm: f32,
        min_pt: &KatScaledParamsCtrlPt,
        max_pt: &KatScaledParamsCtrlPt,
    ) {
        if diam_cm <= min_pt.diam_cm {
            // if the katamari is smaller than the min control point size, just
            // copy the min control point exactly.
            self.clone_from(&min_pt.params);
        } else if diam_cm >= max_pt.diam_cm {
            // likewise if the katamari is larger than the max control point size.
            self.clone_from(&max_pt.params);
        } else {
            let t = inv_lerp!(diam_cm, min_pt.diam_cm, max_pt.diam_cm);
            lerp_param!(self, min_pt, max_pt, t, base_max_speed);
            lerp_param!(self, min_pt, max_pt, t, accel_grav);
            lerp_param!(self, min_pt, max_pt, t, brake_forwards_force);
            lerp_param!(self, min_pt, max_pt, t, brake_backwards_force);
            lerp_param!(self, min_pt, max_pt, t, brake_sideways_force);
            lerp_param!(self, min_pt, max_pt, t, brake_boost_force);
            lerp_param!(self, min_pt, max_pt, t, max_forwards_speed);
            lerp_param!(self, min_pt, max_pt, t, max_backwards_speed);
            lerp_param!(self, min_pt, max_pt, t, max_sideways_speed);
            lerp_param!(self, min_pt, max_pt, t, max_boost_speed);
            lerp_param!(self, min_pt, max_pt, t, push_uphill_accel);
            lerp_param!(self, min_pt, max_pt, t, not_push_uphill_accel);
            lerp_param!(self, min_pt, max_pt, t, forwards_accel);
            lerp_param!(self, min_pt, max_pt, t, backwards_accel);
            lerp_param!(self, min_pt, max_pt, t, sideways_accel);
            lerp_param!(self, min_pt, max_pt, t, boost_accel);
            lerp_param!(self, min_pt, max_pt, t, prince_offset);
            lerp_param!(self, min_pt, max_pt, t, alarm_closest_range);
            lerp_param!(self, min_pt, max_pt, t, alarm_closer_range);
            lerp_param!(self, min_pt, max_pt, t, alarm_close_range);
        }
    }

    pub fn get_push_accel(&self, push_dir: PushDir) -> f32 {
        match push_dir {
            PushDir::Forwards => self.forwards_accel,
            PushDir::Backwards => self.sideways_accel,
            PushDir::Sideways => self.backwards_accel,
        }
    }

    pub fn get_push_max_speed(&self, push_dir: PushDir) -> f32 {
        match push_dir {
            PushDir::Forwards => self.max_forwards_speed,
            PushDir::Backwards => self.max_backwards_speed,
            PushDir::Sideways => self.max_sideways_speed,
        }
    }
}
