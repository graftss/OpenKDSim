use gl_matrix::vec3;

use crate::{
    macros::{set_y, vec3_from},
    math::{vec3_inplace_normalize, vec3_inplace_scale},
    player::katamari::Katamari,
    props::{
        config::NamePropConfig,
        prop::{Prop, PropGlobalState, PropRef},
    },
};

/// The "scale" of each bucket of props in the size chart (in the in-game collection).
/// offset: 0x68960
const SIZE_CHART_PROP_SCALES: [u8; 12] = [0, 0, 0, 1, 2, 3, 3, 4, 4, 4, 4, 4];

// TODO_DOC
/// Fixed parameters common to all props of a given scale.
struct PropScaleConfig {
    // TODO_DOC
    /// offset: 0x0
    min_alert_dist: f32,

    // TODO_DOC
    /// offset: 0x4
    field_0x4: f32,

    // TODO_DOC
    /// offset: 0x8
    field_0x8: f32,

    // TODO_DOC
    /// offset: 0xc
    field_0xc: f32,
}

/// Fixed parameters for each prop scale.
/// Note that the largest two prop scales (index 5 and 6) are apparently unused, since the mapping
/// of size chart size to scale maxes out at index 4.
/// offset: 0x7edf0
const PROP_SCALE_CONFIGS: [PropScaleConfig; 7] = [
    PropScaleConfig {
        min_alert_dist: 200.0,
        field_0x4: 300.0,
        field_0x8: 30.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        min_alert_dist: 300.0,
        field_0x4: 500.0,
        field_0x8: 50.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        min_alert_dist: 500.0,
        field_0x4: 700.0,
        field_0x8: 100.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        min_alert_dist: 700.0,
        field_0x4: 900.0,
        field_0x8: 200.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        min_alert_dist: 900.0,
        field_0x4: 1300.0,
        field_0x8: 300.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        min_alert_dist: 1500.0,
        field_0x4: 2000.0,
        field_0x8: 400.0,
        field_0xc: 10.0,
    },
    PropScaleConfig {
        min_alert_dist: 1800.0,
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

/// Used in three volume-based predicates:
/// offset: 0x35f90 (`min_kat_volume` is offset 0x4 of `motion_state`)
/// offset: 0x361f0 (`min_kat_volume` is offset 0x80 of `motion_state`)
/// offset: 0x36420 (`min_kat_volume` is offset 0x100 of `motion_state`, which seems to be pickup volume)
pub fn vol_and_dist_predicate(prop: &Prop, katamari: &Katamari, min_kat_volume: f32) -> bool {
    if prop.get_compare_vol_m3() > katamari.max_attach_vol_m3 {
        return false;
    }

    let extra_vol = katamari.get_vol() - min_kat_volume;

    if extra_vol.is_sign_negative() {
        return false;
    }

    let size_chart_idx = NamePropConfig::get(prop.get_name_idx()).size_chart_idx as usize;
    let scale_config = &PROP_SCALE_CONFIGS[SIZE_CHART_PROP_SCALES[size_chart_idx] as usize];

    let dist_bound = extra_vol * scale_config.field_0x8
        / (min_kat_volume * scale_config.field_0xc - min_kat_volume)
        + scale_config.min_alert_dist;

    compare_lateral_prop_kat_dist(prop, katamari, DistComparison::Smaller, dist_bound)
}

pub fn nearby_and_1m_kat_predicate(prop: &Prop, katamari: &Katamari) -> bool {
    katamari.get_diam_cm() > 100.0
        && compare_lateral_prop_kat_dist(prop, katamari, DistComparison::Smaller, 200.0)
}

/// Switches to alt motion if:
///   - the prop's immediate parent is attached, AND
///   - the katamari is in the same zone as the prop
/// offset: 0x36050
pub fn guard_parent_in_zone_predicate(
    prop_ref: PropRef,
    prop_zone: Option<u8>,
    kat_zone: Option<u8>,
) -> bool {
    let prop = prop_ref.as_ref().borrow();
    let parent_ref = prop.parent_ref.as_ref().and_then(|p| p.upgrade());

    if let Some(parent_ref) = parent_ref {
        let parent_prop = parent_ref.borrow();
        assert!(prop_zone.is_some());
        prop_zone == kat_zone && parent_prop.global_state == PropGlobalState::Attached
    } else {
        false
    }
}

/// Switches to alt motion *and detaches itself from the parent* when the parent is attached.
/// offset: 0x361b0
pub fn guard_parent_predicate(prop: &mut Prop) -> bool {
    let parent_ref = prop.parent_ref.as_ref().and_then(|p| p.upgrade());
    if let Some(parent_ref) = parent_ref {
        let mut parent_prop = parent_ref.borrow_mut();
        let parent_attached = parent_prop.global_state == PropGlobalState::Attached;

        if parent_attached {
            prop.parent = None;
            prop.next_sibling = None;
            parent_prop.first_child = None;
        }

        parent_attached
    } else {
        prop.alt_motion_action = None;
        prop.move_type = None;
        false
    }
}

// TODO
/// offset: 0x362b0
pub fn behavior_3_predicate() {}

// TODO
/// offset: 0x36080
pub fn behavior_5_predicate() {}

/// Never switches to alt motion.
/// offset: 0x35f80
pub fn never_predicate() -> bool {
    false
}

/// Switches to alt motion when the area `trigger_area` is loaded.
/// offset: 0x360a0
pub fn area_loaded_predicate(trigger_area: u8, loaded_area: u8) -> bool {
    trigger_area == loaded_area
}
