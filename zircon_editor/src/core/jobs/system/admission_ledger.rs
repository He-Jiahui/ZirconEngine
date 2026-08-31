use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use super::super::{
    EditorJobAdmissionLimits, EditorJobAdmissionRequest, EditorJobAdmissionSnapshot,
    EditorJobLimits, EditorJobSpec, JobCategory, JobId, JobSubmitError,
};
use super::EditorJobAdmissionWindow;

pub(super) struct PendingAdmissionReservation {
    pub(super) id: JobId,
    pub(super) request: EditorJobAdmissionRequest,
    pub(super) admitted_at: Instant,
}

struct PendingAdmissionEntry {
    category: JobCategory,
    estimated_bytes: usize,
    admitted_at: Instant,
}

/// Capacity and observability state for executable jobs and reservations.
///
/// This ledger deliberately knows nothing about task execution, dependencies, or
/// ready-queue promotion. A reservation occupies the same accounting entries as
/// a materialized pending job until it is committed or released.
#[derive(Default)]
pub(super) struct PendingAdmissionLedger {
    reservations: BTreeMap<u64, Vec<PendingAdmissionReservation>>,
    entries: BTreeMap<JobId, PendingAdmissionEntry>,
    estimated_bytes: usize,
    pending_ids_by_category: BTreeMap<JobCategory, BTreeSet<JobId>>,
    estimated_bytes_by_category: BTreeMap<JobCategory, usize>,
    merged_submissions: u64,
    cancelled_pending: u64,
    started_pending: u64,
    merged_submissions_by_category: BTreeMap<JobCategory, u64>,
    cancelled_pending_by_category: BTreeMap<JobCategory, u64>,
    started_pending_by_category: BTreeMap<JobCategory, u64>,
}

impl PendingAdmissionLedger {
    pub(super) fn insert(
        &mut self,
        id: JobId,
        category: JobCategory,
        estimated_bytes: usize,
        admitted_at: Instant,
    ) {
        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        self.pending_ids_by_category
            .entry(category)
            .or_default()
            .insert(id);
        let category_bytes = self
            .estimated_bytes_by_category
            .entry(category)
            .or_default();
        *category_bytes = category_bytes.saturating_add(estimated_bytes);
        let replaced = self.entries.insert(
            id,
            PendingAdmissionEntry {
                category,
                estimated_bytes,
                admitted_at,
            },
        );
        debug_assert!(
            replaced.is_none(),
            "pending admission ids must remain unique across jobs and reservations"
        );
    }

    pub(super) fn remove(&mut self, id: JobId) -> Option<(JobCategory, usize)> {
        let entry = self.entries.remove(&id)?;
        self.estimated_bytes = self.estimated_bytes.saturating_sub(entry.estimated_bytes);
        remove_from_set_map(&mut self.pending_ids_by_category, &entry.category, id);
        subtract_category_bytes(
            &mut self.estimated_bytes_by_category,
            entry.category,
            entry.estimated_bytes,
        );
        Some((entry.category, entry.estimated_bytes))
    }

    pub(super) fn reserve_batch(
        &mut self,
        reservation_id: u64,
        reservations: Vec<PendingAdmissionReservation>,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        if self.reservations.contains_key(&reservation_id) {
            return Err(JobSubmitError::AdmissionReservationMismatch);
        }
        self.ensure_reservation_batch_admissible_iter(
            reservations.iter().map(|reservation| &reservation.request),
            limits,
            now,
        )?;

        for reservation in &reservations {
            self.insert(
                reservation.id,
                reservation.request.category,
                reservation.request.estimated_pending_bytes,
                reservation.admitted_at,
            );
        }
        let replaced = self.reservations.insert(reservation_id, reservations);
        debug_assert!(
            replaced.is_none(),
            "pending admission reservation ids must remain unique"
        );
        Ok(())
    }

    pub(super) fn ensure_reservation_batch_admissible(
        &self,
        requests: &[&EditorJobAdmissionRequest],
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.ensure_reservation_batch_admissible_iter(requests.iter().copied(), limits, now)
    }

