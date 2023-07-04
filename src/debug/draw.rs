use core::slice;

use gl_matrix::{
    common::{Mat4, Vec3, Vec4},
    mat4, vec3, vec4,
};

use crate::delegates::DebugDrawDelegate;

/// The draw commands performed by the `debug_draw` delegate.
#[repr(C)]
pub enum DebugDrawType {
    Line = 0,
    Box = 1,
    Point = 2,
}

impl From<DebugDrawType> for i32 {
    fn from(value: DebugDrawType) -> Self {
        match value {
            DebugDrawType::Line => 0,
            DebugDrawType::Box => 1,
            DebugDrawType::Point => 2,
        }
    }
}

#[derive(Default)]
pub struct DebugDrawBus {
    delegate: Option<DebugDrawDelegate>,
    unity_data_ptr: usize,
}

impl DebugDrawBus {
    pub fn new(delegate: DebugDrawDelegate, unity_data_ptr: usize) -> Self {
        Self {
            delegate: Some(delegate),
            unity_data_ptr,
        }
    }

    /// Draw a line in world space.
    pub fn draw_line(&mut self, p0: &Vec3, p1: &Vec3, color: &Vec4) {
        if let Some(draw) = self.delegate {
            unsafe {
                let mut out = self.unity_data_ptr as *mut f32;

                let mut out_p0: &mut [f32; 3] =
                    slice::from_raw_parts_mut(out, 3).try_into().unwrap();
                vec3::copy(&mut out_p0, &p0);
                out = out.offset(3);

                let mut out_p1: &mut [f32; 3] =
                    slice::from_raw_parts_mut(out, 3).try_into().unwrap();
                vec3::copy(&mut out_p1, &p1);
                out = out.offset(3);

                let mut out_color: &mut [f32; 4] =
                    slice::from_raw_parts_mut(out, 4).try_into().unwrap();
                vec4::copy(&mut out_color, &color);
                // out = out.offset(4)
            }

            draw(DebugDrawType::Line);
        }
    }

    /// Draw a box in world space using its `min` and `max` local coordinates and its
    /// transform matrix `transform`.
    pub fn draw_box(&mut self, min: &Vec3, max: &Vec3, transform: &Mat4, color: &Vec4) {
        if let Some(draw) = self.delegate {
            unsafe {
                let mut out = self.unity_data_ptr as *mut f32;

                let mut out_min: &mut [f32; 3] =
                    slice::from_raw_parts_mut(out, 3).try_into().unwrap();
                vec3::copy(&mut out_min, &min);
                out = out.offset(3);

                let mut out_max: &mut [f32; 3] =
                    slice::from_raw_parts_mut(out, 3).try_into().unwrap();
                vec3::copy(&mut out_max, &max);
                out = out.offset(3);

                let mut out_transform: &mut [f32; 16] =
                    slice::from_raw_parts_mut(out, 16).try_into().unwrap();
                mat4::copy(&mut out_transform, &transform);
                out = out.offset(16);

                let mut out_color: &mut [f32; 4] =
                    slice::from_raw_parts_mut(out, 4).try_into().unwrap();
                vec4::copy(&mut out_color, color);
                // out = out.offset(4)
            }

            draw(DebugDrawType::Box);
        }
    }

    /// Draw a point in world space.
    pub fn draw_point(&mut self, point: &Vec3, color: &Vec4) {
        if let Some(draw) = self.delegate {
            unsafe {
                let mut out = self.unity_data_ptr as *mut f32;

                let mut out_point: &mut [f32; 3] =
                    slice::from_raw_parts_mut(out, 3).try_into().unwrap();
                vec3::copy(&mut out_point, &point);
                out = out.offset(3);

                let mut out_color: &mut [f32; 4] =
                    slice::from_raw_parts_mut(out, 4).try_into().unwrap();
                vec4::copy(&mut out_color, color);
                // out = out.offset(4)
            }

            draw(DebugDrawType::Point);
        }
    }
}
