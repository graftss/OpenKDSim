use gl_matrix::common::{Mat4, Vec4};

/// General camera state.
/// offset: 0x192ee0
/// width: 0x980
#[derive(Debug, Default)]
pub struct CameraState {
    /// (??) something to do with clearing i think
    /// offset: 0x96c
    pub cam_eff_1P: bool,
}

/// Transform matrices for the camera.
/// offset: 0xd34180
/// width: 0x188
#[derive(Debug, Default)]
pub struct CameraTransform {
    /// The transformation matrix of the camera looking at its target.
    /// offset: 0x0
    transform: Mat4,

    /// The position of the camera.
    /// offset: 0x170
    pos: Vec4,

    /// The extra zoom out distance as the timer expires at the end of MAS4.
    /// offset: 0x180
    mas4_preclear_offset: f32,
}

#[derive(Debug, Default)]
pub struct Camera {
  state: CameraState,
  transform: CameraTransform
}

impl Camera {
    pub fn set_cam_eff_1P(&mut self, cam_eff_1P: i32) {
        self.state.cam_eff_1P = cam_eff_1P > 0;
    }

    pub fn get_matrix(&self, 
        xx: &mut f32, xy: &mut f32, xz: &mut f32, 
        yx: &mut f32, yy: &mut f32, yz: &mut f32, 
        zx: &mut f32, zy: &mut f32, zz: &mut f32, 
        tx: &mut f32, ty: &mut f32, tz: &mut f32,
        offset: &mut f32,
    ) {    
        *xx = self.transform.transform[0];
        *xy = self.transform.transform[1];
        *xz = self.transform.transform[2];
        *yx = self.transform.transform[4];
        *yy = self.transform.transform[5];
        *yz = self.transform.transform[6];
        *zx = self.transform.transform[8];
        *zy = self.transform.transform[9];
        *zz = self.transform.transform[10];

        *tx = self.transform.pos[0];
        *ty = self.transform.pos[1];
        *tz = self.transform.pos[2];

        *offset = self.transform.mas4_preclear_offset;
    }
}
