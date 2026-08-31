use std::time::{Duration, Instant};

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};
use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldFact, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{
    ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1, ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
};

use super::super::bounded_json::{self, BoundedJsonError};
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
            .with_world_and_replacement_epoch(|world, world_replacement_epoch| {
                world.query_world_bounded_at_replacement_epoch(
                    &query,
                    ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
                    world_replacement_epoch,
                )
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
                WorldQueryBudgetError::ReflectValue(message) => BoundedJsonError::Json(message),
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
            let mut pending = self.level.drain_world_invalidations();
            reverse_pending_world_invalidations(&mut pending);
            self.pending_world_invalidation_output = Some(pending);
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

fn reverse_pending_world_invalidations(pending: &mut [InvalidationBatch]) {
    for batch in pending.iter_mut() {
        batch.dirty.reverse();
        batch.facts.reverse();
    }
    pending.reverse();
}

struct BorrowedWorldInvalidationItems<'a, T> {
    items: &'a [T],
    len: usize,
}

impl<T> Serialize for BorrowedWorldInvalidationItems<'_, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.len))?;
        for item in self.items.iter().rev().take(self.len) {
            sequence.serialize_element(item)?;
        }
        sequence.end()
    }
}

struct BorrowedWorldInvalidationBatch<'a> {
    generation: u64,
    dirty: BorrowedWorldInvalidationItems<'a, WatchToken>,
    facts: BorrowedWorldInvalidationItems<'a, WorldFact>,
}

impl Serialize for BorrowedWorldInvalidationBatch<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = 1 + (self.dirty.len > 0) as usize + (self.facts.len > 0) as usize;
        let mut batch = serializer.serialize_struct("InvalidationBatch", field_count)?;
        batch.serialize_field("generation", &self.generation)?;
        if self.dirty.len > 0 {
            batch.serialize_field("dirty", &self.dirty)?;
        }
        if self.facts.len > 0 {
            batch.serialize_field("facts", &self.facts)?;
        }
        batch.end()
    }
}

struct BorrowedWorldInvalidationPage<'a> {
    pending: &'a [InvalidationBatch],
    max_items: usize,
}

impl<'a> BorrowedWorldInvalidationPage<'a> {
    fn new(pending: &'a [InvalidationBatch], max_items: usize) -> Self {
        Self { pending, max_items }
    }

    fn batches(&self) -> BorrowedWorldInvalidationBatchIter<'a> {
        BorrowedWorldInvalidationBatchIter {
            pending: self.pending.iter().rev(),
            remaining_items: self.max_items,
        }
    }

    fn item_count(&self) -> usize {
        self.batches()
            .map(|batch| 1 + batch.dirty.len + batch.facts.len)
            .sum()
    }

    fn is_empty(&self) -> bool {
        self.max_items == 0 || self.pending.is_empty()
    }
}

impl Serialize for BorrowedWorldInvalidationPage<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for batch in self.batches() {
            sequence.serialize_element(&batch)?;
        }
        sequence.end()
    }
}

struct BorrowedWorldInvalidationBatchIter<'a> {
    pending: std::iter::Rev<std::slice::Iter<'a, InvalidationBatch>>,
    remaining_items: usize,
}

impl<'a> Iterator for BorrowedWorldInvalidationBatchIter<'a> {
    type Item = BorrowedWorldInvalidationBatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_items == 0 {
            return None;
        }
        let batch = self.pending.next()?;
        self.remaining_items -= 1;
        let dirty_count = batch.dirty.len().min(self.remaining_items);
        self.remaining_items -= dirty_count;
        let fact_count = batch.facts.len().min(self.remaining_items);
        self.remaining_items -= fact_count;
        if dirty_count != batch.dirty.len() || fact_count != batch.facts.len() {
            self.remaining_items = 0;
        }
        Some(BorrowedWorldInvalidationBatch {
            generation: batch.generation,
            dirty: BorrowedWorldInvalidationItems {
                items: &batch.dirty,
                len: dirty_count,
            },
            facts: BorrowedWorldInvalidationItems {
                items: &batch.facts,
                len: fact_count,
            },
        })
    }
}

fn build_world_invalidation_page(
    pending: &[InvalidationBatch],
    max_items: usize,
) -> Vec<InvalidationBatch> {
    BorrowedWorldInvalidationPage::new(pending, max_items)
        .batches()
        .map(|batch| InvalidationBatch {
            generation: batch.generation,
            dirty: batch
                .dirty
                .items
                .iter()
                .rev()
                .take(batch.dirty.len)
                .copied()
                .collect(),
            facts: batch
                .facts
                .items
                .iter()
                .rev()
                .take(batch.facts.len)
                .cloned()
                .collect(),
        })
        .collect()
}

