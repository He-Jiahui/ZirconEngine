use std::time::{Duration, Instant};

use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{
    ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1, ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
};

use super::super::bounded_json::BoundedJsonError;
use super::super::frame::encode_world_invalidations_payload;
use super::RuntimeDynamicSession;
use crate::scene::WorldQueryBudgetError;

impl RuntimeDynamicSession {
    /// Queries the session-owned runtime world through the transport-neutral DTO contract.
    pub(super) fn query_world(
        &self,
        query: WorldQuery,
    ) -> Result<WorldQueryResult, BoundedJsonError> {
        self.level
            .with_world(|world| {
                world.query_world_bounded(&query, ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1)
            })
            .map_err(|error| match error {
                WorldQueryBudgetError::EncodedBytes { observed, limit } => {
                    BoundedJsonError::EncodedBytes { observed, limit }
                }
                WorldQueryBudgetError::Items { observed, limit } => {
                    BoundedJsonError::Items { observed, limit }
                }
                WorldQueryBudgetError::NestingDepth { observed, limit } => {
                    BoundedJsonError::NestingDepth { observed, limit }
                }
                WorldQueryBudgetError::ProcessingTime { limit_micros } => {
                    BoundedJsonError::ProcessingTime { limit_micros }
                }
                WorldQueryBudgetError::Json(message) => BoundedJsonError::Json(message),
            })
    }

    /// Registers one revocable session-local world watch.
    pub(super) fn watch_world(&self, registration: WatchRegistration) -> WatchToken {
        self.level.watch_world(registration)
    }

    /// Revokes one session-local watch and reports whether it was still live.
    pub(super) fn unwatch_world(&self, token: WatchToken) -> bool {
        self.level.unwatch_world(token)
    }

    /// Seals every runtime fact observed since the previous serialized drain and retains the
    /// candidate batch until allocation registration commits the ABI output.
    pub(super) fn prepare_world_invalidation_output(
        &mut self,
    ) -> Result<Vec<u8>, BoundedJsonError> {
        if self.world_invalidation_output_in_flight {
            return Err(BoundedJsonError::Json(
                "runtime world invalidation output is already in flight".to_string(),
            ));
        }
        if self.pending_world_invalidation_output.is_none() {
            self.pending_world_invalidation_output = Some(self.level.drain_world_invalidations());
        }
        let bytes = if let Some(page) = self.world_invalidation_output_page.as_deref() {
            encode_world_invalidation_page(page)?
        } else {
            let pending = self
                .pending_world_invalidation_output
                .as_deref()
                .expect("pending world invalidation output was initialized");
            let (page, bytes) = build_largest_world_invalidation_page(pending)?;
            self.world_invalidation_output_page = Some(page);
            bytes
        };
        self.world_invalidation_output_in_flight = true;
        Ok(bytes)
    }

    pub(super) fn commit_world_invalidation_output(&mut self) {
        debug_assert!(self.world_invalidation_output_in_flight);
        let page = self
            .world_invalidation_output_page
            .take()
            .expect("an in-flight world invalidation output must retain its page");
        let pending = self
            .pending_world_invalidation_output
            .as_mut()
            .expect("an in-flight world invalidation output must retain its pending batches");
        commit_world_invalidation_page(pending, &page);
        if pending.is_empty() {
            self.pending_world_invalidation_output = None;
        }
        self.world_invalidation_output_in_flight = false;
    }

    pub(super) fn rollback_world_invalidation_output(&mut self) {
        debug_assert!(self.world_invalidation_output_in_flight);
        self.world_invalidation_output_in_flight = false;
    }
}

fn build_world_invalidation_page(
    pending: &[InvalidationBatch],
    max_items: usize,
) -> Vec<InvalidationBatch> {
    let mut remaining_items = max_items;
    let mut page = Vec::new();
    for batch in pending {
        if remaining_items == 0 {
            break;
        }
        remaining_items -= 1;
        let dirty_count = batch.dirty.len().min(remaining_items);
        remaining_items -= dirty_count;
        let fact_count = batch.facts.len().min(remaining_items);
        remaining_items -= fact_count;
        page.push(InvalidationBatch {
            generation: batch.generation,
            dirty: batch.dirty[..dirty_count].to_vec(),
            facts: batch.facts[..fact_count].to_vec(),
        });
        if dirty_count != batch.dirty.len() || fact_count != batch.facts.len() {
            break;
        }
    }
    page
}