    fn ensure_reservation_batch_admissible_iter<'a>(
        &self,
        requests: impl Clone + ExactSizeIterator<Item = &'a EditorJobAdmissionRequest>,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        if requests.len() == 0 {
            return Err(JobSubmitError::AdmissionReservationMismatch);
        }
        for request in requests.clone() {
            self.ensure_oldest_age_limit(
                request
                    .max_pending_age
                    .unwrap_or(limits.max_oldest_pending_age),
                now,
            )?;
        }
        if self.entries.len().saturating_add(requests.len()) > limits.max_pending_entries {
            return Err(JobSubmitError::AdmissionEntryLimitExceeded {
                limit: limits.max_pending_entries,
            });
        }
        let requested = requests.fold(0_usize, |total, request| {
            total.saturating_add(request.estimated_pending_bytes)
        });
        if self.estimated_bytes.saturating_add(requested) > limits.max_pending_estimated_bytes {
            return Err(JobSubmitError::AdmissionByteLimitExceeded {
                limit: limits.max_pending_estimated_bytes,
                current: self.estimated_bytes,
                requested,
            });
        }

        Ok(())
    }

    pub(super) fn commit_reservation(
        &mut self,
        reservation_id: u64,
        specs: &[&EditorJobSpec],
    ) -> Result<Vec<(JobId, Instant)>, JobSubmitError> {
        let Some(reservations) = self.reservations.get(&reservation_id) else {
            return Err(JobSubmitError::AdmissionReservationMismatch);
        };
        if reservations.len() != specs.len()
            || reservations
                .iter()
                .zip(specs)
                .any(|(reservation, spec)| !reservation_matches_spec(reservation, spec))
        {
            return Err(JobSubmitError::AdmissionReservationMismatch);
        }

        let reservations = self
            .reservations
            .remove(&reservation_id)
            .expect("checked admission reservation must remain present");
        let mut committed = Vec::with_capacity(reservations.len());
        for reservation in reservations {
            let (category, estimated_bytes) = self
                .remove(reservation.id)
                .expect("admission reservation entries must remain indexed");
            debug_assert_eq!(category, reservation.request.category);
            debug_assert_eq!(estimated_bytes, reservation.request.estimated_pending_bytes);
            committed.push((reservation.id, reservation.admitted_at));
        }
        Ok(committed)
    }

    pub(super) fn release_reservation(&mut self, reservation_id: u64) -> bool {
        let Some(reservations) = self.reservations.remove(&reservation_id) else {
            return false;
        };
        for reservation in reservations {
            self.remove(reservation.id)
                .expect("admission reservation entries must remain indexed");
        }
        true
    }

    pub(super) fn release_all_reservations(&mut self) {
        let reservation_ids = self.reservations.keys().copied().collect::<Vec<_>>();
        for reservation_id in reservation_ids {
            self.release_reservation(reservation_id);
        }
    }

    pub(super) fn ensure_admissible(
        &self,
        spec: &EditorJobSpec,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.ensure_oldest_age(spec, limits, now)?;
        if self.entries.len() >= limits.max_pending_entries {
            return Err(JobSubmitError::AdmissionEntryLimitExceeded {
                limit: limits.max_pending_entries,
            });
        }
        if self
            .estimated_bytes
            .saturating_add(spec.estimated_pending_bytes)
            > limits.max_pending_estimated_bytes
        {
            return Err(JobSubmitError::AdmissionByteLimitExceeded {
                limit: limits.max_pending_estimated_bytes,
                current: self.estimated_bytes,
                requested: spec.estimated_pending_bytes,
            });
        }
        Ok(())
    }

    pub(super) fn ensure_batch_admissible(
        &self,
        specs: &[&EditorJobSpec],
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        for spec in specs {
            self.ensure_oldest_age(spec, limits, now)?;
        }
        if self.entries.len().saturating_add(specs.len()) > limits.max_pending_entries {
            return Err(JobSubmitError::AdmissionEntryLimitExceeded {
                limit: limits.max_pending_entries,
            });
        }
        let requested = specs.iter().fold(0_usize, |total, spec| {
            total.saturating_add(spec.estimated_pending_bytes)
        });
        if self.estimated_bytes.saturating_add(requested) > limits.max_pending_estimated_bytes {
            return Err(JobSubmitError::AdmissionByteLimitExceeded {
                limit: limits.max_pending_estimated_bytes,
                current: self.estimated_bytes,
                requested,
            });
        }
        Ok(())
    }

    pub(super) fn pending_admission_window(
        &self,
        limits: &EditorJobLimits,
        now: Instant,
    ) -> Result<EditorJobAdmissionWindow, JobSubmitError> {
        let limits = limits.admission_limits();
        self.ensure_oldest_age_limit(limits.max_oldest_pending_age, now)?;
        if self.entries.len() >= limits.max_pending_entries {
            return Err(JobSubmitError::AdmissionEntryLimitExceeded {
                limit: limits.max_pending_entries,
            });
        }
        Ok(EditorJobAdmissionWindow::new(
            self.entries.len(),
            limits.max_pending_entries,
            limits
                .max_pending_entries
                .saturating_sub(self.entries.len()),
            limits
                .max_pending_estimated_bytes
                .saturating_sub(self.estimated_bytes),
            self.estimated_bytes,
            limits.max_pending_estimated_bytes,
        ))
    }

    pub(super) fn admission_snapshot(&self, now: Instant) -> EditorJobAdmissionSnapshot {
        self.admission_snapshot_for(now, None)
    }

    pub(super) fn category_admission_snapshot(
        &self,
        category: JobCategory,
        now: Instant,
    ) -> EditorJobAdmissionSnapshot {
        self.admission_snapshot_for(now, Some(category))
    }

    pub(super) fn ensure_replacement_admissible(
        &self,
        previous_bytes: usize,
        latest: &EditorJobSpec,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.ensure_oldest_age(latest, limits, now)?;
        let projected_bytes = self
            .estimated_bytes
            .saturating_sub(previous_bytes)
            .saturating_add(latest.estimated_pending_bytes);
        if projected_bytes > limits.max_pending_estimated_bytes {
            return Err(JobSubmitError::AdmissionByteLimitExceeded {
                limit: limits.max_pending_estimated_bytes,
                current: self.estimated_bytes.saturating_sub(previous_bytes),
                requested: latest.estimated_pending_bytes,
            });
        }
        Ok(())
    }

    pub(super) fn replace_bytes(
        &mut self,
        id: JobId,
        category: JobCategory,
        previous_bytes: usize,
        latest_bytes: usize,
    ) {
        self.estimated_bytes = self
            .estimated_bytes
            .saturating_sub(previous_bytes)
            .saturating_add(latest_bytes);
        let category_bytes = self
            .estimated_bytes_by_category
            .entry(category)
            .or_default();
        *category_bytes = category_bytes
            .saturating_sub(previous_bytes)
            .saturating_add(latest_bytes);
        let admission = self
            .entries
            .get_mut(&id)
            .expect("merged pending jobs must retain one admission entry");
        debug_assert_eq!(admission.category, category);
        admission.estimated_bytes = latest_bytes;
    }

    pub(super) fn record_merged(&mut self, category: JobCategory) {
        self.merged_submissions = self.merged_submissions.saturating_add(1);
        increment_category_counter(&mut self.merged_submissions_by_category, category);
    }

    pub(super) fn record_cancelled(&mut self, category: JobCategory) {
        self.cancelled_pending = self.cancelled_pending.saturating_add(1);
        increment_category_counter(&mut self.cancelled_pending_by_category, category);
    }

    pub(super) fn record_started(&mut self, category: JobCategory) {
        self.started_pending = self.started_pending.saturating_add(1);
        increment_category_counter(&mut self.started_pending_by_category, category);
    }

    fn admission_snapshot_for(
        &self,
        now: Instant,
        category: Option<JobCategory>,
    ) -> EditorJobAdmissionSnapshot {
        if let Some(category) = category {
            let ids = self.pending_ids_by_category.get(&category);
            let entries = ids.map_or(0, BTreeSet::len);
            let estimated_bytes = self
                .estimated_bytes_by_category
                .get(&category)
                .copied()
                .unwrap_or_default();
            let oldest_pending_age = ids
                .and_then(|ids| ids.first())
                .and_then(|id| self.entries.get(id))
                .map(|entry| now.saturating_duration_since(entry.admitted_at));
            return EditorJobAdmissionSnapshot::new(
                entries,
                estimated_bytes,
                oldest_pending_age,
                category_counter(&self.merged_submissions_by_category, category),
                category_counter(&self.cancelled_pending_by_category, category),
                category_counter(&self.started_pending_by_category, category),
            );
        }
        EditorJobAdmissionSnapshot::new(
            self.entries.len(),
            self.estimated_bytes,
            self.entries
                .values()
                .next()
                .map(|entry| now.saturating_duration_since(entry.admitted_at)),
            self.merged_submissions,
            self.cancelled_pending,
            self.started_pending,
        )
    }

    fn ensure_oldest_age(
        &self,
        spec: &EditorJobSpec,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        let max_pending_age = spec
            .max_pending_age
            .unwrap_or(limits.max_oldest_pending_age);
        self.ensure_oldest_age_limit(max_pending_age, now)
    }

    fn ensure_oldest_age_limit(
        &self,
        max_pending_age: Duration,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        if self.entries.values().next().is_some_and(|entry| {
            now.saturating_duration_since(entry.admitted_at) >= max_pending_age
        }) {
            return Err(JobSubmitError::OldestPendingAgeExceeded {
                max_age_ms: duration_millis(max_pending_age),
            });
        }
        Ok(())
    }
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn increment_category_counter(counters: &mut BTreeMap<JobCategory, u64>, category: JobCategory) {
    let counter = counters.entry(category).or_default();
    *counter = counter.saturating_add(1);
}

fn category_counter(counters: &BTreeMap<JobCategory, u64>, category: JobCategory) -> u64 {
    counters.get(&category).copied().unwrap_or_default()
}

fn subtract_category_bytes(
    bytes_by_category: &mut BTreeMap<JobCategory, usize>,
    category: JobCategory,
    bytes: usize,
) {
    let should_remove = bytes_by_category.get_mut(&category).is_some_and(|current| {
        *current = current.saturating_sub(bytes);
        *current == 0
    });
    if should_remove {
        bytes_by_category.remove(&category);
    }
}

fn reservation_matches_spec(
    reservation: &PendingAdmissionReservation,
    spec: &EditorJobSpec,
) -> bool {
    reservation.request.category == spec.category
        && reservation.request.priority == spec.priority
        && reservation.request.estimated_pending_bytes == spec.estimated_pending_bytes
        && reservation.request.max_pending_age == spec.max_pending_age
        && spec.admission_key.is_none()
        && !spec.label.trim().is_empty()
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
