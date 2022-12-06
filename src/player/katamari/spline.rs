use lazy_static::lazy_static;

use crate::{
    macros::{inv_lerp, panic_log, temp_debug_log},
    util::spline::compute_spline_point,
};

use super::Katamari;

const SPLINE_CTRL_PTS_DATA: [[f32; 2]; 11] = [
    [0.0, 0.184],
    [0.074, 0.185],
    [0.164, 0.2],
    [0.29, 0.238],
    [0.407, 0.324],
    [0.538, 0.466],
    [0.645, 0.592],
    [0.761, 0.735],
    [0.867, 0.872],
    [0.94, 0.967],
    [1.0, 1.0],
];

pub struct SplineCtrlPt {
    pub max_speed_ratio: f32,
    pub value: f32,
}

impl SplineCtrlPt {
    fn from_float_pairs(raw_data: &[[f32; 2]]) -> Vec<SplineCtrlPt> {
        let mut result = vec![];

        for pair in raw_data {
            result.push(SplineCtrlPt {
                max_speed_ratio: pair[0],
                value: pair[1],
            });
        }

        result
    }
}

lazy_static! {
    static ref SPLINE_CTRL_PTS: Vec<SplineCtrlPt> =
        SplineCtrlPt::from_float_pairs(&SPLINE_CTRL_PTS_DATA);
}

/// Apply a spline easing to remap the player's `max_speed_ratio` to something else.
/// (because reasons)
/// offset: 0x232d0
pub fn compute_spline_accel_mult(max_speed_ratio: f32) -> f32 {
    return 1.0;

    // TODO: go back over this more carefully

    // find the index of the first control point whose `max_speed_ratio` field is
    // larger than the player's `max_speed_ratio` passed to this call.
    let mut first_idx = 0;
    if max_speed_ratio == 1.0 {
        first_idx = 10;
    } else {
        for (i, ctrl_pt) in SPLINE_CTRL_PTS.iter().enumerate() {
            if max_speed_ratio < ctrl_pt.max_speed_ratio {
                first_idx = i;
                break;
            }
        }
    }

    if first_idx == 0 {
        panic_log!("`compute_spline_accel_mut` error: `max_speed_ratio`={max_speed_ratio}.");
    }
    temp_debug_log!("      max_speed_ratio={max_speed_ratio}, first_idx={first_idx}");

    let mut mat = [0.0; 16];
    mat[3] = 1.0;
    mat[7] = 1.0;
    mat[11] = 1.0;
    mat[15] = 1.0;

    if first_idx == 1 {
        mat[1] = SPLINE_CTRL_PTS[0].value;
        mat[5] = SPLINE_CTRL_PTS[0].value;
        mat[9] = SPLINE_CTRL_PTS[1].value;
        mat[13] = SPLINE_CTRL_PTS[2].value;
    } else {
        mat[1] = SPLINE_CTRL_PTS[first_idx - 2].value;
        mat[5] = SPLINE_CTRL_PTS[first_idx - 1].value;
        mat[9] = SPLINE_CTRL_PTS[first_idx].value;
        mat[13] = if first_idx < 10 {
            SPLINE_CTRL_PTS[first_idx + 1].value
        } else {
            SPLINE_CTRL_PTS[first_idx].value
        };
    }

    let t = inv_lerp!(
        max_speed_ratio,
        SPLINE_CTRL_PTS[first_idx - 1].max_speed_ratio,
        SPLINE_CTRL_PTS[first_idx].max_speed_ratio
    );

    compute_spline_point(&mat, t)[1]
}
