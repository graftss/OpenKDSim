use gl_matrix::vec3;

use crate::{
    macros::{set_y, vec3_from},
    math::{vec3_inplace_normalize, vec3_inplace_scale},
    player::katamari::Katamari,
    props::prop::Prop,
};

/// The "scale" of each bucket of props in the size chart (in the in-game collection).
/// offset: 0x68960
const SIZE_CHART_PROP_SCALES: [u8; 12] = [0, 0, 0, 1, 2, 3, 3, 4, 4, 4, 4, 4];

/// Fixed parameters common to all props of a given scale.
struct PropScaleConfig {
    /// offset: 0x0
    field_0x0: f32,

    /// offset: 0x4
    field_0x4: f32,

    /// offset: 0x8
    field_0x8: f32,

    /// offset: 0xc
    field_0xc: f32,
}

/// Fixed parameters for each prop scale.
/// offset: 0x7edf0
const PROP_SCALE_CONFIGS: [PropScaleConfig; 7] = [
    PropScaleConfig {
        field_0x0: 200.0,
        field_0x4: 300.0,
        field_0x8: 30.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        field_0x0: 300.0,
        field_0x4: 500.0,
        field_0x8: 50.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        field_0x0: 500.0,
        field_0x4: 700.0,
        field_0x8: 100.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        field_0x0: 700.0,
        field_0x4: 900.0,
        field_0x8: 200.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        field_0x0: 900.0,
        field_0x4: 1300.0,
        field_0x8: 300.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        field_0x0: 1500.0,
        field_0x4: 2000.0,
        field_0x8: 400.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        field_0x0: 1800.0,
        field_0x4: 2500.0,
        field_0x8: 500.0,
        field_0xc: 10.0,
    },
];

#[derive(Debug)]
enum DistComparison {
    Smaller = 0,
    Larger = 1,
}

/// Compares the lateral distance between `prop` and `katamari` to `dist_bound`.
/// Note that the katamari position used isn't the center: it's the closest point on
/// the katamari's "spherical surface" to the prop.
/// TODO_OPT: isn't the distance just `vec3::dist(&kat.center, &prop.pos) - kat.radius`?
/// offset: 0x364e0
fn compare_lateral_prop_kat_dist(
    prop: &Prop,
    katamari: &Katamari,
    comparison: DistComparison,
    dist_bound: f32,
) -> bool {
    let mut kat_center = katamari.get_center().clone();
    set_y!(kat_center, 0.0);

    let mut prop_pos = prop.pos.clone();
    set_y!(prop_pos, 0.0);

    // Compute the point on the surface of the katamari nearest the prop, `kat_pos`.
    // To do this, add (the katamari center) and (the radius towards the prop).
    let mut kat_radius_to_prop = vec3_from!(-, prop_pos, kat_center);
    vec3_inplace_normalize(&mut kat_radius_to_prop);
    vec3_inplace_scale(&mut kat_radius_to_prop, katamari.get_radius());

    let kat_pos = vec3_from!(+, kat_center, kat_radius_to_prop);
    assert!(kat_pos[1] == 0.0);

    let prop_to_kat = vec3_from!(-, kat_pos, prop_pos);
    let dist = vec3::len(&prop_to_kat);

    match comparison {
        DistComparison::Smaller => dist < dist_bound,
        DistComparison::Larger => dist > dist_bound,
    }
}

pub fn check_volume_predicate(prop: &Prop, katamari: &Katamari, min_kat_volume: f32) -> bool {
    if prop.get_compare_vol_m3() > katamari.max_attach_vol_m3 {
        return false;
    }

    if min_kat_volume > katamari.get_vol() {
        return false;
    }

    // let scale = SIZE_CHART_PROP_SCALES[NamePropConfig::get(prop.get_name_idx())]
    // let scale_config = PROP_SCALE_CONFIGS

    // TODO_HIGH: rest of function
    // let x = (katamari.get_vol() - min_kat_volume) *

    true
}
