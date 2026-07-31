const WORD_BITS: usize = u64::BITS as usize;

/// Ordered integer set with a fixed machine-word hierarchy.
///
/// The hierarchy has at most `ceil(usize::BITS / log2(WORD_BITS))` levels, so
/// insert/pop work is bounded independently of the feature count.
pub(super) struct OrderedReadySet {
    levels: Vec<Vec<u64>>,
    len: usize,
}

impl OrderedReadySet {
    pub(super) fn new(capacity: usize) -> Self {
        let mut levels = Vec::new();
        let mut word_count = capacity.div_ceil(WORD_BITS);
        while word_count > 0 {
            levels.push(vec![0; word_count]);
            if word_count == 1 {
                break;
            }
            word_count = word_count.div_ceil(WORD_BITS);
        }
        Self { levels, len: 0 }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(super) fn insert(&mut self, index: usize) -> bool {
        let Some(base_level) = self.levels.first() else {
            return false;
        };
        let word_index = index / WORD_BITS;
        if word_index >= base_level.len() {
            return false;
        }
        let bit = 1_u64 << (index % WORD_BITS);
        if base_level[word_index] & bit != 0 {
            return false;
        }

        let mut item_index = index;
        for level in &mut self.levels {
            let word_index = item_index / WORD_BITS;
            let bit = 1_u64 << (item_index % WORD_BITS);
            let was_empty = level[word_index] == 0;
            level[word_index] |= bit;
            if !was_empty {
                break;
            }
            item_index = word_index;
        }
        self.len += 1;
        true
    }

    pub(super) fn pop_first(&mut self) -> Option<usize> {
        if self.len == 0 {
            return None;
        }
        let top_level = self.levels.len() - 1;
        let mut item_index = self.levels[top_level][0].trailing_zeros() as usize;
        for level_index in (0..top_level).rev() {
            let word = self.levels[level_index][item_index];
            item_index = item_index * WORD_BITS + word.trailing_zeros() as usize;
        }
        self.remove(item_index);
        Some(item_index)
    }

    fn remove(&mut self, index: usize) {
        let mut item_index = index;
        for level in &mut self.levels {
            let word_index = item_index / WORD_BITS;
            let bit = 1_u64 << (item_index % WORD_BITS);
            level[word_index] &= !bit;
            if level[word_index] != 0 {
                break;
            }
            item_index = word_index;
        }
        self.len -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::OrderedReadySet;

    #[test]
    fn pops_sparse_indices_in_stable_order() {
        let mut set = OrderedReadySet::new(1_000);
        for index in [999, 64, 511, 0, 65, 511] {
            set.insert(index);
        }

        assert_eq!(
            std::iter::from_fn(|| set.pop_first()).collect::<Vec<_>>(),
            [0, 64, 65, 511, 999]
        );
    }
}
