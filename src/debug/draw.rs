use std::mem::transmute;

use gl_matrix::{
    common::{Mat4, Vec3, Vec4},
    mat4, vec3, vec4,
};

use crate::{
    collision::mesh::{MeshSector, TriGroup},
    delegates::DebugDrawDelegate,
};

/// The draw commands performed by the `debug_draw` delegate.
#[repr(C)]
pub enum DebugDrawType {
    Line = 0,
    Box = 1,
    Point = 2,
    TriangleList = 3,
    TriangleStrip = 4,
}

impl From<DebugDrawType> for i32 {
    fn from(value: DebugDrawType) -> Self {
        match value {
            DebugDrawType::Line => 0,
            DebugDrawType::Box => 1,
            DebugDrawType::Point => 2,
            DebugDrawType::TriangleList => 3,
            DebugDrawType::TriangleStrip => 4,
        }
    }
}

#[derive(Default)]
pub struct DebugDrawBus {
    delegate: Option<DebugDrawDelegate>,
    unity_data_ptr: usize,
}

macro_rules! slice {
    ($out: expr, $count: expr) => {
        core::slice::from_raw_parts_mut($out, $count)
            .try_into()
            .unwrap()
    };
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

                let mut out_p0: &mut [f32; 3] = slice!(out, 3);
                vec3::copy(&mut out_p0, &p0);
                out = out.offset(3);

                let mut out_p1: &mut [f32; 3] = slice!(out, 3);
                vec3::copy(&mut out_p1, &p1);
                out = out.offset(3);

                let mut out_color: &mut [f32; 4] = slice!(out, 4);
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

                let mut out_min: &mut [f32; 3] = slice!(out, 3);
                vec3::copy(&mut out_min, &min);
                out = out.offset(3);

                let mut out_max: &mut [f32; 3] = slice!(out, 3);
                vec3::copy(&mut out_max, &max);
                out = out.offset(3);

                let mut out_transform: &mut [f32; 16] = slice!(out, 16);
                mat4::copy(&mut out_transform, &transform);
                out = out.offset(16);

                let mut out_color: &mut [f32; 4] = slice!(out, 4);
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

                let mut out_point: &mut [f32; 3] = slice!(out, 3);
                vec3::copy(&mut out_point, &point);
                out = out.offset(3);

                let mut out_color: &mut [f32; 4] = slice!(out, 4);
                vec4::copy(&mut out_color, color);
                // out = out.offset(4)
            }

            draw(DebugDrawType::Point);
        }
    }

    /// Draw a triangle group relative to the transform matrix `transform`.
    pub fn draw_tri_group(&mut self, tri_group: &TriGroup, transform: &Mat4, color: &Vec4) {
        if let Some(draw) = self.delegate {
            unsafe {
                let mut out = self.unity_data_ptr as *mut f32;

                let out_num_vertices: &mut [f32; 1] = slice!(out, 1);
                let num_vertices = tri_group.vertices.len() as u32;
                out_num_vertices[0] = transmute::<u32, f32>(num_vertices);
                out = out.offset(1);

                for vertex in &tri_group.vertices {
                    let mut out_vertex: &mut [f32; 3] = slice!(out, 3);
                    vec3::copy(&mut out_vertex, &vertex.point);
                    out = out.offset(3);
                }

                let mut out_transform: &mut [f32; 16] = slice!(out, 16);
                mat4::copy(&mut out_transform, transform);
                out = out.offset(16);

                let mut out_color: &mut [f32; 4] = slice!(out, 4);
                vec4::copy(&mut out_color, color);
                // out = out.offset(4);
            }

            let draw_type = match tri_group.is_tri_strip {
                true => DebugDrawType::TriangleStrip,
                false => DebugDrawType::TriangleList,
            };
            draw(draw_type);
        }
    }

    pub fn draw_mesh_sector(
        &mut self,
        sector: &MeshSector,
        transform: &Mat4,
        sector_colors: &[Vec4],
    ) {
        for (idx, tri_group) in sector.tri_groups.iter().enumerate() {
            self.draw_tri_group(tri_group, transform, &sector_colors[idx]);
        }
    }
}
