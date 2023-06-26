#![feature(const_float_bits_conv)]
#![allow(non_snake_case, dead_code, unused_imports)]

// reference this first so it's available to all other modules
mod macros;

mod collision;
mod constants;
mod delegates;
mod gamestate;
mod global;
mod math;
mod mission;
mod mono_data;
mod player;
mod props;
mod util;

use backtrace::Backtrace;
use collision::raycast_state::{RaycastCallType, RaycastState};
use delegates::*;
use gamestate::GameState;
use gl_matrix::common::Mat4;
use macros::temp_debug_log;
use player::prince::OujiState;
use props::{
    config::NamePropConfig,
    prop::{AddPropArgs, Prop},
};
use std::cell::RefCell;

use crate::macros::{log, panic_log};

thread_local! {
    static STATE: RefCell<GameState> = RefCell::new(GameState::default());
}

/// Helper function to read the prop config for name index `name_idx` from the game state.
fn with_prop_config<T>(name_idx: usize, cb: fn(config: &NamePropConfig) -> T) -> T {
    STATE.with(|state| {
        if let Some(configs) = state.borrow().props.config {
            if let Some(config) = configs.get(name_idx as usize) {
                return cb(config);
            }
        }

        panic_log!("Error reading prop config for name index {}.", name_idx);
    })
}

// Helper function to read from the prop at control index `ctrl_idx`.
fn with_prop<T>(ctrl_idx: usize, cb: fn(prop: &Prop) -> T) -> T {
    STATE.with(|state| {
        if let Some(prop_ref) = state.borrow().props.get_prop(ctrl_idx as usize) {
            return cb(&prop_ref.clone().borrow());
        }

        panic_log!("Error reading prop with control index {}.", ctrl_idx);
    })
}

/// Helper function to mutate the prop at control index `ctrl_idx`.
fn with_prop_mut<F, T>(ctrl_idx: i32, cb: F) -> T
where
    F: FnOnce(&mut Prop) -> T,
{
    STATE.with(|state| {
        let mut s = state.borrow_mut();
        let prop = s.props.get_mut_prop(ctrl_idx as usize).unwrap();

        let mut p = prop.as_ref().borrow_mut();
        cb(&mut p)
    })
}

