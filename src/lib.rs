#![allow(non_snake_case, dead_code)]

mod camera;
mod constants;
mod delegates;
mod ending;
mod gamestate;
mod global;
mod input;
mod katamari;
mod mission;
mod name_prop_config;
mod preclear;
mod prince;
mod prop;
mod util;

use delegates::*;
use gamestate::GameState;
use gl_matrix::common::Mat4;
use name_prop_config::{NAME_PROP_CONFIGS, NamePropConfig};
use prince::OujiState;
use static_init::{dynamic};
use core::{panic, slice};
use std::fs::{OpenOptions};
use std::io::prelude::*;
use std::path::Path;

#[dynamic] 
static mut STATE: GameState = GameState::default();

pub fn debug_log(str: &str) {
    let path = Path::new("C:\\Program Files (x86)\\Steam\\steamapps\\common\\Katamari Damacy REROLL\\debug.log");
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();
    
    if let Err(_e) = writeln!(file, "{}", str){
        eprintln!("oopsie");
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariCatchCountB() -> i32 {
    STATE.read().global.catch_count_b
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariRadius(player: i32) -> f32 {
    // this is divided by 100 for no reason (the 100 is immediately multiplied back in unity).
    STATE.read().read_katamari(player).get_radius() / 100.0
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDiameterInt(player: i32) -> i32 {
    STATE.read().read_katamari(player).get_diam_int()
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariVolume(player: i32) -> f32 {
    STATE.read().read_katamari(player).get_vol()
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDisplayRadius(player: i32) -> f32 {
    STATE.read().read_katamari(player).get_display_radius()
}

#[no_mangle]
pub unsafe extern "C" fn GetPreclearAlpha() -> f32 {
    STATE.read().preclear.get_alpha()
}

#[no_mangle]
pub unsafe extern "C" fn SetKatamariSpeed(
    forw_s: f32, side_s: f32, back_s: f32, boost_s: f32, 
    forw_a: f32, side_a: f32, back_a: f32, boost_a: f32, 
    rot_s: f32, dp_y: f32, 
    cam_x: f32, cam_y: f32, cam_z: f32
) {
    STATE.write().global.set_speeds(forw_s, side_s, back_s, boost_s, forw_a, side_a, back_a, boost_a, rot_s, dp_y, cam_x, cam_y, cam_z);
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariTranslation(
    player: i32, 
    x: &mut f32, y: &mut f32, z: &mut f32, 
    sx: &mut f32, sy: &mut f32, sz: &mut f32
) {
    STATE.read().read_katamari(player).get_translation(x, y, z, sx, sy, sz);
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariMatrix(
    player: i32, 
    xx: &mut f32, xy: &mut f32, xz: &mut f32, 
    yx: &mut f32, yy: &mut f32, yz: &mut f32, 
    zx: &mut f32, zy: &mut f32, zz: &mut f32
) {
    STATE.read().read_katamari(player).get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz);
}

#[no_mangle]
pub unsafe extern "C" fn SetGravity(x: f32, y: f32, z: f32) {
    STATE.write().global.set_gravity(x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn GetMapRollMatrix(
    xx: &mut f32, xy: &mut f32, xz: &mut f32, 
    yx: &mut f32, yy: &mut f32, yz: &mut f32, 
    zx: &mut f32, zy: &mut f32, zz: &mut f32
) {
    STATE.read().ending.get_map_roll_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz);
}

#[no_mangle]
pub unsafe extern "C" fn SetGameMode(mode: i32) {
    STATE.write().global.set_gamemode(mode)
}

#[no_mangle]
pub unsafe extern "C" fn SetKatamariTranslation(player: i32, x: f32, y: f32, z: f32) {
    STATE.write().write_katamari(player).set_translation(x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMonoGenerate(cb: MonoGenerateDelegate) {
    STATE.write().delegates.mono_generate = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMotionEnd(cb: MotionEndDelegate) {
    STATE.write().delegates.motion_end = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackMessageRequest(cb: MessageRequestDelegate) {
    STATE.write().delegates.message_request = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackDoHit(cb: DoHitDelegate) {
    STATE.write().delegates.do_hit = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetHitCount(cb: GetHitCountDelegate) {
    STATE.write().delegates.get_hit_count = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetImpactPoint(cb: GetImpactPointDelegate) {
    STATE.write().delegates.get_impact_point = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetImpactNormal(cb: GetImpactNormalDelegate) {
    STATE.write().delegates.get_impact_normal = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackGetHitAttribute(cb: GetHitAttributeDelegate) {
    STATE.write().delegates.get_hit_attribute = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlaySoundFX(cb: PlaySoundFxDelegate) {
    STATE.write().delegates.play_sound_fx = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlayVisualFX(cb: PlayVisualFxDelegate) {
    STATE.write().delegates.play_visual_fx = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackVibration(cb: VibrationDelegate) {
    STATE.write().delegates.vibration = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackPlayAnimation(cb: PlayAnimationDelegate) {
    STATE.write().delegates.play_animation = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackLogPropCollected(cb: LogPropCollectedDelegate) {
    STATE.write().delegates.log_prop_collected = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackSetCamera(cb: SetCameraDelegate) {
    STATE.write().delegates.set_camera = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackVsVolumeDiff(cb: VsVolumeDiffDelegate) {
    STATE.write().delegates.vs_volume_diff = Some(cb);
}

#[no_mangle]
pub unsafe extern "C" fn TakesCallbackOujiState(player: i32, oujistate: &mut *mut OujiState, data_size: &mut i32) -> bool {
    STATE.write().write_prince(player).copy_oujistate_ptr(oujistate, data_size);
    true
}

#[no_mangle]
pub unsafe extern "C" fn SetGameTime(game_time_ms: i32, remain_time_ticks: i32, freeze: i32, cam_eff_1P: i32) {
    STATE.write().set_game_time(game_time_ms, remain_time_ticks, freeze, cam_eff_1P);
}

#[no_mangle]
pub unsafe extern "C" fn GetCamera(
    player: i32, 
    xx: &mut f32, xy: &mut f32, xz: &mut f32, 
    yx: &mut f32, yy: &mut f32, yz: &mut f32, 
    zx: &mut f32, zy: &mut f32, zz: &mut f32, 
    tx: &mut f32, ty: &mut f32, tz: &mut f32,
    offset: &mut f32
) {
    STATE.read().read_camera(player).get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz, offset);
}

#[no_mangle]
pub unsafe extern "C" fn GetPrince(
    player: i32, 
    xx: &mut f32, xy: &mut f32, xz: &mut f32, 
    yx: &mut f32, yy: &mut f32, yz: &mut f32, 
    zx: &mut f32, zy: &mut f32, zz: &mut f32, 
    tx: &mut f32, ty: &mut f32, tz: &mut f32,
    view_mode: &mut i32, face_mode: &mut i32,
    alarm_mode: &mut i32, alarm_type: &mut i32,
    hit_water: &mut i32, map_loop_rate: &mut f32,
) {
    STATE.read().get_prince(
        player, 
        xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz, 
        view_mode, face_mode, alarm_mode, alarm_type, hit_water, map_loop_rate
    );
}

#[no_mangle]
pub unsafe extern "C" fn SetStickState(
    player: i32,
    ls_x: f32, ls_y: f32, rs_x: f32, rs_y: f32,
    l3_down: bool, r3_down: bool, l3_held: bool, r3_held: bool,
) {
    STATE.write()
        .write_input(player)
        .set_stick_state(ls_x, ls_y, rs_x, rs_y, l3_down, r3_down, l3_held, r3_held);
}

#[no_mangle]
pub unsafe extern "C" fn SetTriggerState(
    player: i32,
    l1_down: bool, l1_held: bool, l2_down: bool, l2_held: bool,
    r1_down: bool, r1_held: bool, r2_down: bool, r2_held: bool,
    cross_click: bool,
) {
    STATE.write()
        .write_input(player)
        .set_trigger_state(l1_down, l1_held, l2_down, l2_held, r1_down, r1_held, r2_down, r2_held, cross_click);
}

#[no_mangle]
pub unsafe extern "C" fn GetSubObjectCount(ctrl_idx: i32) -> i32 {
    STATE.read()
        .read_prop(ctrl_idx)
        .count_subobjects()
}

#[no_mangle]
pub unsafe extern "C" fn GetSubObjectPosition(
    ctrl_idx: i32, subobj_idx: i32,
    pos_x: &mut f32, pos_y: &mut f32, pos_z: &mut f32,
    rot_x: &mut f32, rot_y: &mut f32, rot_z: &mut f32,
) {
    STATE.read()
        .read_prop(ctrl_idx)
        .get_subobject_position(subobj_idx, pos_x, pos_y, pos_z, rot_x, rot_y, rot_z);
}

/// Passes the bounding sphere radius of the prop with control index `ctrl_idx` to Unity.
#[no_mangle]
pub unsafe extern "C" fn GetPropSize(ctrl_idx: i32, radius: &mut f32) {
    *radius = STATE.read().read_prop(ctrl_idx).get_radius()
}

/// Reads whether the prop `ctrl_idx` is attached to a katamari.
#[no_mangle]
pub unsafe extern "C" fn IsAttached(ctrl_idx: i32) -> bool {
    STATE.read().read_prop(ctrl_idx).is_attached()
}

#[no_mangle]
pub unsafe extern "C" fn MonoGetPlacementDataFloat(ctrl_idx: i32, data_type: i32) -> f32 {
    if data_type == 0xf {
        STATE.read().read_prop(ctrl_idx).get_radius()
    } else {
        panic!("unexpected `data_type` in `MonOGetPlacementDataFloat`.");
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetPropMatrix(ctrl_idx: i32, out: *mut Mat4) {
    STATE.read().read_prop(ctrl_idx).unsafe_copy_transform(out);
}

#[no_mangle]
pub unsafe extern "C" fn GetPropMatrices(out: *mut f32) {
    let mut next_mat = out;

    for prop in &STATE.read().props {
        if !prop.is_initialized() { break; }

        // Convert the `f32` pointer into `out` to a matrix pointer.
        let mat: &mut Mat4 = slice::from_raw_parts_mut(next_mat, 16).try_into().unwrap();

        // Copy the next prop's matrix into `out`.
        prop.unsafe_copy_transform(mat);

        // Increment the pointer into `out` to the next matrix.
        next_mat = next_mat.offset(16);
    }
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
    NamePropConfig::get(name_idx).mono_data_offset_exists().into()
}

#[no_mangle]
pub unsafe extern "C" fn MonoGetVolume(ctrl_idx: i32, volume: &mut f32, collect_diam: &mut i32) {
    STATE.read().read_prop(ctrl_idx).get_volume(volume, collect_diam);
}

#[no_mangle]
pub unsafe extern "C" fn SetPropStopFlag(ctrl_idx: i32, flag: i32) {
    STATE.write().write_prop(ctrl_idx).set_disabled(flag);
}

#[no_mangle]
pub unsafe extern "C" fn SetGameStart(player: i32, area: i32) {
    STATE.write().set_game_start(player, area);
}

#[no_mangle]
pub unsafe extern "C" fn SetAreaChange(player: i32) {
    STATE.write().set_area_change(player);
}

#[no_mangle]
pub unsafe extern "C" fn SetMapChangeMode(map_change_mode: i32) {
    STATE.write().set_map_change_mode(map_change_mode);
}

#[no_mangle]
pub unsafe extern "C" fn TestCrap(name_idx: i32, compare_vol: &mut f32, x: &mut i32) {
    *compare_vol = NamePropConfig::get(name_idx).compare_vol_mult;
    *x = NamePropConfig::get(name_idx).innate_motion_type.into();
}

/*

[DllImport("PS2KatamariSimulation")]
public static extern void SetPreclearMode(int mode);
*/


// [DllImport("PS2KatamariSimulation")]
// public static extern int GetPropAttached(IntPtr propData);

// [DllImport("PS2KatamariSimulation")]
// public static extern float GetRadiusTargetPercent(int player);

/*
[DllImport("PS2KatamariSimulation")]
public static extern void ProcMonoCtrl(int ctrlIndex, int nameIndex, int subObjNum, bool isInit);

[DllImport("PS2KatamariSimulation")]
public static extern void GetMonoDataOffset(int u16MonoNameIdx, int objNo, out float px, out float py, out float pz, out float rx, out float ry, out float rz, out float rw);

[DllImport("PS2KatamariSimulation")]
public static extern void Init(int playerIndex, float overRideSize, int mission);

[DllImport("PS2KatamariSimulation")]
public static extern void ChangeNextArea();

[DllImport("PS2KatamariSimulation")]
public static extern void Tick(float delta);

[DllImport("PS2KatamariSimulation")]
public static extern void DoPropPlacementFinalisation();

[DllImport("PS2KatamariSimulation")]
public static extern void SetTutorialA(int page, int value);


[DllImport("PS2KatamariSimulation")]
private static extern void MonoInitStart(IntPtr monoData, int mission, int area, int stage, int kadaiFlag, int clearFlag, int endFlag);

[DllImport("PS2KatamariSimulation")]
private static extern int MonoInitAddProp(float posX, float posY, float posZ, float rotX, float rotY, float rotZ, float rotW, float sclX, float sclY, float sclZ, ushort u16MonoNameIdx, ushort u8LocPosType, short s8RandomLocGroupNo, short s16MonoMoveTypeNo, short s8MonoHitOnAreaNo, ushort u8MonoLinkActNo, ushort u8MonoExActTypeNo, ushort u8MonoIdNameNo, short s8MonoDispOffAreaNo, ushort u8VsMonoDropFlag, short s8MonoCommentNo, short s8MonoCommentGroupNo, short s8MonoTwinsNo, ushort u8MonoShakeOffFlag);

[DllImport("PS2KatamariSimulation")]
public static extern void MonoInitAddPropSetParent(int placementIndex, int parentPlacementIndex);

[DllImport("PS2KatamariSimulation")]
private static extern void MonoInitEnd();

	[DllImport("PS2KatamariSimulation")]
	private static extern float MonoGetHitOffsetGround(int placementIndex);

	[DllImport("PS2KatamariSimulation")]
	private static extern IntPtr MonoGetPlacementMonoDataName(int placementIndex);


    [DllImport("PS2KatamariSimulation")]
    public static extern void SetCameraCheckScaleUp(int player, int flag);

    [DllImport("PS2KatamariSimulation")]
    public static extern void SetCameraMode(int player, int mode);

    [DllImport("PS2KatamariSimulation")]
    public static extern void SetStoreFlag(int flag);

    [DllImport("PS2KatamariSimulation")]
    public static extern int KataVsGet_AttackCount(int _pl);

    [DllImport("PS2KatamariSimulation")]
    public static extern int KataVsGet_CatchCount(int _pl);    
*/