fn build_largest_world_invalidation_page(
    pending: &[InvalidationBatch],
) -> Result<(Vec<InvalidationBatch>, Vec<u8>), BoundedJsonError> {
    let started = Instant::now();
    let max_items = ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1.max_items;
    let page = build_world_invalidation_page(pending, max_items);
    match encode_world_invalidation_page_at(&page, started) {
        Ok(bytes) => return Ok((page, bytes)),
        Err(error) if !deterministic_world_invalidation_failure(&error) => return Err(error),
        Err(_) => {}
    }

    let minimum_items = pending.first().map_or(1, |batch| {
        usize::from(!batch.dirty.is_empty() || !batch.facts.is_empty()) + 1
    });
    let first = build_world_invalidation_page(pending, minimum_items);
    let first_bytes = encode_world_invalidation_page_at(&first, started)?;
    let mut best = (first, first_bytes);
    let mut low = minimum_items.saturating_add(1);
    let mut high = max_items;
    while low < high {
        let candidate = low + (high - low) / 2;
        let page = build_world_invalidation_page(pending, candidate);
        match encode_world_invalidation_page_at(&page, started) {
            Ok(bytes) => {
                best = (page, bytes);
                low = candidate + 1;
            }
            Err(error) if deterministic_world_invalidation_failure(&error) => high = candidate,
            Err(error) => return Err(error),
        }
    }
    Ok(best)
}

fn encode_world_invalidation_page(page: &[InvalidationBatch]) -> Result<Vec<u8>, BoundedJsonError> {
    encode_world_invalidation_page_at(page, Instant::now())
}

fn encode_world_invalidation_page_at(
    page: &[InvalidationBatch],
    started: Instant,
) -> Result<Vec<u8>, BoundedJsonError> {
    check_world_invalidation_encoding_deadline(started)?;
    let bytes = if page.is_empty() {
        Ok(Vec::new())
    } else {
        encode_world_invalidations_payload(page)
    }?;
    check_world_invalidation_encoding_deadline(started)?;
    Ok(bytes)
}

fn check_world_invalidation_encoding_deadline(started: Instant) -> Result<(), BoundedJsonError> {
    let limit_micros = ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1.max_processing_time_micros;
    if started.elapsed() > Duration::from_micros(limit_micros) {
        return Err(BoundedJsonError::ProcessingTime { limit_micros });
    }
    Ok(())
}

fn deterministic_world_invalidation_failure(error: &BoundedJsonError) -> bool {
    matches!(
        error,
        BoundedJsonError::EncodedBytes { .. }
            | BoundedJsonError::Items { .. }
            | BoundedJsonError::NestingDepth { .. }
    )
}

fn commit_world_invalidation_page(
    pending: &mut Vec<InvalidationBatch>,
    page: &[InvalidationBatch],
) {
    for delivered in page {
        let source = pending
            .first_mut()
            .expect("a delivered invalidation fragment must retain its source batch");
        debug_assert_eq!(source.generation, delivered.generation);
        source.dirty.drain(..delivered.dirty.len());
        source.facts.drain(..delivered.facts.len());
        if source.dirty.is_empty() && source.facts.is_empty() {
            pending.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_invalidation_pages_commit_only_the_delivered_prefix() {
        let mut pending = vec![InvalidationBatch {
            generation: 7,
            dirty: vec![WatchToken::new(1), WatchToken::new(2), WatchToken::new(3)],
            facts: Vec::new(),
        }];

        let first = build_world_invalidation_page(&pending, 3);
        assert_eq!(first[0].dirty.len(), 2);
        commit_world_invalidation_page(&mut pending, &first);
        assert_eq!(pending[0].dirty, vec![WatchToken::new(3)]);

        let second = build_world_invalidation_page(&pending, 3);
        commit_world_invalidation_page(&mut pending, &second);
        assert!(pending.is_empty());
    }
}