#[no_mangle]
pub unsafe extern "C" fn OpenSimTest() {
    STATE.with(|state| {
        let mut raycast_state = RaycastState::default();
        let del = state.borrow().delegates.clone();
        raycast_state.set_delegates(&del);
        log!("hellooooo before finding hit");
        raycast_state.find_nearest_unity_hit(RaycastCallType::Objects, true);
    });
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariCatchCountB() -> i32 {
    STATE.with(|inner| inner.borrow().global.catch_count_b)
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariRadius(player_idx: i32) -> f32 {
    // this is divided by 100 for no reason (the 100 is immediately multiplied back in unity).
    STATE.with(|state| {
        state
            .borrow()
            .get_player(player_idx as usize)
            .katamari
            .get_radius()
            / 100.0
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDiameterInt(player_idx: i32) -> i32 {
    STATE.with(|state| {
        state
            .borrow()
            .get_player(player_idx as usize)
            .katamari
            .get_diam_int()
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariVolume(player_idx: i32) -> f32 {
    STATE.with(|state| {
        state
            .borrow()
            .get_player(player_idx as usize)
            .katamari
            .get_vol()
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDisplayRadius(player_idx: i32) -> f32 {
    STATE.with(|state| {
        state
            .borrow()
            .get_player(player_idx as usize)
            .katamari
            .get_display_radius()
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetPreclearAlpha() -> f32 {
    STATE.with(|state| state.borrow().get_player(0).camera.preclear.get_alpha())
}

#[no_mangle]
pub unsafe extern "C" fn SetKatamariSpeed(
    forw_s: f32,
    side_s: f32,
    back_s: f32,
    boost_s: f32,
    forw_a: f32,
    side_a: f32,
    back_a: f32,
    boost_a: f32,
    rot_s: f32,
    limit_y: f32,
    cam_x: f32,
    cam_y: f32,
    cam_z: f32,
) {
    STATE.with(|state| {
        state.borrow_mut().set_katamari_speed(
            forw_s, side_s, back_s, boost_s, forw_a, side_a, back_a, boost_a, rot_s, limit_y,
            cam_x, cam_y, cam_z,
        );
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariTranslation(
    player_idx: i32,
    x: &mut f32,
    y: &mut f32,
    z: &mut f32,
    sx: &mut f32,
    sy: &mut f32,
    sz: &mut f32,
) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_player(player_idx as usize)
            .katamari
            .get_translation(x, y, z, sx, sy, sz);
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariMatrix(
    player_idx: i32,
    xx: &mut f32,
    xy: &mut f32,
    xz: &mut f32,
    yx: &mut f32,
    yy: &mut f32,
    yz: &mut f32,
    zx: &mut f32,
    zy: &mut f32,
    zz: &mut f32,
) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_player(player_idx as usize)
            .katamari
            .get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetGravity(x: f32, y: f32, z: f32) {
    STATE.with(|state| {
        state.borrow_mut().global.set_gravity(x, y, z);
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetMapRollMatrix(
    xx: &mut f32,
    xy: &mut f32,
    xz: &mut f32,
    yx: &mut f32,
    yy: &mut f32,
    yz: &mut f32,
    zx: &mut f32,
    zy: &mut f32,
    zz: &mut f32,
) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .mission_state
            .ending
            .as_mut()
            .unwrap()
            .get_map_roll_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetGameMode(mode: u32) {
    STATE.with(|state| state.borrow_mut().mission_state.set_gamemode(mode as u8))
}

#[no_mangle]
pub unsafe extern "C" fn SetKatamariTranslation(player_idx: i32, x: f32, y: f32, z: f32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(player_idx as usize)
            .katamari
            .set_translation(x, y, z);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMonoGenerate(cb: MonoGenerateDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().mono_generate = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMotionEnd(cb: MotionEndDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().motion_end = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMessageRequest(cb: MessageRequestDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().message_request = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackDoHit(cb: DoHitDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().do_hit = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetHitCount(cb: GetHitCountDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().get_hit_count = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetImpactPoint(cb: GetImpactPointDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().get_impact_point = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetImpactNormal(cb: GetImpactNormalDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().get_impact_normal = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetHitAttribute(cb: GetHitAttributeDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().get_hit_attribute = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlaySoundFX(cb: PlaySoundFxDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().play_sound_fx = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlayVisualFX(cb: PlayVisualFxDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().play_visual_fx = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackVibration(cb: VibrationDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().vibration = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlayAnimation(cb: PlayAnimationDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().play_animation = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackLogPropCollected(cb: LogPropCollectedDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().log_prop_collected = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackSetCamera(cb: SetCameraDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().set_camera = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackVsVolumeDiff(cb: VsVolumeDiffDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().vs_volume_diff = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackOujiState(
    player_idx: i32,
    oujistate: &mut *mut OujiState,
    data_size: &mut i32,
) -> bool {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(player_idx as usize)
            .prince
            .copy_oujistate_ptr(oujistate, data_size);
        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetGameTime(
    game_time_ms: i32,
    remain_time_ticks: i32,
    freeze: i32,
    cam_eff_1P: i32,
) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .set_game_time(game_time_ms, remain_time_ticks, freeze, cam_eff_1P);
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetCamera(
    player_idx: i32,
    xx: &mut f32,
    xy: &mut f32,
    xz: &mut f32,
    yx: &mut f32,
    yy: &mut f32,
    yz: &mut f32,
    zx: &mut f32,
    zy: &mut f32,
    zz: &mut f32,
    tx: &mut f32,
    ty: &mut f32,
    tz: &mut f32,
    offset: &mut f32,
) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_player(player_idx as usize)
            .camera
            .get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz, offset);
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetPrince(
    player_idx: i32,
    xx: &mut f32,
    xy: &mut f32,
    xz: &mut f32,
    yx: &mut f32,
    yy: &mut f32,
    yz: &mut f32,
    zx: &mut f32,
    zy: &mut f32,
    zz: &mut f32,
    tx: &mut f32,
    ty: &mut f32,
    tz: &mut f32,
    view_mode: &mut i32,
    face_mode: &mut i32,
    alarm_mode: &mut i32,
    alarm_type: &mut i32,
    hit_water: &mut i32,
    map_loop_rate: &mut f32,
) {
    STATE.with(|state| {
        state.borrow().get_prince(
            player_idx as usize,
            xx,
            xy,
            xz,
            yx,
            yy,
            yz,
            zx,
            zy,
            zz,
            tx,
            ty,
            tz,
            view_mode,
            face_mode,
            alarm_mode,
            alarm_type,
            hit_water,
            map_loop_rate,
        );
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetStickState(
    player_idx: i32,
    ls_x: f32,
    ls_y: f32,
    rs_x: f32,
    rs_y: f32,
    l3_down: bool,
    r3_down: bool,
    l3_held: bool,
    r3_held: bool,
) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(player_idx as usize)
            .input
            .set_stick_state(ls_x, ls_y, rs_x, rs_y, l3_down, r3_down, l3_held, r3_held);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetTriggerState(
    player_idx: i32,
    l1_down: u8,
    l1_held: u8,
    l2_down: u8,
    l2_held: u8,
    r1_down: u8,
    r1_held: u8,
    r2_down: u8,
    r2_held: u8,
    cross_click: bool,
) {
    if l1_down + l1_held + l2_down + l2_held + r1_down + r1_held + r2_down + r2_held > 0 {
        log!(
            "set trigger state {}, {}, {}, {}, {}, {}, {}, {}",
            l1_down,
            l1_held,
            l2_down,
            l2_held,
            r1_down,
            r1_held,
            r2_down,
            r2_held
        );
    }
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(player_idx as usize)
            .input
            .set_trigger_state(
                l1_down != 0,
                l1_held != 0,
                l2_down != 0,
                l2_held != 0,
                r1_down != 0,
                r1_held != 0,
                r2_down != 0,
                r2_held != 0,
                cross_click,
            );
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetSubObjectCount(ctrl_idx: i32) -> i32 {
    with_prop(ctrl_idx as usize, |prop| prop.count_subobjects())
}

#[no_mangle]
pub unsafe extern "C" fn GetSubObjectPosition(
    ctrl_idx: i32,
    subobj_idx: i32,
    pos_x: &mut f32,
    pos_y: &mut f32,
    pos_z: &mut f32,
    rot_x: &mut f32,
    rot_y: &mut f32,
    rot_z: &mut f32,
) {
    with_prop_mut(ctrl_idx, |prop| {
        prop.get_subobject_position(subobj_idx, pos_x, pos_y, pos_z, rot_x, rot_y, rot_z);
    });
}

/// Passes the bounding sphere radius of the prop with control index `ctrl_idx` to Unity.
#[no_mangle]
pub unsafe extern "C" fn GetPropSize(ctrl_idx: i32, radius: &mut f32) {
    *radius = with_prop(ctrl_idx as usize, |prop| prop.get_radius());
}

/// Reads whether the prop `ctrl_idx` is attached to a katamari.
#[no_mangle]
pub unsafe extern "C" fn IsAttached(ctrl_idx: i32) -> bool {
    with_prop(ctrl_idx as usize, |prop| prop.is_attached())
}

#[no_mangle]
pub unsafe extern "C" fn MonoGetPlacementDataFloat(ctrl_idx: i32, data_type: i32) -> f32 {
    if data_type == 0xf {
        with_prop(ctrl_idx as usize, |prop| prop.get_radius())
    } else {
        panic_log!("unexpected `data_type` in `MonOGetPlacementDataFloat`.");
    }
}

/// Write the transform of the prop at `ctrl_idx` to `out`.
/// Note: unused in original game
#[no_mangle]
pub unsafe extern "C" fn GetPropMatrix(ctrl_idx: i32, out: *mut Mat4) {
    with_prop_mut(ctrl_idx, |prop| {
        prop.unsafe_copy_transform(out);
    });
}

#[no_mangle]
pub unsafe extern "C" fn GetPropMatrices(out: *mut f32) -> i32 {
    STATE.with(|state| state.borrow().props.get_prop_matrices(out))
}

#[no_mangle]
pub unsafe extern "C" fn GetMonoDataConstScreamSeType(name_idx: i32) -> i32 {
    with_prop_config(name_idx as usize, |config| config.scream_sfx_idx.into())
}

#[no_mangle]
pub unsafe extern "C" fn GetMonoDataConstParent(name_idx: i32) -> i32 {
    with_prop_config(name_idx as usize, |config| {
        config.const_parent_name_idx.into()
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetMonoDataOffsetExist(name_idx: i32) -> i32 {
    with_prop_config(name_idx as usize, |config| {
        config.mono_data_offset_exists().into()
    })
}

#[no_mangle]
pub unsafe extern "C" fn MonoGetVolume(ctrl_idx: i32, volume: &mut f32, collect_diam: &mut i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .props
            .get_prop(ctrl_idx as usize)
            .unwrap()
            .borrow()
            .get_volume(volume, collect_diam);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetPropStopFlag(ctrl_idx: i32, flag: i32) {
    with_prop_mut(ctrl_idx, |prop| prop.set_disabled(flag));
}

#[no_mangle]
pub unsafe extern "C" fn SetGameStart(player_idx: i32, area: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .set_game_start(player_idx as usize, area as u8);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetAreaChange(player_idx: i32) {
    STATE.with(|state| {
        state.borrow_mut().set_area_change(player_idx as usize);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetMapChangeMode(map_change_mode: i32) {
    STATE.with(|state| {
        state.borrow_mut().set_map_change_mode(map_change_mode);
    })
}

#[no_mangle]
pub unsafe extern "C" fn KataVsGet_AttackCount(player_idx: i32) -> i32 {
    STATE.with(|state| {
        state
            .borrow()
            .get_player(player_idx as usize)
            .katamari
            .vs_attack_count
            .into()
    })
}

#[no_mangle]
pub unsafe extern "C" fn KataVsGet_CatchCount(player_idx: i32) -> i32 {
    STATE.with(|state| {
        state
            .borrow()
            .get_player(player_idx as usize)
            .katamari
            .vs_catch_count
            .into()
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetRadiusTargetPercent(player_idx: i32) -> f32 {
    STATE.with(|state| {
        state
            .borrow()
            .get_radius_target_percent(player_idx as usize)
    })
}

/// Writes 3 bytes of status data to `out` for each loaded prop.    
#[no_mangle]
pub unsafe extern "C" fn GetPropAttached(out: *mut u8) -> i32 {
    STATE.with(|state| state.borrow().get_props_attach_status(out))
}

#[no_mangle]
pub unsafe extern "C" fn MonoInitStart(
    mono_data: *const u8,
    mission: i32,
    area: i32,
    stage: i32,
    kadai_flag: i32,
    clear_flag: i32,
    end_flag: i32,
) {
    // since this is the first initialization API call made by unity (before `Init`, go figure)
    // this seems like a reasonable place to reset the game state between attempts
    STATE.with(|state| state.replace(GameState::default()));

    std::panic::set_hook(Box::new(|panic_info| {
        log!("panic: {:?}", panic_info);
        log!("trace: {:?}", Backtrace::new());
    }));
    log!(
        "MonoInitStart({}, {}, {}, {}, {}, {})",
        mission,
        area,
        stage,
        kadai_flag,
        clear_flag,
        end_flag
    );
    STATE.with(|state| {
        state.borrow_mut().mono_init_start(
            mono_data,
            mission as u8,
            area as u8,
            stage as u8,
            kadai_flag != 0,
            clear_flag != 0,
            end_flag != 0,
        );
    })
}

#[no_mangle]
pub unsafe extern "C" fn MonoInitAddProp(
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    rot_w: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    name_idx: u16,
    loc_pos_type: u16,
    random_group_id: u16,
    mono_move_type: u16,
    mono_hit_on_area: u16,
    link_action: u16,
    extra_action_type: u16,
    unique_name_id: u16,
    disp_off_area_no: u16,
    vs_drop_flag: u16,
    comment_id: u16,
    comment_group_id: u16,
    twin_id: u16,
    shake_off_flag: u16,
) -> i32 {
    let mut args = AddPropArgs {
        pos_x,
        pos_y,
        pos_z,
        rot_x,
        rot_y,
        rot_z,
        rot_w,
        scale_x,
        scale_y,
        scale_z,
        name_idx,
        loc_pos_type,
        random_group_id,
        mono_move_type,
        mono_hit_on_area,
        link_action,
        extra_action_type,
        unique_name_id,
        disp_off_area_no,
        vs_drop_flag,
        comment_id,
        comment_group_id,
        twin_id,
        shake_off_flag,
    };

    args.transform_coords_to_sim();

    STATE.with(|state| state.borrow_mut().add_prop(&args))
}

#[no_mangle]
pub unsafe extern "C" fn MonoInitAddPropSetParent(ctrl_idx: i32, parent_ctrl_idx: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .add_prop_set_parent(ctrl_idx, parent_ctrl_idx);
    })
}

#[no_mangle]
pub unsafe extern "C" fn MonoInitEnd() {
    STATE.with(|state| {
        state.borrow_mut().mono_init_end();
    });
}

/// Returns a pointer to the "internal name" string of the prop with control index `ctrl_idx`.
#[no_mangle]
pub unsafe extern "C" fn MonoGetPlacementMonoDataName(ctrl_idx: i32) -> *const u8 {
    STATE.with(|state| {
        state
            .borrow()
            .props
            .get_internal_prop_name(ctrl_idx as usize)
    })
}

/// Returns the highest *local* y coordinate on the prop's transformed AABB.
/// It's used in Unity but appears to do nothing, which is just typical.
#[no_mangle]
pub unsafe extern "C" fn MonoGetHitOffsetGround(ctrl_idx: i32) -> f32 {
    STATE.with(|state| {
        state
            .borrow()
            .props
            .get_prop(ctrl_idx as usize)
            .unwrap()
            .clone()
            .borrow()
            .max_aabb_y()
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetCameraMode(player_idx: i32, mode: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(player_idx as usize)
            .camera
            .set_mode(mode.into())
    });
}

#[no_mangle]
pub unsafe extern "C" fn SetCameraCheckScaleUp(player_idx: i32, flag: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(player_idx as usize)
            .camera
            .check_scale_up(flag != 0)
    });
}

#[no_mangle]
pub unsafe extern "C" fn SetShootingMode(player_idx: i32, fg: i32, reset: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(player_idx as usize)
            .set_shooting_mode(fg != 0, reset != 0)
    });
}

#[no_mangle]
pub unsafe extern "C" fn SetPreclearMode(mode: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .get_mut_player(0)
            .camera
            .preclear
            .set_mode(mode != 0);
    });
}

#[no_mangle]
pub unsafe extern "C" fn SetTutorialA(page: i32, page_step: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .mission_state
            .tutorial
            .as_mut()
            .unwrap()
            .set_page(page, page_step)
    });
}

#[no_mangle]
pub unsafe extern "C" fn SetStoreFlag(flag: i32) {
    STATE.with(|state| state.borrow_mut().set_store_flag(flag != 0));
}

#[no_mangle]
pub unsafe extern "C" fn ChangeNextArea() {
    STATE.with(|state| state.borrow_mut().change_next_area());
}

#[no_mangle]
pub unsafe extern "C" fn Tick(delta: f32) {
    STATE.with(|state| state.borrow_mut().tick(delta));
}

#[no_mangle]
pub unsafe extern "C" fn Init(player_idx: i32, override_init_size: f32, mission: i32) {
    log!("Init({}, {}, {})", player_idx, override_init_size, mission);
    STATE.with(|state| {
        state
            .borrow_mut()
            .init(player_idx as usize, override_init_size, mission as u8)
    });
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackDebugDrawLine(cb: DebugDrawLineDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.borrow_mut().debug_draw_line = Some(cb);
    });
}

/// This seems to be what simulates a single object in the collection UI and the names UI.
/// Not a priority.
#[no_mangle]
pub unsafe extern "C" fn ProcMonoCtrl(
    _ctrl_idx: i32,
    _name_idx: i32,
    _subobj_nm: i32,
    _is_init: i32,
) {
    // TODO
}
