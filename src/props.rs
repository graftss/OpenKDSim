use std::{rc::Rc, slice};

use gl_matrix::common::Mat4;

use crate::{constants::ZERO, mono_data::PropMonoData};

use self::{
    config::NamePropConfig,
    motion::global_path::GlobalPathState,
    params::PropParams,
    prop::{AddPropArgs, Prop, PropRef},
};

pub mod config;
pub mod motion;
pub mod params;
pub mod prop;

/// State of all props in the current mission.
#[derive(Debug, Default)]
pub struct Props {
    pub props: Vec<PropRef>,
    pub global_paths: GlobalPathState,
    pub config: Option<&'static Vec<NamePropConfig>>,
    pub params: PropParams,
}

impl Props {
    pub fn get_prop(&self, ctrl_idx: usize) -> Option<&PropRef> {
        self.props.get(ctrl_idx)
    }

    pub fn get_mut_prop(&mut self, ctrl_idx: usize) -> Option<&mut PropRef> {
        self.props.get_mut(ctrl_idx)
    }

    /// Mimicks the `GetPropAttached` API function.
    /// Returns the number of 3-byte prop statuses written to `out`.
    pub unsafe fn get_attach_statuses(&self, out: *mut u8, kat_diam_int: i32) -> i32 {
        let mut num_props = 0;

        for (ctrl_idx, prop_ref) in self.props.iter().enumerate() {
            let prop = prop_ref.borrow();
            if !prop.is_initialized() {
                break;
            }

            let status = out.offset((ctrl_idx * 3).try_into().unwrap());
            prop.get_attach_status(status, kat_diam_int);
            num_props += 1;
        }

        num_props
    }

    /// Creates a new prop from the provided arguments and adds it to the `props` list.
    pub fn add_prop(
        &mut self,
        ctrl_idx: u16,
        args: &AddPropArgs,
        area: u8,
        mono_data: Option<&Rc<PropMonoData>>,
    ) {
        if let Some(md) = mono_data {
            let prop = Prop::new(ctrl_idx, args, area, md);
            self.props.push(prop);
        }
    }

    pub fn change_next_area(&mut self, area: u8) {
        // destroy props which have the new area as their "display off" area.
        self.props.retain(|prop_ref| {
            let prop_cell = prop_ref.clone();
            let prop = prop_cell.borrow_mut();
            prop.check_destroy_on_area_load(area)
        });
    }

    /// Return a pointer to the internal name string of the prop with control index `ctrl_idx`.
    /// Used by unity in some insanely weird way that possibly doesn't even do anything, but whatever.
    pub fn get_internal_prop_name(&self, ctrl_idx: usize) -> *const u8 {
        if let Some(configs) = self.config {
            let name_idx = self
                .get_prop(ctrl_idx)
                .map_or(0, |prop| prop.clone().borrow().get_name_idx());

            if let Some(config) = configs.get(name_idx as usize) {
                return config.internal_name.as_ptr();
            }
        }

        ZERO.as_ptr()
    }

    /// Mimicks the `GetPropMatrices` API function.
    pub unsafe fn get_prop_matrices(&self, out: *mut f32) -> i32 {
        let mut next_mat = out;
        let mut result = 0;

        for prop_ref in self.props.iter() {
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
            result += 1;
        }

        result
    }
}
