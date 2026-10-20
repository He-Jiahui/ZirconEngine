//! Bounded per-thread sharing for persistent frame-report labels.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

const MAX_LABELS: usize = 2048;
const MAX_RETAINED_LABEL_BYTES: usize = 256;

#[derive(Default)]
struct LabelCache {
    entries: BTreeMap<Arc<str>, u64>,
    clock: u64,
}

thread_local! {
    static LABELS: RefCell<LabelCache> = RefCell::new(LabelCache::default());
}

/// Owned report text whose clones share the bytes, including across frame records.
#[derive(Clone, Eq, PartialOrd, Ord, Hash)]
pub struct RenderReportLabel(Arc<str>);

impl RenderReportLabel {
    pub fn new(value: impl AsRef<str>) -> Self {
        let value = value.as_ref();
        if value.len() > MAX_RETAINED_LABEL_BYTES {
            return Self(Arc::from(value));
        }
        LABELS.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.clock = cache.clock.saturating_add(1);
            let stamp = cache.clock;
            if let Some((label, _)) = cache.entries.get_key_value(value) {
                let label = Arc::clone(label);
                *cache.entries.get_mut(value).expect("label was just found") = stamp;
                return Self(label);
            }
            if cache.entries.len() == MAX_LABELS {
                let oldest = cache
                    .entries
                    .iter()
                    .min_by_key(|(_, used)| **used)
                    .map(|(label, _)| Arc::clone(label))
                    .expect("bounded cache is full");
                cache.entries.remove(oldest.as_ref());
            }
            let label: Arc<str> = Arc::from(value);
            cache.entries.insert(Arc::clone(&label), stamp);
            Self(label)
        })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for RenderReportLabel {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<str> for RenderReportLabel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl<T: AsRef<str> + ?Sized> PartialEq<T> for RenderReportLabel {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}
impl fmt::Display for RenderReportLabel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl fmt::Debug for RenderReportLabel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), formatter)
    }
}
impl From<&str> for RenderReportLabel {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<String> for RenderReportLabel {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl From<&String> for RenderReportLabel {
    fn from(value: &String) -> Self {
        Self::new(value)
    }
}
impl From<&RenderReportLabel> for RenderReportLabel {
    fn from(value: &RenderReportLabel) -> Self {
        value.clone()
    }
}
impl From<RenderReportLabel> for String {
    fn from(value: RenderReportLabel) -> Self {
        value.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_report_labels_share_bytes_and_survive_cache_eviction() {
        let first = RenderReportLabel::new("report.test.pipeline");
        let again = RenderReportLabel::new("report.test.pipeline");
        assert!(Arc::ptr_eq(&first.0, &again.0));
        for index in 0..=MAX_LABELS {
            RenderReportLabel::new(format!("report.test.unique.{index}"));
        }
        assert_eq!(first.as_str(), "report.test.pipeline");
        LABELS.with(|cache| assert_eq!(cache.borrow().entries.len(), MAX_LABELS));
    }

    #[test]
    fn oversized_dynamic_labels_do_not_enter_the_retained_cache() {
        let text = "x".repeat(MAX_RETAINED_LABEL_BYTES + 1);
        let first = RenderReportLabel::new(&text);
        let again = RenderReportLabel::new(&text);
        assert!(!Arc::ptr_eq(&first.0, &again.0));
        assert_eq!(first, text);
    }
}
