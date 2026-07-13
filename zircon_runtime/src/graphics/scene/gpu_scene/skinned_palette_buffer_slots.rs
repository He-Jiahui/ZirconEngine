#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SkinnedPaletteBufferSlots {
    committed: Option<usize>,
    staged: Option<usize>,
}

impl SkinnedPaletteBufferSlots {
    pub(crate) fn stage(&mut self) -> usize {
        *self
            .staged
            .get_or_insert_with(|| self.committed.map(|slot| 1 - slot).unwrap_or(0))
    }

    pub(crate) fn committed(&self) -> Option<usize> {
        self.committed
    }

    pub(crate) fn commit_after_success(&mut self) {
        if let Some(staged) = self.staged.take() {
            self.committed = Some(staged);
        }
    }

    pub(crate) fn discard_staged(&mut self) {
        self.staged = None;
    }
}

#[cfg(test)]
mod tests {
    use super::SkinnedPaletteBufferSlots;

    #[test]
    fn palette_double_buffer_swaps_per_successful_frame() {
        let mut slots = SkinnedPaletteBufferSlots::default();

        assert_eq!(slots.stage(), 0);
        assert_eq!(slots.committed(), None);
        slots.commit_after_success();

        assert_eq!(slots.committed(), Some(0));
        assert_eq!(slots.stage(), 1);
        assert_eq!(slots.stage(), 1);
        slots.commit_after_success();

        assert_eq!(slots.committed(), Some(1));
        assert_eq!(slots.stage(), 0);
    }

    #[test]
    fn failed_frame_does_not_replace_committed_palette() {
        let mut slots = SkinnedPaletteBufferSlots::default();
        assert_eq!(slots.stage(), 0);
        slots.commit_after_success();

        assert_eq!(slots.stage(), 1);
        assert_eq!(slots.committed(), Some(0));
        assert_eq!(slots.stage(), 1);
    }

    #[test]
    fn missing_storage_discards_failed_frame_staging() {
        let mut slots = SkinnedPaletteBufferSlots::default();
        assert_eq!(slots.stage(), 0);
        slots.commit_after_success();

        assert_eq!(slots.stage(), 1);
        slots.discard_staged();
        slots.commit_after_success();

        assert_eq!(slots.committed(), Some(0));
        assert_eq!(slots.stage(), 1);
    }
}
