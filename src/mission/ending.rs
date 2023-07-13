use gl_matrix::common::Mat4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EndingState {
    map_roll: Mat4,
}

impl EndingState {
    pub fn get_map_roll_matrix(
        &self,
        xx: &mut f32,
        xy: &mut f32,
        xz: &mut f32,
        yx: &mut f32,
        yy: &mut f32,
        yz: &mut f32,
        zx: &mut f32,
        zy: &mut f32,
        zz: &mut f32,
    ) -> () {
        *xx = self.map_roll[0];
        *xy = self.map_roll[1];
        *xz = self.map_roll[2];
        *yx = self.map_roll[4];
        *yy = self.map_roll[5];
        *yz = self.map_roll[6];
        *zx = self.map_roll[8];
        *zy = self.map_roll[9];
        *zz = self.map_roll[10];
    }
}
