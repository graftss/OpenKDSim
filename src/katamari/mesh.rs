use gl_matrix::common::Vec4;

use crate::{macros::read_f32, math::vec4_scale};

static KAT_MESHES: &'static [u8] = include_bytes!("../data/kat_mesh.bin");

pub struct KatMesh {
    pub points: Vec<Vec4>,
}

impl KatMesh {
    /// Parse the built-in mesh endpoints from the dumped `kat_mesh.bin` file.
    pub fn init_builtin_meshes() -> Vec<KatMesh> {
        let mut result = vec![];
        let mut mesh_points: Vec<Vec4> = vec![];

        for point_bytes in KAT_MESHES.chunks(16) {
            let mut point = [
                read_f32!(point_bytes, 0),
                read_f32!(point_bytes, 4),
                read_f32!(point_bytes, 8),
                read_f32!(point_bytes, 12),
            ];

            // make all the points negative to translate from simulation to unity coordinates
            vec4_scale(&mut point, -1.0);

            println!("point={:?}", point);

            if point[3] == -1.0 {
                println!("found mesh with {} points", mesh_points.len());
                result.push(KatMesh {
                    points: mesh_points,
                });
                mesh_points = vec![];
            } else {
                mesh_points.push(point);
            }
        }

        result
    }
}
