use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextFrameDedupReport {
    pub(crate) frame_index: u64,
    pub(crate) entry_count: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) clear_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct TextFrameDedupEntry<K, V> {
    key: K,
    text: Arc<str>,
    value: V,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextFrameDedup<K, V> {
    entries: Vec<TextFrameDedupEntry<K, V>>,
    frame_report: TextFrameDedupReport,
}

impl<K, V> Default for TextFrameDedup<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            frame_report: TextFrameDedupReport::default(),
        }
    }
}

impl<K, V> TextFrameDedup<K, V>
where
    K: Eq,
{
    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.entries.clear();
        self.frame_report = TextFrameDedupReport {
            frame_index,
            ..TextFrameDedupReport::default()
        };
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn report(&self) -> TextFrameDedupReport {
        let mut report = self.frame_report;
        report.entry_count = self.entries.len();
        report
    }

    pub(crate) fn get(&mut self, key: &K, text: &str) -> Option<&V> {
        let mut collision_seen = false;

        for entry in &self.entries {
            if &entry.key != key {
                continue;
            }
            if entry.text.as_ref() == text {
                self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
                return Some(&entry.value);
            }
            collision_seen = true;
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        if collision_seen {
            self.frame_report.collision_miss_count =
                self.frame_report.collision_miss_count.saturating_add(1);
        }
        None
    }

    pub(crate) fn insert(&mut self, key: K, text: impl Into<Arc<str>>, value: V) -> &V {
        let text = text.into();
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| &entry.key == &key && entry.text.as_ref() == text.as_ref())
        {
            self.entries[index].value = value;
            self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
            return &self.entries[index].value;
        }

        self.entries.push(TextFrameDedupEntry { key, text, value });
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.refresh_report_size();
        &self.entries.last().expect("entry was just pushed").value
    }

    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        text: impl Into<Arc<str>>,
        produce: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let text = text.into();
        let mut collision_seen = false;

        for (index, entry) in self.entries.iter().enumerate() {
            if &entry.key != &key {
                continue;
            }
            if entry.text.as_ref() == text.as_ref() {
                self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
                return (&self.entries[index].value, false);
            }
            collision_seen = true;
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        if collision_seen {
            self.frame_report.collision_miss_count =
                self.frame_report.collision_miss_count.saturating_add(1);
        }
        self.entries.push(TextFrameDedupEntry {
            key,
            text,
            value: produce(),
        });
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.refresh_report_size();
        (
            &self.entries.last().expect("entry was just pushed").value,
            true,
        )
    }

    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.entries.len();
    }
}
