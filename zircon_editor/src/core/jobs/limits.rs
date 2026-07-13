use std::collections::BTreeMap;

use super::JobCategory;

const DEFAULT_THUMBNAIL_LIMIT: usize = 2;
const DEFAULT_EXPORT_LIMIT: usize = 1;

#[derive(Clone, Debug)]
pub struct EditorJobLimits {
    limits: BTreeMap<JobCategory, usize>,
}

impl EditorJobLimits {
    pub fn with_limit(mut self, category: JobCategory, limit: usize) -> Self {
        self.limits.insert(category, limit.max(1));
        self
    }

    pub fn limit(&self, category: JobCategory) -> usize {
        self.limits.get(&category).copied().unwrap_or(usize::MAX)
    }

    pub(super) fn with_runtime_defaults(mut self, worker_parallelism: usize) -> Self {
        self.limits
            .entry(JobCategory::Import)
            .or_insert(worker_parallelism.max(1));
        self
    }
}

impl Default for EditorJobLimits {
    fn default() -> Self {
        Self {
            limits: BTreeMap::from([
                (JobCategory::Thumbnail, DEFAULT_THUMBNAIL_LIMIT),
                (JobCategory::Export, DEFAULT_EXPORT_LIMIT),
            ]),
        }
    }
}
