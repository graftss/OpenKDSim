#![allow(non_snake_case, dead_code)]

// reference this first so it's available to all other modules
mod macros;

mod camera;
mod constants;
mod delegates;
mod ending;
mod gamestate;
mod global;
mod input;
mod katamari;
mod mission;
mod mono_data;
mod name_prop_config;
mod preclear;
mod prince;
mod prop;
mod prop_motion;
mod util;

use core::{panic, slice};
use delegates::*;
use gamestate::GameState;
use gl_matrix::common::Mat4;
use name_prop_config::NamePropConfig;
use prince::OujiState;
use prop::AddPropArgs;
use std::cell::RefCell;
use util::debug_log;

use crate::macros::panic_log;

thread_local! {
    static STATE: RefCell<GameState> = RefCell::new(GameState::default());
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariCatchCountB() -> i32 {
    STATE.with(|inner| inner.borrow().global.catch_count_b)
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariRadius(player: i32) -> f32 {
    // this is divided by 100 for no reason (the 100 is immediately multiplied back in unity).
    STATE.with(|state| state.borrow().borrow_katamari(player).get_radius() / 100.0)
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDiameterInt(player: i32) -> i32 {
    STATE.with(|state| state.borrow().borrow_katamari(player).get_diam_int())
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariVolume(player: i32) -> f32 {
    STATE.with(|state| state.borrow().borrow_katamari(player).get_vol())
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDisplayRadius(player: i32) -> f32 {
    STATE.with(|state| state.borrow().borrow_katamari(player).get_display_radius())
}

#[no_mangle]
pub unsafe extern "C" fn GetPreclearAlpha() -> f32 {
    STATE.with(|state| state.borrow().preclear.get_alpha())
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
    dp_y: f32,
    cam_x: f32,
    cam_y: f32,
    cam_z: f32,
) {
    STATE.with(|state| {
        state.borrow_mut().global.set_speeds(
            forw_s, side_s, back_s, boost_s, forw_a, side_a, back_a, boost_a, rot_s, dp_y, cam_x,
            cam_y, cam_z,
        );
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariTranslation(
    player: i32,
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
            .borrow_katamari(player)
            .get_translation(x, y, z, sx, sy, sz);
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariMatrix(
    player: i32,
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
            .borrow_katamari(player)
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
            .ending
            .get_map_roll_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetGameMode(mode: i32) {
    STATE.with(|state| state.borrow_mut().global.set_gamemode(mode))
}

#[no_mangle]
pub unsafe extern "C" fn SetKatamariTranslation(player: i32, x: f32, y: f32, z: f32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .borrow_mut_katamari(player)
            .set_translation(x, y, z);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMonoGenerate(cb: MonoGenerateDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.mono_generate = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMotionEnd(cb: MotionEndDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.motion_end = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMessageRequest(cb: MessageRequestDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.message_request = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackDoHit(cb: DoHitDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.do_hit = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetHitCount(cb: GetHitCountDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.get_hit_count = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetImpactPoint(cb: GetImpactPointDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.get_impact_point = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetImpactNormal(cb: GetImpactNormalDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.get_impact_normal = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetHitAttribute(cb: GetHitAttributeDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.get_hit_attribute = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlaySoundFX(cb: PlaySoundFxDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.play_sound_fx = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlayVisualFX(cb: PlayVisualFxDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.play_visual_fx = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackVibration(cb: VibrationDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.vibration = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlayAnimation(cb: PlayAnimationDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.play_animation = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackLogPropCollected(cb: LogPropCollectedDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.log_prop_collected = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackSetCamera(cb: SetCameraDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.set_camera = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackVsVolumeDiff(cb: VsVolumeDiffDelegate) {
    STATE.with(|state| {
        state.borrow_mut().delegates.vs_volume_diff = Some(cb);
    })
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackOujiState(
    player: i32,
    oujistate: &mut *mut OujiState,
    data_size: &mut i32,
) -> bool {
    STATE.with(|state| {
        state
            .borrow_mut()
            .borrow_mut_prince(player)
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
    player: i32,
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
            .borrow_camera(player)
            .get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz, offset);
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetPrince(
    player: i32,
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
            player,
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
    player: i32,
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
            .borrow_mut_input(player)
            .set_stick_state(ls_x, ls_y, rs_x, rs_y, l3_down, r3_down, l3_held, r3_held);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetTriggerState(
    player: i32,
    l1_down: bool,
    l1_held: bool,
    l2_down: bool,
    l2_held: bool,
    r1_down: bool,
    r1_held: bool,
    r2_down: bool,
    r2_held: bool,
    cross_click: bool,
) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .borrow_mut_input(player)
            .set_trigger_state(
                l1_down,
                l1_held,
                l2_down,
                l2_held,
                r1_down,
                r1_held,
                r2_down,
                r2_held,
                cross_click,
            );
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetSubObjectCount(ctrl_idx: i32) -> i32 {
    STATE.with(|state| {
        state
            .borrow()
            .read_prop_ref(ctrl_idx)
            .clone()
            .borrow()
            .count_subobjects()
    })
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
    STATE.with(|state| {
        state
            .borrow()
            .read_prop_ref(ctrl_idx)
            .borrow()
            .get_subobject_position(subobj_idx, pos_x, pos_y, pos_z, rot_x, rot_y, rot_z);
    })
}

/// Passes the bounding sphere radius of the prop with control index `ctrl_idx` to Unity.
#[no_mangle]
pub unsafe extern "C" fn GetPropSize(ctrl_idx: i32, radius: &mut f32) {
    *radius = STATE.with(|state| state.borrow().read_prop_ref(ctrl_idx).borrow().get_radius());
}

/// Reads whether the prop `ctrl_idx` is attached to a katamari.
#[no_mangle]
pub unsafe extern "C" fn IsAttached(ctrl_idx: i32) -> bool {
    STATE.with(|state| {
        state
            .borrow()
            .read_prop_ref(ctrl_idx)
            .borrow()
            .is_attached()
    })
}

#[no_mangle]
pub unsafe extern "C" fn MonoGetPlacementDataFloat(ctrl_idx: i32, data_type: i32) -> f32 {
    if data_type == 0xf {
        STATE.with(|state| state.borrow().read_prop_ref(ctrl_idx).borrow().get_radius())
    } else {
        panic_log!("unexpected `data_type` in `MonOGetPlacementDataFloat`.");
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetPropMatrix(ctrl_idx: i32, out: *mut Mat4) {
    STATE.with(|state| {
        state
            .borrow()
            .read_prop_ref(ctrl_idx)
            .borrow()
            .unsafe_copy_transform(out);
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetPropMatrices(out: *mut f32) {
    let mut next_mat = out;

    STATE.with(|state| {
        for prop_ref in &state.borrow().props {
            let prop = prop_ref.borrow();
            if !prop.is_initialized() {
                break;
            }

            // Convert the `f32` pointer into `out` to a matrix pointer.
            let mat: &mut Mat4 = slice::from_raw_parts_mut(next_mat, 16).try_into().unwrap();

            // Copy the next prop's matrix into `out`.
            prop.unsafe_copy_transform(mat);

            // Increment the pointer into `out` to the next matrix.
            next_mat = next_mat.offset(16);
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn GetMonoDataConstScreamSeType(name_idx: i32) -> i32 {
    NamePropConfig::get(name_idx).scream_sfx_idx.into()
}

#[no_mangle]
pub unsafe extern "C" fn GetMonoDataConstParent(name_idx: i32) -> i32 {
    NamePropConfig::get(name_idx).const_parent_name_idx.into()
}

#[no_mangle]
pub unsafe extern "C" fn GetMonoDataOffsetExist(name_idx: i32) -> i32 {
    NamePropConfig::get(name_idx)
        .mono_data_offset_exists()
        .into()
}

#[no_mangle]
pub unsafe extern "C" fn MonoGetVolume(ctrl_idx: i32, volume: &mut f32, collect_diam: &mut i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .read_prop_ref(ctrl_idx)
            .borrow()
            .get_volume(volume, collect_diam);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetPropStopFlag(ctrl_idx: i32, flag: i32) {
    STATE.with(|state| {
        state
            .borrow_mut()
            .write_prop_ref(ctrl_idx)
            .borrow_mut()
            .set_disabled(flag);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetGameStart(player: i32, area: i32) {
    STATE.with(|state| {
        state.borrow_mut().set_game_start(player, area);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetAreaChange(player: i32) {
    STATE.with(|state| {
        state.borrow_mut().set_area_change(player);
    })
}

#[no_mangle]
pub unsafe extern "C" fn SetMapChangeMode(map_change_mode: i32) {
    STATE.with(|state| {
        state.borrow_mut().set_map_change_mode(map_change_mode);
    })
}

#[no_mangle]
pub unsafe extern "C" fn KataVsGet_AttackCount(player: i32) -> i32 {
    STATE.with(|state| {
        state
            .borrow()
            .borrow_katamari(player)
            .vs_attack_count
            .into()
    })
}

#[no_mangle]
pub unsafe extern "C" fn KataVsGet_CatchCount(player: i32) -> i32 {
    STATE.with(|state| state.borrow().borrow_katamari(player).vs_catch_count.into())
}

#[no_mangle]
pub unsafe extern "C" fn GetRadiusTargetPercent(player: i32) -> f32 {
    STATE.with(|state| state.borrow().get_radius_target_percent(player))
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
    kadaiFlag: i32,
    clearFlag: i32,
    endFlag: i32,
) {
    STATE.with(|state| {
        state.borrow_mut().mono_init_start(
            mono_data, mission, area, stage, kadaiFlag, clearFlag, endFlag,
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
    let args = AddPropArgs {
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

    debug_log(&format!("{:#?}", args));

    STATE.with(|state| state.borrow_mut().add_prop(args))
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
    STATE.with(|state| state.borrow().get_internal_prop_name(ctrl_idx))
}

#[no_mangle]
pub unsafe extern "C" fn MonoGetHitOffsetGround(_ctrl_idx: f32) -> f32 {
    // TODO (need to compute prop aabb's)
    0.0
}

#[no_mangle]
pub unsafe extern "C" fn Tick(_delta: f32) {
    // TODO, obviously
}

/*
    [DllImport("PS2KatamariSimulation")]
    public static extern void SetPreclearMode(int mode);

    [DllImport("PS2KatamariSimulation")]
    public static extern void ProcMonoCtrl(int ctrlIndex, int nameIndex, int subObjNum, bool isInit);

    [DllImport("PS2KatamariSimulation")]
    public static extern void ChangeNextArea();

    [DllImport("PS2KatamariSimulation")]
    public static extern void SetTutorialA(int page, int value);

    [DllImport("PS2KatamariSimulation")]
    public static extern void SetCameraCheckScaleUp(int player, int flag);

    [DllImport("PS2KatamariSimulation")]
    public static extern void SetCameraMode(int player, int mode);

    [DllImport("PS2KatamariSimulation")]
    public static extern void SetStoreFlag(int flag);
*/
