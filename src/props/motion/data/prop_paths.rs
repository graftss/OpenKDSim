

use gl_matrix::common::{Vec3};
use lazy_static::lazy_static;

use crate::macros::{include_bytes_align_as, transmute_included_bytes};

static PROP_PATH_POINTS: &[u8] = include_bytes_align_as!(f32, "./bin/prop_path_points.bin");
static PROP_PATHS: &[u8] = include_bytes_align_as!(PropPath, "./bin/prop_paths.bin");

/// A path that can be travelled by a prop.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PropPath {
    /// An index into the `PropPathData::points` vector indicating the first point on this path.
    pub point_idx: u32,

    /// (??) A global multiplier on the speed of the path.
    /// (But this is usually -1, which probably indicates "normal speed"?)
    pub speed: f32,
}

/// The collection of all data that parameterizes prop paths, as it is stored (roughly)
/// in the original simulation.
pub struct PropPathData {
    paths: &'static [PropPath],
    points: &'static [Vec3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathStage {
    House,
    Town,
    World,
    UrsaMajor,
    ShopDemo,
    VsMode,
    Gameshow,
}

impl PropPathData {
    const NUM_PROP_PATHS: usize = 261;
    const NUM_POINTS: usize = 18921;

    /// Read the `PropPathData` from the binary files extracted from the original simulation.
    pub fn from_bin() -> PropPathData {
        let points = unsafe { transmute_included_bytes!(PROP_PATH_POINTS, Vec3, Self::NUM_POINTS) };

        let paths =
            unsafe { transmute_included_bytes!(PROP_PATHS, PropPath, Self::NUM_PROP_PATHS) };

        Self { points, paths }
    }

    /// The first index in the `paths` array corresponding to the paths in `stage`.
    /// For example, the path with index 17 in the `House` stage is at index
    /// 85 + 17 = 102 in `paths`.
    fn path_stage_init_idx(stage: PathStage) -> usize {
        match stage {
            PathStage::House => 85,
            PathStage::Town => 10,
            PathStage::World => 145,
            PathStage::UrsaMajor => 126,
            PathStage::ShopDemo => 0,
            PathStage::VsMode => 77,
            PathStage::Gameshow => 113,
        }
    }
}

lazy_static! {
    pub static ref PROP_PATH_DATA: PropPathData = PropPathData::from_bin();
}
