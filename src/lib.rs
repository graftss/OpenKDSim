#![allow(non_snake_case, dead_code)]

mod global;
mod katamari;
mod mission;
mod camera;
mod gamestate;
mod preclear;
mod ending;
mod constants;
mod delegates;

use delegates::*;
use gamestate::GameState;
use static_init::{dynamic};
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
pub extern "C" fn add_numbers(number1: i32, number2: i32) -> i32 {
    debug_log("asdf");
    number1 + number2 * 2
}

#[no_mangle]
pub extern "C" fn call_fn_ptr(fn_ptr: extern fn(i32) -> ()) -> () {
    debug_log("hihihihidddddi");
    fn_ptr(3);
}

#[no_mangle]
pub extern "C" fn modify_int(x: &mut i32, y: &mut i32, z: &mut i32) -> () {
    *z = *x + *y;
    *x = 0;
    *y = 0;
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
pub unsafe extern "C" fn SetKatamariSpeed(forw_s: f32, side_s: f32, back_s: f32, boost_s: f32, forw_a: f32, side_a: f32, back_a: f32, boost_a: f32, rot_s: f32, dp_y: f32, cam_x: f32, cam_y: f32, cam_z: f32) -> () {
    STATE.write().global.set_speeds(forw_s, side_s, back_s, boost_s, forw_a, side_a, back_a, boost_a, rot_s, dp_y, cam_x, cam_y, cam_z);
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariTranslation(player: i32, x: &mut f32, y: &mut f32, z: &mut f32, sx: &mut f32, sy: &mut f32, sz: &mut f32) -> () {
    STATE.read().read_katamari(player).get_translation(x, y, z, sx, sy, sz);
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariMatrix(player: i32, xx: &mut f32, xy: &mut f32, xz: &mut f32, yx: &mut f32, yy: &mut f32, yz: &mut f32, zx: &mut f32, zy: &mut f32, zz: &mut f32) -> () {
    STATE.read().read_katamari(player).get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz);
}

#[no_mangle]
pub unsafe extern "C" fn SetGravity(x: f32, y: f32, z: f32) {
    STATE.write().global.set_gravity(x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn GetMapRollMatrix(xx: &mut f32, xy: &mut f32, xz: &mut f32, yx: &mut f32, yy: &mut f32, yz: &mut f32, zx: &mut f32, zy: &mut f32, zz: &mut f32) {
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

// [DllImport("PS2KatamariSimulation")]
// [return: MarshalAs(UnmanagedType.I1)]
// public static extern bool TakesCallbackOujiState(int playerID, out IntPtr stateData, out int dataSize);

// [DllImport("PS2KatamariSimulation")]
// public static extern float GetRadiusTargetPercent(int player);

/*
[DllImport("PS2KatamariSimulation")]
public static extern float MonoGetPlacementDataFloat(int placementIndex, int dataType);

[DllImport("PS2KatamariSimulation")]
public static extern void ProcMonoCtrl(int ctrlIndex, int nameIndex, int subObjNum, bool isInit);

[DllImport("PS2KatamariSimulation")]
public static extern void GetMonoDataOffset(int u16MonoNameIdx, int objNo, out float px, out float py, out float pz, out float rx, out float ry, out float rz, out float rw);

[DllImport("PS2KatamariSimulation")]
public static extern int GetMonoDataOffsetExist(int u16MonoNameIdx);

[DllImport("PS2KatamariSimulation")]
public static extern int GetMonoDataConstScreamSeType(int u16MonoNameIdx);

[DllImport("PS2KatamariSimulation")]
public static extern int GetMonoDataConstParent(int u16MonoNameIdx);

[DllImport("PS2KatamariSimulation")]
public static extern void MonoGetVolume(int pracementindex, ref float volume, ref float catchDiameter);

[DllImport("PS2KatamariSimulation")]
public static extern int MonoGetPlacementData(int placementIndex, int dataType);

[DllImport("PS2KatamariSimulation")]
public static extern void MonoGetPlacementDataLocation(int placementIndex, int dataType, out float x, out float y, out float z, out float w);

[DllImport("PS2KatamariSimulation")]
public static extern void SetPropStopFlag(int index, int flag);

[DllImport("PS2KatamariSimulation")]
public static extern void Init(int playerIndex, float overRideSize, int mission);

[DllImport("PS2KatamariSimulation")]
public static extern void ChangeNextArea();

[DllImport("PS2KatamariSimulation")]
public static extern void Tick(float delta);

[DllImport("PS2KatamariSimulation")]
public static extern void DoPropPlacementFinalisation();

[DllImport("PS2KatamariSimulation")]
public static extern void GetCamera(int player, out float xx, out float xy, out float xz, out float yx, out float yy, out float yz, out float zx, out float zy, out float zz, out float tx, out float ty, out float tz, out float offs);

[DllImport("PS2KatamariSimulation")]
public static extern void GetPropMatrix(int monoControlIndex, IntPtr matrixData);

[DllImport("PS2KatamariSimulation")]
public static extern int GetSubObjectCount(int monoControlIndex);

[DllImport("PS2KatamariSimulation")]
public static extern void GetSubObjectPosition(int monoControlIndex, int subObjectIndex, out float posX, out float posY, out float posZ, out float rotX, out float rotY, out float rotZ);

[DllImport("PS2KatamariSimulation")]
public static extern void GetPropSize(int monoControlIndex, out float size);

[DllImport("PS2KatamariSimulation")]
public static extern int GetPropMatrices(IntPtr matrixData);

[DllImport("PS2KatamariSimulation")]
public static extern bool IsAttached(int monoControlIndex);

[DllImport("PS2KatamariSimulation")]
public static extern bool CanBeCollected(int monoControlIndex);

[DllImport("PS2KatamariSimulation")]
public static extern int GetPropAttached(IntPtr propData);

[DllImport("PS2KatamariSimulation")]
public static extern bool GetPrince(int player, out float xx, out float xy, out float xz, out float yx, out float yy, out float yz, out float zx, out float zy, out float zz, out float tx, out float ty, out float tz, out int viewMode, out int faceMode, out int alMode, out int alType, out int hitWater, out float mapLoopRate);

[DllImport("PS2KatamariSimulation")]
public static extern void SetShootingMode(int player, bool fg, bool reset);

[DllImport("PS2KatamariSimulation")]
public static extern void AddKatamariRadius(int player, float add);

[DllImport("PS2KatamariSimulation")]
public static extern void SetStickState(int player, float leftStickX, float leftStickY, float rightStickX, float rightStickY, bool leftStickClickDown, bool rightStickClickDown, bool leftStickClickIsDown, bool rightStickClickIsDown);

[DllImport("PS2KatamariSimulation")]
public static extern void SetTutorialA(int page, int value);

[DllImport("PS2KatamariSimulation")]
public static extern void SetTriggerState(int playerNo, bool triggerLeft1Down, bool triggerLeft2Down, bool triggerRight1Down, bool triggerRight2Down, bool triggerLeft1IsDown, bool triggerLeft2IsDown, bool triggerRight1IsDown, bool triggerRight2IsDown, bool crossClick);

[DllImport("PS2KatamariSimulation")]
public static extern void SetPropMatrix(int monoControlIndex, IntPtr matrix);

[DllImport("PS2KatamariSimulation")]
public static extern void SetGameTime(int gameTime, int remainTime, int freeze, int camEff1P);

[DllImport("PS2KatamariSimulation")]
public static extern void SetPreclearMode(int mode);

public static void SetMapChangeMode_(int mode)
{
    //PMod.Debug.DebugUtil.TempDebugLog($"SetMapChangeMode({mode})");
    SetMapChangeMode(mode);
}

[DllImport("PS2KatamariSimulation")]
public static extern void SetMapChangeMode(int mode);


private static void MonoInitStart_(IntPtr monoData, int mission, int area, int stage, int kadaiFlag, int clearFlag, int endFlag)
{
    MonoInitStart(monoData, mission, area, stage, kadaiFlag, clearFlag, endFlag);
}

[DllImport("PS2KatamariSimulation")]
private static extern void MonoInitStart(IntPtr monoData, int mission, int area, int stage, int kadaiFlag, int clearFlag, int endFlag);

[DllImport("PS2KatamariSimulation")]
private static extern int MonoInitAddProp(float posX, float posY, float posZ, float rotX, float rotY, float rotZ, float rotW, float sclX, float sclY, float sclZ, ushort u16MonoNameIdx, ushort u8LocPosType, short s8RandomLocGroupNo, short s16MonoMoveTypeNo, short s8MonoHitOnAreaNo, ushort u8MonoLinkActNo, ushort u8MonoExActTypeNo, ushort u8MonoIdNameNo, short s8MonoDispOffAreaNo, ushort u8VsMonoDropFlag, short s8MonoCommentNo, short s8MonoCommentGroupNo, short s8MonoTwinsNo, ushort u8MonoShakeOffFlag);

[DllImport("PS2KatamariSimulation")]
public static extern void MonoInitAddPropSetParent(int placementIndex, int parentPlacementIndex);

[DllImport("PS2KatamariSimulation")]
private static extern void MonoInitEnd();
*/
