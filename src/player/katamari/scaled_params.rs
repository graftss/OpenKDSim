/// Katamari parameters which vary based on the katamari's current size.
#[derive(Debug, Default, Clone, Copy)]
pub struct KatScaledParams {
    /// Base max speed which acts as a multiplier on the speeds of all movement types.
    pub base_max_speed: f32,

    /// Downward acceleration from gravity.
    pub gravity_accel: f32,

    /// (??) The force exerted when braking with forwards movement.
    pub brake_forwards_force: f32,

    /// (??) The force exerted when braking with backwards movement.
    pub brake_backwards_force: f32,

    /// (??) The force exerted when braking with sideways movement.
    pub brake_sideways_force: f32,

    /// (??) The force exerted when braking boost movement.
    pub brake_boost_force: f32,

    /// Max speed with forwards movement.
    pub max_forwards_speed: f32,

    /// Max speed with backwards movement.
    pub max_backwards_speed: f32,

    /// Max speed with sideways movement.
    pub max_sideways_speed: f32,

    /// Max speed with boost movement.
    pub max_boost_speed: f32,

    /// (??)
    pub push_uphill_accel: f32,

    /// (??)
    pub not_push_uphill_accel: f32,

    /// Acceleration during forwards movement.
    pub forwards_accel: f32,

    /// Acceleration during backwards movement.
    pub backwards_accel: f32,

    /// Acceleration during sideways movement.
    pub sideways_accel: f32,

    /// Acceleration during boost movement.
    pub boost_accel: f32,

    /// The prince's lateral distance from the katamari center.
    pub prince_offset: f32,

    /// (??)
    pub alarm_closest_range: f32,

    /// (??)
    pub alarm_closer_range: f32,

    /// (??)
    pub alarm_close_range: f32,
}

impl KatScaledParams {
    pub fn update(
        &mut self,
        min_diam: f32,
        min_params: &KatScaledParams,
        max_diam: f32,
        max_params: &KatScaledParams,
    ) {
    }
}
