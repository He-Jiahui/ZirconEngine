use std::collections::{BTreeMap, BTreeSet};

use super::super::{EditorJobLimits, EditorJobSpec, JobCategory, JobContext, JobId, JobPriority};

pub(super) type PendingTask = Box<dyn FnOnce(JobContext) + Send + 'static>;
pub(super) type PendingCancelTask = Box<dyn FnOnce(JobContext) + Send + 'static>;

pub(super) const MAX_ADMISSION_BUCKET_PROBES_PER_PASS: usize =
    JobPriority::ALL.len() * JobCategory::ALL.len();

pub(super) struct PendingJob {
    pub(super) id: JobId,
    pub(super) spec: EditorJobSpec,
    pub(super) task: PendingTask,
    pub(super) cancel_task: PendingCancelTask,
}

impl PendingJob {
    pub(super) fn new(
        id: JobId,
        spec: EditorJobSpec,
        task: PendingTask,
        cancel_task: PendingCancelTask,
    ) -> Self {
        Self {
            id,
            spec,
            task,
            cancel_task,
        }
    }
}

#[derive(Default)]
pub(super) struct PendingJobQueue {
    jobs: BTreeMap<JobId, PendingJob>,
    ready: BTreeMap<(u8, JobCategory), BTreeSet<JobId>>,
    waiting_counts: BTreeMap<JobId, usize>,
    dependents_by_dependency: BTreeMap<JobId, BTreeSet<JobId>>,
    referenced_dependencies: BTreeMap<JobId, usize>,
    admission_probes: usize,
}

impl PendingJobQueue {
    pub(super) fn insert(&mut self, pending: PendingJob, unscheduled: &[JobId]) {
        let id = pending.id;
        for dependency in &pending.spec.after {
            *self.referenced_dependencies.entry(*dependency).or_default() += 1;
        }
        if unscheduled.is_empty() {
            self.ready
                .entry(queue_key(&pending.spec))
                .or_default()
                .insert(id);
        } else {
            self.waiting_counts.insert(id, unscheduled.len());
            for dependency in unscheduled {
                self.dependents_by_dependency
                    .entry(*dependency)
                    .or_default()
                    .insert(id);
            }
        }
        let replaced = self.jobs.insert(id, pending);
        debug_assert!(replaced.is_none(), "pending job ids must remain unique");
    }

    pub(super) fn remove(&mut self, id: JobId) -> Option<PendingJob> {
        let pending = self.jobs.remove(&id)?;
        let key = queue_key(&pending.spec);
        remove_from_set_map(&mut self.ready, &key, id);
        self.waiting_counts.remove(&id);
        for dependency in &pending.spec.after {
            remove_from_set_map(&mut self.dependents_by_dependency, dependency, id);
            decrement_count(&mut self.referenced_dependencies, dependency);
        }
        Some(pending)
    }

    pub(super) fn take_next(
        &mut self,
        limits: &EditorJobLimits,
        running_by_category: &BTreeMap<JobCategory, usize>,
    ) -> Option<PendingJob> {
        for priority in JobPriority::ALL {
            let mut selected = None;
            for category in JobCategory::ALL {
                self.admission_probes = self.admission_probes.saturating_add(1);
                if running_by_category
                    .get(&category)
                    .copied()
                    .unwrap_or_default()
                    >= limits.limit(category)
                {
                    continue;
                }
                let key = (priority.admission_rank(), category);
                let Some(id) = self.ready.get(&key).and_then(|ids| ids.first().copied()) else {
                    continue;
                };
                if selected.is_none_or(|selected_id| id < selected_id) {
                    selected = Some(id);
                }
            }
            if let Some(id) = selected {
                return self.remove(id);
            }
        }
        None
    }

    pub(super) fn mark_dependency_schedulable(&mut self, dependency: JobId) {
        let Some(dependents) = self.dependents_by_dependency.remove(&dependency) else {
            return;
        };
        let mut ready = Vec::new();
        for dependent in dependents {
            let Some(remaining) = self.waiting_counts.get_mut(&dependent) else {
                continue;
            };
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0 {
                ready.push(dependent);
            }
        }
        for id in ready {
            self.waiting_counts.remove(&id);
            if let Some(pending) = self.jobs.get(&id) {
                self.ready
                    .entry(queue_key(&pending.spec))
                    .or_default()
                    .insert(id);
            }
        }
    }

    pub(super) fn drain(&mut self) -> Vec<PendingJob> {
        let ids = self.jobs.keys().copied().collect::<Vec<_>>();
        ids.into_iter().filter_map(|id| self.remove(id)).collect()
    }

    pub(super) fn depends_on(&self, dependency: JobId) -> bool {
        self.referenced_dependencies.contains_key(&dependency)
    }

    pub(super) fn len(&self) -> usize {
        self.jobs.len()
    }

    #[cfg(test)]
    pub(super) fn admission_probe_count(&self) -> usize {
        self.admission_probes
    }
}

fn queue_key(spec: &EditorJobSpec) -> (u8, JobCategory) {
    (spec.priority.admission_rank(), spec.category)
}

fn remove_from_set_map<K: Ord + Clone>(map: &mut BTreeMap<K, BTreeSet<JobId>>, key: &K, id: JobId) {
    let should_remove = map.get_mut(key).is_some_and(|ids| {
        ids.remove(&id);
        ids.is_empty()
    });
    if should_remove {
        map.remove(key);
    }
}

fn decrement_count<K: Ord + Clone>(map: &mut BTreeMap<K, usize>, key: &K) {
    let should_remove = map.get_mut(key).is_some_and(|count| {
        *count = count.saturating_sub(1);
        *count == 0
    });
    if should_remove {
        map.remove(key);
    }
}
