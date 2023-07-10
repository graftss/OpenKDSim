use crate::macros::panic_log;

#[derive(Debug, Default)]
pub struct KingCommentState {
    /// The number of props in each comment group, indexed by their `comment_group_id`.
    /// offset: 0x155250
    pub group_sizes: Vec<u16>,

    /// The number of props in each comment group that are attached to the katamari,
    /// indexed by their `comment_group_id`.
    /// offset: 0x1530bc
    pub group_attached_counts: Vec<u16>,
}

impl KingCommentState {
    /// Initialize the king comment state by resetting all comment group counts to 0.
    pub fn reset(&mut self) {
        // TODO_PARAM
        let MAX_COMMENT_GROUPS = 64;

        self.group_sizes.clear();
        self.group_attached_counts.clear();

        for _ in 0..MAX_COMMENT_GROUPS {
            self.group_sizes.push(0);
            self.group_attached_counts.push(0);
        }
    }

    /// Add an item to the group `group_idx`.
    pub fn add_to_group(&mut self, group_id: u16) {
        let group_idx = group_id as usize;

        if group_idx >= self.group_sizes.len() {
            panic_log!("attempted to add to invalid comment group: {group_idx}");
        }

        self.group_sizes[group_idx] += 1;
    }
}
