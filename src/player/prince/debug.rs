use super::Prince;

impl Prince {
    pub fn debug_huff_state(&self) -> String {
        format!(
            "ih_0x9d={}, id_0x9e={}, ih_0x482={}, ht_0x480={}, ht_0x486={}",
            self.is_huffing_0x9d,
            self.is_huffing_0x9e,
            self.is_huffing_0x482,
            self.huff_timer_0x480,
            self.huff_timer_0x486
        )
    }
}
