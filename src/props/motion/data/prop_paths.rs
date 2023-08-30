use gl_matrix::common::{Vec3, Vec4};
use lazy_static::lazy_static;

use crate::{
    macros::{include_bytes_align_as, transmute_included_bytes},
    mission::Mission,
    props::{
        motion::actions::path::{FollowPathFlags, PathMotion},
        prop::Prop,
    },
};

static PROP_PATH_POINTS: &[u8] = include_bytes_align_as!(f32, "./bin/prop_path_points.bin");
static PROP_PATHS: &[u8] = include_bytes_align_as!(PropPath, "./bin/prop_paths.bin");

/// Computes the distance from a path point `$path_pt` to another point `$other_pt`.
/// This is unqiue because the path points need to be reflected about the origin, so
/// instead of `$other_pt - $path_pt` it's `$other_pt + $path_pt`
macro_rules! path_pt_dist {
    ($path_pt: expr, $other_pt: expr) => {{
        let diff = [
            $path_pt[0] - $other_pt[0],
            $path_pt[1] - $other_pt[1],
            $path_pt[2] - $other_pt[2],
        ];
        (diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2])
    }};
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

/// Maps a `Mission` to its associated `PathStage`, if that mission has one.
impl TryFrom<Mission> for PathStage {
    type Error = ();
    fn try_from(value: Mission) -> Result<Self, Self::Error> {
        match value {
            Mission::MAS1
            | Mission::MAS2
            | Mission::MAS4
            | Mission::Cancer
            | Mission::Cygnus
            | Mission::Mission13
            | Mission::Eternal1 => Ok(PathStage::House),
            Mission::MAS3
            | Mission::MAS5
            | Mission::MAS8
            | Mission::Corona
            | Mission::Pisces
            | Mission::Virgo
            | Mission::Eternal2 => Ok(PathStage::Town),
            Mission::MAS6
            | Mission::MAS7
            | Mission::MAS9
            | Mission::MTM
            | Mission::Gemini
            | Mission::Taurus
            | Mission::Mission20
            | Mission::NorthStar
            | Mission::Eternal3 => Ok(PathStage::World),
            Mission::Ursa => Ok(PathStage::UrsaMajor),
            Mission::Mission25ShopDemo => Ok(PathStage::ShopDemo),
            Mission::Vs0
            | Mission::Vs1
            | Mission::Vs2
            | Mission::Vs3
            | Mission::Vs4
            | Mission::Vs5
            | Mission::Vs6
            | Mission::Vs7 => Ok(PathStage::VsMode),
            Mission::GameShow => Ok(PathStage::Gameshow),
            _ => Err(()),
        }
    }
}

impl PathStage {
    /// Returns `true` if `mission` is associated to some `PathStage`, which means that
    /// there are paths defined for that mission.
    pub fn has_paths(mission: Mission) -> bool {
        TryInto::<PathStage>::try_into(mission).is_ok()
    }
}

/// A path that can be travelled by a prop.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PropPath {
    /// An index into the `PropPathData::points` vector indicating the first point on this path.
    pub point_idx: u32,

    /// If positive, a fixed speed that props following this path will have.
    /// However, most values of this field are -1, which means that props moving along
    /// this path inherit their speed from their `name_idx`.
    pub speed: f32,
}

impl PropPath {
    /// The "null pointer" values of `point_idx` in the original simulation are encoded
    /// here as `u32::MAX`.
    const NULL_POINT_IDX: u32 = u32::MAX;

    fn has_null_point_idx(&self) -> bool {
        self.point_idx == Self::NULL_POINT_IDX
    }
}

/// The collection of all data that parameterizes prop paths, as it is stored (roughly)
/// in the original simulation.
pub struct PropPathData {
    pub paths: &'static [PropPath],
    pub points: &'static [Vec4],
}

impl PropPathData {
    const NUM_PROP_PATHS: usize = 261;
    const NUM_POINTS: usize = 18921;

    /// Read the `PropPathData` from the binary files extracted from the original simulation.
    fn from_bin() -> PropPathData {
        let points = unsafe { transmute_included_bytes!(PROP_PATH_POINTS, Vec4, Self::NUM_POINTS) };

        let paths =
            unsafe { transmute_included_bytes!(PROP_PATHS, PropPath, Self::NUM_PROP_PATHS) };

        Self { points, paths }
    }

