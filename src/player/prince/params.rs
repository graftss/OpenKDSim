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
}