use circular_queue::CircularQueue;
use gl_matrix::common::Vec3;

/// An element of the surface contact history, which records the number of
/// contacts and the closest normal contact for both walls and floors.
/// This data is recorded each tick and used to detect when the katamari might
/// be stuck.
/// offset: 0xb3250 (array of 20; 10 for each player)
#[derive(Debug, Default)]
pub struct HitHistoryElt {
    /// The number of katamari-to-wall contacts on a previous tick.
    /// offset: 0x0
    pub num_wall_contacts: u8,

    /// The number of katamari-to-floor contacts on a previous tick.
    /// offset: 0x2
    pub num_floor_contacts: u8,

    /// The contact wall unit normal vector.
    /// offset: 0x8
    pub wall_normal_unit: Vec3,

    /// The contact floor unit normal vector.
    /// offset: 0x18
    pub floor_normal_unit: Vec3,
}

#[derive(Debug)]
pub struct HitHistory {
    data: CircularQueue<HitHistoryElt>,
}

impl HitHistory {
    const CAPACITY: u8 = 10;

    /// Push an element to the history.
    pub fn push(
        &mut self,
        num_wall_contacts: u8,
        num_floor_contacts: u8,
        wall_normal_unit: &Vec3,
        floor_normal_unit: &Vec3,
    ) {
        self.data.push(HitHistoryElt {
            num_wall_contacts,
            num_floor_contacts,
            wall_normal_unit: wall_normal_unit.clone(),
            floor_normal_unit: floor_normal_unit.clone(),
        });
    }

    /// Read the `i`-th most recent element in the history.
    pub fn get(&self, i: usize) -> Option<&HitHistoryElt> {
        for (idx, data) in self.data.iter().enumerate() {
            if idx == i {
                return Some(data);
            }
        }

        None
    }
}

impl Default for HitHistory {
    fn default() -> Self {
        Self {
            data: CircularQueue::with_capacity(Self::CAPACITY as usize),
        }
    }
}