    /// Get a slice of all points on the `path_idx`-th path in `mission`.
    pub fn get_mission_path_points(&self, mission: Mission, path_idx: usize) -> Option<&[Vec4]> {
        self.get_mission_path(mission, path_idx).map(|path| {
            let start = path.point_idx as usize;
            let end = start + self.get_last_path_point_idx(mission, path_idx).unwrap() as usize;
            &self.points[start..end + 1]
        })
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

    /// Get the `stage_path_idx`-th path in `stage`, if it exists.
    fn get_stage_path(&self, stage: PathStage, path_idx: usize) -> Option<&PropPath> {
        let path_idx = Self::path_stage_init_idx(stage) + path_idx;
        let path = &self.paths[path_idx];
        if path.has_null_point_idx() {
            None
        } else {
            Some(path)
        }
    }

    /// Get the `mission_path_idx`-th path in `mission`, if it exists.
    pub fn get_mission_path(&self, mission: Mission, path_idx: usize) -> Option<&PropPath> {
        TryInto::<PathStage>::try_into(mission)
            .ok()
            .and_then(|stage| self.get_stage_path(stage, path_idx))
    }

    /// Get the `point_idx`-th point on the `path_idx`-th path in `stage`.
    /// Returns `true` if no such path exists (for some reason), and `false` if it does.
    fn get_stage_path_point(
        &self,
        out: &mut Vec3,
        stage: PathStage,
        point_idx: usize,
        path_idx: usize,
    ) -> bool {
        self.get_stage_path(stage, path_idx).map_or(true, |path| {
            let pt = self.points[path.point_idx as usize + point_idx];
            out[0] = pt[0];
            out[1] = pt[1];
            out[2] = pt[2];
            false
        })
    }

    /// Get the `point_idx`-th point on the `path_idx`-th path in `mission`.
    ///
    /// Returns `true` if no such path exists (for some reason), and `false` if it does.
    /// offset: 0x37720
    pub fn get_mission_path_point(
        &self,
        out: &mut Vec3,
        mission: Mission,
        point_idx: usize,
        path_idx: usize,
    ) -> bool {
        if let Ok(stage) = TryInto::<PathStage>::try_into(mission) {
            self.get_stage_path_point(out, stage, point_idx, path_idx)
        } else {
            true
        }
    }

    /// A sentinel value used to indicate the end of a path's point list in the `points` array.
    /// Specifically, the `Vec4` value [0.0, 0.0, 0.0, 255.0] indicates the end of a list.
    /// Note that all other index-3 values are 1.0, so it's enough to check that it's not 1.0
    /// rather than specifically checking for the sentinel.
    const END_POINT_LIST_SENTINEL: f32 = 255.0;

    /// Compute the last point index on the `path_idx`-th path in `mission`. This is the
    /// **largest valid** point index for the path.
    /// If no such path exists, returns `None`.
    /// offset: 0x37790
    pub fn get_last_path_point_idx(&self, mission: Mission, path_idx: usize) -> Option<u16> {
        if let Ok(stage) = TryInto::<PathStage>::try_into(mission) {
            if let Some(path) = self.get_stage_path(stage, path_idx) {
                let mut point_idx = path.point_idx as usize;
                let mut result = 0;
                loop {
                    if self.points[point_idx][3] == Self::END_POINT_LIST_SENTINEL {
                        return Some(result - 1);
                    }
                    point_idx += 1;
                    result += 1;
                }
            }
        }

        None
    }

    /// offset: 0x37800
    pub fn load_next_target_point(
        &self,
        motion: &mut impl PathMotion,
        prop: &mut Prop,
        mission: Mission,
    ) -> bool {
        let path_idx = motion.get_path_idx() as usize;

        if self.get_mission_path(mission, path_idx).is_some() {
            let last_pt_idx = self.get_last_path_point_idx(mission, path_idx).unwrap();
            let flags = motion.get_flags();
            let reversed = flags.contains(FollowPathFlags::Reversed);

            // compute and set the next target point index
            let next_pt_idx = match (reversed, motion.get_target_point_idx()) {
                (false, point_idx) if point_idx == last_pt_idx => 0,
                (false, point_idx) => point_idx + 1,
                (true, 0) => last_pt_idx,
                (true, point_idx) => point_idx - 1,
            };
            motion.set_target_point_idx(next_pt_idx);

            // compute and set the next target point
            let mut next_pt = [0.0; 3];
            self.get_mission_path_point(&mut next_pt, mission, next_pt_idx as usize, path_idx);

            next_pt[1] += if flags.contains(FollowPathFlags::Unk_0x8) {
                prop.get_aabb_max_x()
            } else {
                prop.get_aabb_max_y()
            };

            motion.set_target_point(&next_pt);

            true
        } else {
            prop.end_motion();
            false
        }
    }

    /// Sets the target point index of `motion` to the nearest or second-nearest point along
    /// the path already loaded in `motion`. Used to initialize the target point of path-following
    /// motion types.
    /// offset: 0x372c0
    // TODO_REFACTOR: make this take `path_idx` as input and return the `Option<u16>` target point
    // idx as output
    pub fn load_initial_target_point_idx(
        &self,
        motion: &mut impl PathMotion,
        prop: &mut Prop,
        mission: Mission,
    ) {
        static UNINIT: u16 = u16::MAX;

        let path_idx = motion.get_path_idx() as usize;

        if let Some(path) = self.get_mission_path(mission, path_idx) {
            let pos = prop.pos;
            let mut path_pt = self.points[path.point_idx as usize];

            // initialize the min dist to the point on the path with index 0
            let mut min_dist = path_pt_dist!(path_pt, pos);
            let mut min_pt_idx = path.point_idx as u16;

            println!("??????? {pos:?} {min_dist}");

            // leave the second smallest dist "uninitialized"
            let mut second_dist = 0.0;
            let mut second_pt_idx = UNINIT;

            // start the iteration variable `pt_idx` at the index-1 point, since we already
            // accounted for the 0th point above.
            let mut pt_idx = (path.point_idx as u16) + 1;
            loop {
                path_pt = self.points[pt_idx as usize];

                // reached the end of the list; decide on a result
                if path_pt[3] == Self::END_POINT_LIST_SENTINEL {
                    let mut result = second_pt_idx;

                    // TODO_REFACTOR: it seems like this can probably be simplified
                    if min_pt_idx == 0 {
                        if second_pt_idx != 1 {
                            result = min_pt_idx;
                        }
                    } else if second_pt_idx == 0 {
                        if min_pt_idx == 1 {
                            result = min_pt_idx;
                        }
                    } else if second_pt_idx < min_pt_idx {
                        result = min_pt_idx;
                    }

                    motion.set_target_point_idx(result - path.point_idx as u16);
                    return;
                }

                let dist = path_pt_dist!(path_pt, pos);
                if dist >= min_dist {
                    // not a new min dist; check if it's a new second smallest dist
                    if second_pt_idx == UNINIT || dist < second_dist {
                        second_dist = dist;
                        second_pt_idx = pt_idx;
                    }
                } else {
                    // new min dist; update min dist, and new second smallest dist is old min dist
                    second_dist = min_dist;
                    second_pt_idx = min_pt_idx;
                    min_dist = dist;
                    min_pt_idx = pt_idx;
                }

                println!("dist={dist}, min_dist={min_dist}, min_idx={}, 2nd_dist={second_dist}, 2nd_idx={}, pt={:?}", min_pt_idx-path.point_idx as u16, second_pt_idx - path.point_idx as u16, path_pt);

                pt_idx += 1;
            }
        } else {
            return prop.end_motion();
        }
    }
}

lazy_static! {
    /// Global static interface for prop path data.
    pub static ref PROP_PATH_DATA: PropPathData = PropPathData::from_bin();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_last_point_idx() {
        assert_eq!(
            Some(168),
            PROP_PATH_DATA.get_last_path_point_idx(Mission::Eternal3, 16)
        );
        assert_eq!(
            Some(12),
            PROP_PATH_DATA.get_last_path_point_idx(Mission::Eternal3, 49)
        );
    }
}