fn build_largest_world_invalidation_page(
    pending: &[InvalidationBatch],
) -> Result<(Vec<InvalidationBatch>, Vec<u8>), BoundedJsonError> {
    let started = Instant::now();
    let max_items = ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1.max_items;
    let (best_items, best_bytes) =
        match encode_borrowed_world_invalidation_page_at(pending, max_items, started) {
            Ok(bytes) => (max_items, bytes),
            Err(error) if !deterministic_world_invalidation_failure(&error) => return Err(error),
            Err(_) => {
                let minimum_items = pending.last().map_or(1, |batch| {
                    usize::from(!batch.dirty.is_empty() || !batch.facts.is_empty()) + 1
                });
                let mut best_items = minimum_items;
                let mut best_bytes =
                    encode_borrowed_world_invalidation_page_at(pending, minimum_items, started)?;
                let mut low = minimum_items.saturating_add(1);
                let mut high = max_items.saturating_sub(1);
                while low <= high {
                    let candidate = low + (high - low) / 2;
                    match encode_borrowed_world_invalidation_page_at(pending, candidate, started) {
                        Ok(bytes) => {
                            best_items = candidate;
                            best_bytes = bytes;
                            low = candidate.saturating_add(1);
                        }
                        Err(error) if deterministic_world_invalidation_failure(&error) => {
                            high = candidate.saturating_sub(1);
                        }
                        Err(error) => return Err(error),
                    }
                }
                (best_items, best_bytes)
            }
        };
    Ok((
        build_world_invalidation_page(pending, best_items),
        best_bytes,
    ))
}

