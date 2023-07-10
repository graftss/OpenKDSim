use crate::global::GlobalState;

use self::data::RANDOM_GROUP_CONFIG;

mod data;

/// Contains all random groups present within a single mission.
#[derive(Debug, Default)]
pub struct MissionRandomConfig {
    groups: Vec<RandomGroupConfig>,
}

/// A single random group, which contains one or more items that are randomly
/// sampled to populate the group when props are loaded.
#[derive(Debug, Default)]
pub struct RandomGroupConfig {
    items: Vec<RandomGroupItem>,
}

/// An item in a random group.
#[derive(Debug, Default)]
pub struct RandomGroupItem {
    /// The maximum number of this item that may be spawned within its group.
    quantity: u16,

    /// The name index of the prop corresponding to this item.
    name_idx: u16,

    /// The spawn weight of this item. The max value is 256.
    spawn_weight: u16,
}

#[derive(Debug, Default)]
pub struct RandomPropGroup {
    /// offset: 0x0
    initialized: bool,

    /// The number of props in the mission which will be randomly selected from this group.
    /// offset: 0x1
    num_props: u16,

    /// The total number of objects in the pool which are available to be randomly spawned.
    /// offset: 0x3
    pool_size: u16,

    pool: Vec<PoolItem>,
}

#[derive(Debug, Default)]
struct PoolItem {
    remaining: u16,
    name_idx: u16,
    can_spawn: bool,
}

impl PoolItem {
    fn spawn(&mut self) {
        self.remaining -= 1;
        if self.remaining == 0 {
            self.can_spawn = false;
        }
    }
}

impl RandomPropGroup {
    /// Initialize the random group at index `group_idx`.
    /// offset: 0x2d670
    pub fn init(&mut self, global: &mut GlobalState, mission_idx: usize, group_idx: usize) {
        let group_config = &RANDOM_GROUP_CONFIG[mission_idx].groups[group_idx];

        self.initialized = true;
        self.num_props = 0;
        self.pool_size = 0;

        for item in group_config.items.iter() {
            let mut pool_item = PoolItem {
                remaining: item.quantity,
                name_idx: item.name_idx,
                can_spawn: false,
            };

            let should_spawn = match item.spawn_weight {
                // a max spawn weight of 0x100 means the item should always spawn
                // in the group
                0x100 => true,

                // if the spawn weight is below the max, roll rng1 against the weight
                // to see if it should spawn at all
                spawn_chance => (global.rng.get_rng1() & 0xff) < (spawn_chance as u32),
            };

            if should_spawn {
                pool_item.can_spawn = true;
                self.pool_size += pool_item.remaining;
            }

            self.pool.push(pool_item);
        }
    }

    /// Sample a name index from this random group. Returns the resulting name index.
    /// offset: 0x2d780
    pub fn sample(&mut self, global: &mut GlobalState) -> Option<u16> {
        if self.pool_size == 0 {
            return None;
        }

        let pool_items = self.pool.len() as u8;
        let random_pool_idx = global.rng.get_rng2() % pool_items;
        let pool_item = &mut self.pool[random_pool_idx as usize];

        if pool_item.can_spawn {
            // if no items remain in the pool, can't spawn the prop
            if pool_item.remaining == 0 {
                self.num_props -= 1;
                return None;
            }

            // TODO_DOC: no clue what the purpose of this is
            if self.pool_size < self.num_props {
                if global.rng.get_rng2() & 1 == 0 {
                    self.num_props -= 1;
                    return None;
                }
            }

            // spawn an object of type `pool_item.name_idx`
            self.pool_size -= 1;
            pool_item.spawn();
            return Some(pool_item.name_idx);
        } else {
            // if we selected a pool item that can't spawn, look upward
            // through the rest of the pool items for the first one that *can* be spawned.
            let mut next_pool_idx = random_pool_idx;
            for _ in 1..pool_items {
                // choose the next pool index as the successor to the previous one.
                // if we hit the end of the pool, wrap back around to the start of the pool.
                next_pool_idx += 1;
                if next_pool_idx == pool_items {
                    next_pool_idx = 0;
                }

                // look for a pool item that both (1) can spawn, and (2) has a remaining slot
                let pool_item = &mut self.pool[next_pool_idx as usize];
                if pool_item.can_spawn && pool_item.remaining > 0 {
                    self.pool_size -= 1;
                    pool_item.spawn();
                    return Some(pool_item.name_idx);
                }
            }
        }

        self.num_props -= 1;
        return None;
    }
}

#[derive(Debug, Default)]
pub struct RandomPropsState {
    groups: Vec<RandomPropGroup>,
}

impl RandomPropsState {
    pub fn reset(&mut self) {
        // TODO_PARAM
        let MAX_RANDOM_GROUPS = 64;

        self.groups.clear();
        for _ in 0..MAX_RANDOM_GROUPS {
            self.groups.push(RandomPropGroup::default());
        }
    }

    pub fn record_random_prop(
        &mut self,
        global: &mut GlobalState,
        mission_idx: usize,
        group_idx: usize,
    ) {
        let group = &mut self.groups[group_idx];

        if !group.initialized {
            group.init(global, mission_idx, group_idx);
        }

        group.num_props += 1;
    }

    pub fn sample_group(&mut self, global: &mut GlobalState, group_idx: usize) -> Option<u16> {
        self.groups[group_idx].sample(global)
    }
}
