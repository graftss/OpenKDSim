#![allow(non_snake_case, dead_code)]

mod globalstate;
mod katamari;
mod mission;
mod camera;
mod gamestate;
mod preclear_mode;

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
    STATE.read().GetKatamariCatchCountB()
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariRadius(player: i32) -> f32 {
    STATE.read().GetKatamariRadius(player as usize)
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDiameterInt(player: i32) -> i32 {
    STATE.read().GetKatamariDiameterInt(player as usize)
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariVolume(player: i32) -> f32 {
    STATE.read().GetKatamariVolume(player as usize)
}

#[no_mangle]
pub unsafe extern "C" fn GetKatamariDisplayRadius(player: i32) -> f32 {
    STATE.read().GetKatamariDisplayRadius(player as usize)
}

#[no_mangle]
pub unsafe extern "C" fn GetPreclearAlpha() -> f32 {
    STATE.read().GetPreclearAlpha()
}

#[no_mangle]
pub unsafe extern "C" fn SetKatamariSpeed(forw_s: f32, side_s: f32, back_s: f32, boost_s: f32, forw_a: f32, side_a: f32, back_a: f32, boost_a: f32, rot_s: f32, dp_y: f32, cam_x: f32, cam_y: f32, cam_z: f32) -> () {
    STATE.write().global.set_speeds(forw_s, side_s, back_s, boost_s, forw_a, side_a, back_a, boost_a, rot_s, dp_y, cam_x, cam_y, cam_z);
}


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
public static extern void GetKatamariMatrix(int player, out float xx, out float xy, out float xz, out float yx, out float yy, out float yz, out float zx, out float zy, out float zz);

[DllImport("PS2KatamariSimulation")]
public static extern void GetKatamariTranslation(int player, out float x, out float y, out float z, out float sx, out float sy, out float sz);

[DllImport("PS2KatamariSimulation")]
public static extern void GetMapRollMatrix(out float xx, out float xy, out float xz, out float yx, out float yy, out float yz, out float zx, out float zy, out float zz);

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
public static extern void SetGameMode(int mode);

[DllImport("PS2KatamariSimulation")]
public static extern void SetTutorialA(int page, int value);

[DllImport("PS2KatamariSimulation")]
public static extern void SetTriggerState(int playerNo, bool triggerLeft1Down, bool triggerLeft2Down, bool triggerRight1Down, bool triggerRight2Down, bool triggerLeft1IsDown, bool triggerLeft2IsDown, bool triggerRight1IsDown, bool triggerRight2IsDown, bool crossClick);

[DllImport("PS2KatamariSimulation")]
public static extern void SetCameraSideVector(float x, float y, float z);

[DllImport("PS2KatamariSimulation")]
public static extern void SetKatamariMatrix(float xx, float xy, float xz, float yx, float yy, float yz, float zx, float zy, float zz);

[DllImport("PS2KatamariSimulation")]
public static extern void SetKatamariTranslation(int player, float x, float y, float z);

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

[DllImport("PS2KatamariSimulation")]
public static extern void SetGravity(float x, float y, float z);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackMonoGenerate(myCallbackMonoGenerateDelegate functionPointer, int idx, int u16MonoNameIdx);

[DllImport("PS2KatamariSimulation")]
public static extern int TakesCallbackMotionEnd(myCallbackMotionEndDelegate functionPointer, int playerNo);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackPrint(myCallbackPrintDelegate functionPointer, string text);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackDrawDebugLine(myCallbackDrawDebugLineDelegate callback, float point0x, float point0y, float point0z, float point1x, float point1y, float point1z, int type);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackDrawDebugSphere(myCallbackDrawDebugSphereDelegate callback, float x, float y, float z, float radius);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackMessageRequest(myCallbackMessageRequestDelegate callback, int msgNo);

[DllImport("PS2KatamariSimulation")]
public static extern int TakesCallbackDoHit(myCallbackDoHitDelegate callback, float point0x, float point0y, float point0z, float point1x, float point1y, float point1z, int includeObjects, int drawHits, int callType);

[DllImport("PS2KatamariSimulation")]
public static extern int TakesCallbackGetHitCount(myCallbackGetHitCountDelegate callback);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackGetImpactPoint(myCallbackGetImpactPointDelegate functionPointer, int hitIndex, ref float x, ref float y, ref float z);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackGetImpactNormal(myCallbackGetImpactNormalDelegate functionPointer, int hitIndex, ref float x, ref float y, ref float z);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackGetHitAttribute(myCallbackGetHitAttributeDelegate functionPointer, int hitIndex, ref int hitAttribute);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackPlaySoundFX(myCallbackGetCallbackPlaySoundFX functionPointer, int soundID, float volumne, int pan);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackPlayVisualFX(myCallbackGetCallbackPlayVisualFX functionPointer, int vfxID, float posX, float posY, float posZ, float dirX, float dirY, float dirZ, float scale, int attachID, int playerID);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackVibration(myCallbackGetCallbackVibration functionPointer, int playerNo, float ratio, float time, int no);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackPlayAnimation(myCallbackGetCallbackPlayAnimation functionPointer, int playerNo, int animationID, float speed, int repeat);

[DllImport("PS2KatamariSimulation")]
[return: MarshalAs(UnmanagedType.I1)]
public static extern bool TakesCallbackOujiState(int playerID, out IntPtr stateData, out int dataSize);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackLogPropCollected(myCallbackGetCallbackLogPropCollected functionPointer, int monoControlIndex);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackSetCamera(myCallbackGetCallbackSetCamera functionPointer, float xx, float xy, float xz, float yx, float yy, float yz, float zx, float zy, float zz, float tx, float ty, float tz);

[DllImport("PS2KatamariSimulation")]
public static extern void TakesCallbackVsVolumeDiff(myCallbackVsVolumeDiff functionPointer, int f1, int f2, int f3, float ratio, float time, int recover);

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