fn encode_borrowed_world_invalidation_page_at(
    pending: &[InvalidationBatch],
    max_items: usize,
    started: Instant,
) -> Result<Vec<u8>, BoundedJsonError> {
    check_world_invalidation_encoding_deadline(started)?;
    let page = BorrowedWorldInvalidationPage::new(pending, max_items);
    let bytes = if page.is_empty() {
        Ok(Vec::new())
    } else {
        bounded_json::encode(&page, ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1, || {
            page.item_count()
        })
    }?;
    check_world_invalidation_encoding_deadline(started)?;
    Ok(bytes)
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
            .last_mut()
            .expect("a delivered invalidation fragment must retain its source batch");
        debug_assert_eq!(source.generation, delivered.generation);
        source
            .dirty
            .truncate(source.dirty.len() - delivered.dirty.len());
        source
            .facts
            .truncate(source.facts.len() - delivered.facts.len());
        if source.dirty.is_empty() && source.facts.is_empty() {
            pending.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::*;

    #[test]
    fn world_invalidation_pages_commit_only_the_delivered_prefix() {
        let mut pending = vec![InvalidationBatch {
            generation: 7,
            dirty: vec![WatchToken::new(1), WatchToken::new(2), WatchToken::new(3)],
            facts: Vec::new(),
        }];
        reverse_pending_world_invalidations(&mut pending);

        let first = build_world_invalidation_page(&pending, 3);
        assert_eq!(first[0].dirty.len(), 2);
        commit_world_invalidation_page(&mut pending, &first);
        assert_eq!(pending[0].dirty, vec![WatchToken::new(3)]);

        let second = build_world_invalidation_page(&pending, 3);
        commit_world_invalidation_page(&mut pending, &second);
        assert!(pending.is_empty());
    }

    #[test]
    fn world_invalidation_tail_queues_preserve_batch_and_item_order() {
        let original = vec![
            InvalidationBatch {
                generation: 7,
                dirty: vec![WatchToken::new(1), WatchToken::new(2)],
                facts: Vec::new(),
            },
            InvalidationBatch {
                generation: 8,
                dirty: vec![WatchToken::new(3), WatchToken::new(4)],
                facts: Vec::new(),
            },
        ];
        let mut pending = original.clone();
        reverse_pending_world_invalidations(&mut pending);

        let page = build_world_invalidation_page(&pending, 6);
        assert_eq!(page, original);
        commit_world_invalidation_page(&mut pending, &page);
        assert!(pending.is_empty());
    }

    #[test]
    fn borrowed_world_invalidation_pages_match_owned_wire_bytes() {
        let mut pending = vec![
            InvalidationBatch {
                generation: 7,
                dirty: vec![WatchToken::new(1), WatchToken::new(2)],
                facts: vec![
                    WorldFact::AssetReloadApplied(Default::default()),
                    WorldFact::AssetReloadApplied(Default::default()),
                ],
            },
            InvalidationBatch {
                generation: 8,
                dirty: vec![WatchToken::new(3), WatchToken::new(4)],
                facts: vec![WorldFact::AssetReloadApplied(Default::default())],
            },
        ];
        reverse_pending_world_invalidations(&mut pending);

        for max_items in 0..=9 {
            let owned = build_world_invalidation_page(&pending, max_items);
            let owned_bytes = encode_world_invalidation_page(&owned).unwrap();
            let borrowed_bytes =
                encode_borrowed_world_invalidation_page_at(&pending, max_items, Instant::now())
                    .unwrap();
            assert_eq!(borrowed_bytes, owned_bytes, "max_items={max_items}");
        }
    }

    #[test]
    fn world_invalidation_tail_queue_source_has_no_front_removal() {
        let source = include_str!("world_sync.rs");
        let start = source
            .find("fn commit_world_invalidation_page(")
            .expect("world invalidation commit owner");
        let end = source[start..]
            .find("#[cfg(test)]")
            .map(|offset| start + offset)
            .expect("world invalidation commit boundary");
        let commit = &source[start..end];

        assert!(commit.contains("pending.last_mut()"));
        assert!(commit.contains("pending.pop()"));
        assert_eq!(commit.matches(".truncate(").count(), 2);
        assert!(!commit.contains("remove(0)"));
        assert!(!commit.contains(".drain(.."));
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn world_invalidation_tail_queue_release_benchmark_evidence() {
        const ITEMS: usize = 20_000;
        const SAMPLE_PAIRS: usize = 21;

        let (batch_legacy_ns, batch_optimized_ns) = measure_batch_tail_queue(ITEMS, SAMPLE_PAIRS);
        write_tail_queue_evidence(
            "WORLD_INVALIDATION_BATCH_TAIL_QUEUE_BENCH_V1",
            ITEMS,
            &batch_legacy_ns,
            &batch_optimized_ns,
        );

        let (item_legacy_ns, item_optimized_ns) = measure_item_tail_queue(ITEMS, SAMPLE_PAIRS);
        write_tail_queue_evidence(
            "WORLD_INVALIDATION_ITEM_TAIL_QUEUE_BENCH_V1",
            ITEMS,
            &item_legacy_ns,
            &item_optimized_ns,
        );
    }

    fn measure_batch_tail_queue(items: usize, sample_pairs: usize) -> (Vec<u128>, Vec<u128>) {
        let mut legacy_samples_ns = Vec::with_capacity(sample_pairs);
        let mut optimized_samples_ns = Vec::with_capacity(sample_pairs);
        for sample_index in 0..sample_pairs {
            let mut legacy: Vec<_> = (0..items).collect();
            let mut optimized: Vec<_> = (0..items).rev().collect();
            let mut measure_legacy = || {
                let started = Instant::now();
                while !legacy.is_empty() {
                    black_box(legacy.remove(0));
                }
                legacy_samples_ns.push(started.elapsed().as_nanos());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                while let Some(batch) = optimized.pop() {
                    black_box(batch);
                }
                optimized_samples_ns.push(started.elapsed().as_nanos());
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }
        (legacy_samples_ns, optimized_samples_ns)
    }

    fn measure_item_tail_queue(items: usize, sample_pairs: usize) -> (Vec<u128>, Vec<u128>) {
        let mut legacy_samples_ns = Vec::with_capacity(sample_pairs);
        let mut optimized_samples_ns = Vec::with_capacity(sample_pairs);
        for sample_index in 0..sample_pairs {
            let mut legacy: Vec<_> = (0..items).collect();
            let mut optimized: Vec<_> = (0..items).rev().collect();
            let mut measure_legacy = || {
                let started = Instant::now();
                while !legacy.is_empty() {
                    let item = legacy[0];
                    drop(legacy.drain(..1));
                    black_box(item);
                }
                legacy_samples_ns.push(started.elapsed().as_nanos());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                while let Some(item) = optimized.last().copied() {
                    optimized.truncate(optimized.len() - 1);
                    black_box(item);
                }
                optimized_samples_ns.push(started.elapsed().as_nanos());
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }
        (legacy_samples_ns, optimized_samples_ns)
    }

    fn write_tail_queue_evidence(
        marker: &str,
        items: usize,
        legacy_samples_ns: &[u128],
        optimized_samples_ns: &[u128],
    ) {
        assert_eq!(legacy_samples_ns.len(), optimized_samples_ns.len());
        let sample_pairs = legacy_samples_ns.len();
        let legacy_p95_ns = nearest_rank_percentile(legacy_samples_ns, 95);
        let optimized_p95_ns = nearest_rank_percentile(optimized_samples_ns, 95);
        let legacy = join_nanosecond_samples(legacy_samples_ns);
        let optimized = join_nanosecond_samples(optimized_samples_ns);
        let legacy_moves = (items as u128)
            .checked_mul((items - 1) as u128)
            .and_then(|moves| moves.checked_div(2))
            .unwrap();
        println!(
            "{marker} items={items} sample_pairs={sample_pairs} legacy_moves={legacy_moves} \
             optimized_moves=0 legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_ns={legacy} optimized_ns={optimized}"
        );
        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
            "optimized P95 {optimized_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn join_nanosecond_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        assert!((1..=100).contains(&percentile));
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let index = (ordered.len() * percentile).div_ceil(100) - 1;
        ordered[index]
    }
}
