use crate::macros::panic_log;

#[derive(Debug)]
pub struct BoostGachaParam {
    /// The minimum diameter at which this gacha value applies.
    pub min_diam_cm: f32,

    /// The number of post-spin gachas needed to boost.
    pub num_gachas: u8,
}

#[derive(Debug)]
pub struct PrinceParams {
    /// The maximum ratio of the katamari's max speed that allows the prince to
    /// enter a non-normal view mode (l1 look or r1 jump).
    /// offset: 0x7b258
    pub max_speed_for_view_mode: f32,

    /// The number of gachas needed to spin.
    pub prince_gachas_for_spin: u8,

    /// The number of post-spin gachas needed to boost, which varies by size.
    /// offset: 0x9ead8
    pub prince_extra_gachas_for_boost: Vec<BoostGachaParam>,

    /// (??) TODO
    pub prince_roll_forwards_angle_threshold: f32,

    /// Global multiplier on the prince's turn speed around the katamari.
    /// Set by the API function `SetKatamariSpeed`.
    /// default: 1.0
    /// offset: 0x7acf4
    pub global_turn_speed_mult: f32,

    /// (??) If the angle between the stick is greater than this, the prince is not considered
    /// to be pushing the katamari.
    /// default: pi/6
    /// offset: 0x715c4
    pub max_angle_btwn_sticks_for_push: f32,

    /// The maximum speed multiplier applied from push magnitude.
    /// default: 0x4
    /// offset: 0x7b240
    pub run_speed_mult: f32,

    /// The minimum push magnitude needed to jog. Below this magnitude, the prince walks.
    /// default: 0.29
    /// offset: 0x7b244
    pub jog_min_push_mag: f32,

    /// The minimum push magnitude needed to jog. Below this magnitude, the prince jogs or walks.
    /// default: 0.99
    /// offset: 0x7b248
    pub run_min_push_mag: f32,

    /// The minimum `input_avg_push_len` value to admit a wallclimb.
    /// default: 0.95
    /// offset: 0x715f8
    pub wallclimb_min_avg_push_len: f32,

    /// The angle threshold which admits wallclimbs. The angle 0 is straight up, and the
    /// *absolute value* of the input sum's angle must be below this value to admit a wallclimb.
    /// default: pi/18 ~~ 0.1745329
    /// offset: 0x71590 (used at 0x167ce)
    pub wallclimb_input_sum_angle_threshold: f32,
}

impl Default for PrinceParams {
    fn default() -> Self {
        Self {
            max_speed_for_view_mode: f32::from_bits(0x3f59999a),

            prince_gachas_for_spin: 3,

            prince_extra_gachas_for_boost: vec![
                BoostGachaParam {
                    min_diam_cm: 0.0,
                    num_gachas: 2,
                },
                BoostGachaParam {
                    min_diam_cm: 50.0,
                    num_gachas: 3,
                },
                BoostGachaParam {
                    min_diam_cm: 500.0,
                    num_gachas: 4,
                },
                BoostGachaParam {
                    min_diam_cm: 1000.0,
                    num_gachas: 5,
                },
            ],

            prince_roll_forwards_angle_threshold: f32::from_bits(0x3f060a92), // 0.5235988
            global_turn_speed_mult: 1.0,
            max_angle_btwn_sticks_for_push: f32::from_bits(0x3f060a92), // pi/6
            jog_min_push_mag: f32::from_bits(0x3e947ae1),               // 0.24
            run_min_push_mag: f32::from_bits(0x3f7d70a4),               // 0.99
            run_speed_mult: f32::from_bits(0x3ecccccd),                 // 0.4
            wallclimb_min_avg_push_len: f32::from_bits(0x3f733333),     // 0.95
            wallclimb_input_sum_angle_threshold: f32::from_bits(0x3e32b8c2), // 0.1745329 (or pi/18)
        }
    }
}

const GACHAS_FOR_BOOST_PANIC_STR: &'static str =
    "panic in `gachas_for_boost`: invalid structure of `extra_gachas_for_boost`";

impl PrinceParams {
    /// Compute the number of gachas needed to boost
    pub fn gachas_for_boost(&self, diam_cm: f32) -> u8 {
        for (i, param) in self.prince_extra_gachas_for_boost.iter().enumerate() {
            if param.min_diam_cm > diam_cm {
                return match i {
                    0 => {
                        panic_log!("{}", GACHAS_FOR_BOOST_PANIC_STR);
                    }
                    _ => {
                        self.prince_gachas_for_spin
                            + self.prince_extra_gachas_for_boost[i - 1].num_gachas
                    }
                };
            }
        }

        panic_log!("{}", GACHAS_FOR_BOOST_PANIC_STR);
    }

    /// Compute a multiplier on speed applied when pushing the katamari based on the
    /// magnitude of the push. Used in `Katamari::update_velocity`.
    /// offset: 0x22644 (in `kat_update_velocity`)
    pub fn push_mag_speed_mult(&self, push_mag: f32, pre_speed: f32) -> f32 {
        if push_mag < self.jog_min_push_mag {
            // case 1: prince is walking (`push_mag` in `[0, self.jog_min_push_mag]`)
            push_mag / self.jog_min_push_mag * self.run_speed_mult * pre_speed
        } else if push_mag < self.run_min_push_mag {
            // case 2: prince is jogging (`push_mag` in `[self.jog_min_push_mag, self.run_min_push_mag]`)
            pre_speed * self.run_speed_mult
        } else {
            // case 3: prince is running (`push_mag` in `[self.run_min_push_mag, 1]`)
            // TODO_REFACTOR: translate using an inv lerp macro
            let lerped = (1.0
                - ((push_mag - self.run_min_push_mag) / (1.0 - self.run_min_push_mag))
                    * self.run_speed_mult)
                + self.run_speed_mult;
            pre_speed * lerped
        }
    }
}
