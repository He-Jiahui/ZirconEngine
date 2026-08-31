use std::collections::{BTreeSet, HashMap};
use std::hash::{BuildHasherDefault, Hasher};
use std::hint::black_box;
use std::mem::size_of;
use std::num::NonZeroU64;
use std::time::{Duration, Instant};

const RADIX_BITS: u32 = 8;
const FANOUT: usize = 1 << RADIX_BITS;
const PREFIX_PROMOTION_FACTOR: usize = 1_024;
const PAIRED_LOOKUP_SAMPLES: usize = 31;
const LOOKUP_REPEATS: usize = 8;
const DENSE_ENTITY_COUNT: u32 = 262_144;
const HIGH_ENTITY_INDEX: u32 = 4_000_000;
const SPARSE_HIGH_RANGE: u32 = 262_144;
const SPARSE_HIGH_STRIDE: u32 = 1_000;
const SPARSE_HIGH_HIT_REPETITIONS: usize = 1_024;
const DUAL_SPAN_ROWS: u32 = 1_024;
const DUAL_SPAN_QUERY_REPETITIONS: usize = 128;
const EXTREME_ENTITY_INDEX: u32 = u32::MAX - 1;
const EMPTY_PAGE_KEY: u32 = u32::MAX;
const PAGE_HASH_MULTIPLIER: u32 = 0x9E37_79B9;
const MIN_PAGE_BUCKETS: usize = 8;

#[derive(Clone, Copy)]
struct Location(NonZeroU64);

#[derive(Clone, Copy)]
struct LegacyLocation {
    generation: u32,
    dense_row: usize,
}

impl LegacyLocation {
    #[inline(always)]
    fn observation(self) -> u64 {
        (u64::from(self.generation) << 32) | self.dense_row as u64
    }
}

impl Location {
    #[inline(always)]
    fn new(generation: u32, dense_row: u32) -> Self {
        let stored_row = dense_row.checked_add(1).expect("dense row domain");
        let word = (u64::from(generation) << 32) | u64::from(stored_row);
        Self(NonZeroU64::new(word).expect("stored row keeps location non-zero"))
    }

    #[inline(always)]
    fn observation(self) -> u64 {
        let generation = self.0.get() >> 32;
        let dense_row = u64::from(self.0.get() as u32 - 1);
        (generation << 32) | dense_row
    }
}

#[derive(Default)]
struct LegacyLocator {
    rows: Vec<Option<LegacyLocation>>,
}

impl LegacyLocator {
    fn insert(&mut self, index: u32, location: Location) {
        let index = index as usize;
        if self.rows.len() <= index {
            self.rows.resize(index + 1, None);
        }
        self.rows[index] = Some(LegacyLocation {
            generation: (location.0.get() >> 32) as u32,
            dense_row: (location.0.get() as u32 - 1) as usize,
        });
    }

    #[inline(always)]
    fn get(&self, index: u32) -> Option<LegacyLocation> {
        self.rows.get(index as usize).copied().flatten()
    }

    fn allocated_bytes(&self) -> usize {
        self.rows.capacity() * size_of::<Option<LegacyLocation>>()
    }
}

struct Node<T> {
    children: Box<[Option<Box<T>>; FANOUT]>,
    occupied: u16,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self {
            children: Box::new(std::array::from_fn(|_| None)),
            occupied: 0,
        }
    }
}

impl<T: Default> Node<T> {
    fn child_or_insert(&mut self, index: usize) -> &mut T {
        if self.children[index].is_none() {
            self.children[index] = Some(Box::new(T::default()));
            self.occupied += 1;
        }
        self.children[index].as_deref_mut().expect("inserted child")
    }

    fn child(&self, index: usize) -> Option<&T> {
        self.children[index].as_deref()
    }
}

type Root = Node<Branch>;
type Branch = Node<Leaf>;
type Leaf = Node<Page>;

struct Page {
    rows: [Option<Location>; FANOUT],
    occupied: u16,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            rows: [None; FANOUT],
            occupied: 0,
        }
    }
}

impl Page {
    fn insert(&mut self, slot: usize, location: Location) -> bool {
        let inserted = self.rows[slot].replace(location).is_none();
        if inserted {
            self.occupied += 1;
        }
        inserted
    }

    fn allocated_bytes(&self) -> usize {
        size_of::<Self>()
    }
}

#[derive(Default)]
struct RadixLocator {
    root: Option<Box<Root>>,
    branches: usize,
    leaves: usize,
    pages: usize,
}

impl RadixLocator {
    fn insert(&mut self, index: u32, location: Location) {
        let [root_index, branch_index, leaf_index, slot_index] = split(index);
        let root = self.root.get_or_insert_with(|| Box::new(Root::default()));
        let had_branch = root.children[root_index].is_some();
        let branch = root.child_or_insert(root_index);
        if !had_branch {
            self.branches += 1;
        }
        let had_leaf = branch.children[branch_index].is_some();
        let leaf = branch.child_or_insert(branch_index);
        if !had_leaf {
            self.leaves += 1;
        }
        let had_page = leaf.children[leaf_index].is_some();
        let page = leaf.child_or_insert(leaf_index);
        if !had_page {
            self.pages += 1;
        }
        page.insert(slot_index, location);
    }

    #[inline(always)]
    fn get(&self, index: u32) -> Option<Location> {
        let [root_index, branch_index, leaf_index, slot_index] = split(index);
        self.root
            .as_deref()?
            .child(root_index)?
            .child(branch_index)?
            .child(leaf_index)?
            .rows[slot_index]
    }

    fn allocated_bytes(&self) -> usize {
        self.root
            .as_deref()
            .map_or(0, RadixAllocatedBytes::allocated_bytes)
    }
}

trait RadixAllocatedBytes {
    fn allocated_bytes(&self) -> usize;
}

impl RadixAllocatedBytes for Page {
    fn allocated_bytes(&self) -> usize {
        self.allocated_bytes()
    }
}

impl<T: RadixAllocatedBytes> RadixAllocatedBytes for Node<T> {
    fn allocated_bytes(&self) -> usize {
        size_of::<Self>()
            + size_of::<[Option<Box<T>>; FANOUT]>()
            + self
                .children
                .iter()
                .flatten()
                .map(|child| child.allocated_bytes())
                .sum::<usize>()
    }
}

#[derive(Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut value = 0_u64;
        for (shift, byte) in bytes.iter().copied().take(8).enumerate() {
            value |= u64::from(byte) << (shift * 8);
        }
        self.0 = value;
    }

    fn write_u32(&mut self, value: u32) {
        self.0 = u64::from(value);
    }
}

type IdentityBuildHasher = BuildHasherDefault<IdentityHasher>;

#[derive(Default)]
struct HashPageLocator {
    pages: HashMap<u32, Box<Page>, IdentityBuildHasher>,
}

#[derive(Default)]
struct HybridLocator {
    flat_prefix: Vec<Option<Location>>,
    flat_location_count: usize,
    flat_window_base: u32,
    flat_window: Vec<Option<Location>>,
    flat_window_location_count: usize,
    sparse_pages: HashPageLocator,
    sparse_page_keys: BTreeSet<u32>,
    location_count: usize,
}

impl HybridLocator {
    fn insert(&mut self, index: u32, location: Location) {
        if (index as usize) < self.flat_prefix.len() {
            if self.flat_prefix[index as usize].replace(location).is_none() {
                self.flat_location_count += 1;
                self.location_count += 1;
                self.promote_qualified_prefix();
            }
        } else if let Some(slot) = self.flat_window_slot(index) {
            if self.flat_window[slot].replace(location).is_none() {
                self.flat_window_location_count += 1;
                self.location_count += 1;
                self.promote_qualified_prefix();
            }
        } else {
            let (inserted, new_page) = self.sparse_pages.insert(index, location);
            if new_page {
                self.sparse_page_keys.insert(index >> RADIX_BITS);
            }
            if inserted {
                self.location_count += 1;
                self.promote_qualified_prefix();
                self.promote_qualified_window(index);
            }
        }
    }

    fn promote_qualified_prefix(&mut self) {
        let promotable_page = self
            .location_count
            .saturating_mul(PREFIX_PROMOTION_FACTOR)
            .checked_div(FANOUT)
            .and_then(|page_count| page_count.checked_sub(1))
            .and_then(|page_index| u32::try_from(page_index).ok())
            .unwrap_or(u32::MAX);
        let window_pages = self.flat_window_page_range();
        let absorb_window = window_pages
            .as_ref()
            .is_some_and(|range| *range.end() <= promotable_page);
        let sparse_limit = match window_pages.as_ref() {
            Some(range) if *range.start() <= promotable_page && !absorb_window => {
                range.start().saturating_sub(1)
            }
            _ => promotable_page,
        };
        let pages = self
            .sparse_page_keys
            .range(..=sparse_limit)
            .copied()
            .collect::<Vec<_>>();
        let highest_page = pages
            .last()
            .copied()
            .into_iter()
            .chain(absorb_window.then(|| {
                *window_pages
                    .as_ref()
                    .expect("absorbed window has pages")
                    .end()
            }))
            .max();
        let Some(highest_page) = highest_page else {
            return;
        };

        self.flat_prefix
            .resize((highest_page as usize + 1) * FANOUT, None);
        for page_index in pages {
            self.sparse_page_keys.remove(&page_index);
            let page = self
                .sparse_pages
                .pages
                .remove(&page_index)
                .expect("qualified sparse page remains owned");
            self.flat_location_count += usize::from(page.occupied);
            let start = page_index as usize * FANOUT;
            self.flat_prefix[start..start + FANOUT].copy_from_slice(&page.rows);
        }
        if absorb_window {
            let start = self.flat_window_base as usize;
            self.flat_prefix[start..start + self.flat_window.len()]
                .copy_from_slice(&self.flat_window);
            self.flat_location_count += self.flat_window_location_count;
            self.flat_window_base = 0;
            self.flat_window = Vec::new();
            self.flat_window_location_count = 0;
        }
        if self.sparse_pages.pages.is_empty() {
            self.sparse_pages = HashPageLocator::default();
            self.sparse_page_keys = BTreeSet::new();
        }
    }

    fn promote_qualified_window(&mut self, index: u32) {
        let inserted_page = index >> RADIX_BITS;
        if !self.sparse_page_keys.contains(&inserted_page) {
            return;
        }
        let candidate_base_page = self
            .flat_window_page_range()
            .map_or(inserted_page, |range| (*range.start()).min(inserted_page));
        let candidate_end_page = self
            .flat_window_page_range()
            .map_or(inserted_page, |range| (*range.end()).max(inserted_page));
        let candidate_slots =
            (candidate_end_page as usize - candidate_base_page as usize + 1) * FANOUT;
        if candidate_slots > self.location_count.saturating_mul(PREFIX_PROMOTION_FACTOR) {
            return;
        }

        let candidate_base = candidate_base_page << RADIX_BITS;
        if self.flat_window.is_empty() {
            self.flat_window_base = candidate_base;
            self.flat_window.resize(candidate_slots, None);
        } else if candidate_base < self.flat_window_base {
            let leading_slots = (self.flat_window_base - candidate_base) as usize;
            let mut expanded = vec![None; candidate_slots];
            expanded[leading_slots..leading_slots + self.flat_window.len()]
                .copy_from_slice(&self.flat_window);
            self.flat_window = expanded;
            self.flat_window_base = candidate_base;
        } else {
            self.flat_window.resize(candidate_slots, None);
        }

        let pages = self
            .sparse_page_keys
            .range(candidate_base_page..=candidate_end_page)
            .copied()
            .collect::<Vec<_>>();
        for page_index in pages {
            self.sparse_page_keys.remove(&page_index);
            let page = self
                .sparse_pages
                .pages
                .remove(&page_index)
                .expect("indexed sparse page remains owned");
            self.flat_window_location_count += usize::from(page.occupied);
            let page_base = page_index << RADIX_BITS;
            let start = (page_base - self.flat_window_base) as usize;
            self.flat_window[start..start + FANOUT].copy_from_slice(&page.rows);
        }
        if self.sparse_pages.pages.is_empty() {
            self.sparse_pages = HashPageLocator::default();
            self.sparse_page_keys = BTreeSet::new();
        }
    }

    #[inline(always)]
    fn get(&self, index: u32) -> Option<Location> {
        if index >= self.flat_window_base {
            let window_slot = (index - self.flat_window_base) as usize;
            if window_slot < self.flat_window.len() {
                return self.flat_window[window_slot];
            }
        }
        if (index as usize) < self.flat_prefix.len() {
            self.flat_prefix[index as usize]
        } else {
            self.sparse_pages.get(index)
        }
    }

    #[inline(always)]
    fn flat_window_slot(&self, index: u32) -> Option<usize> {
        if self.flat_window.is_empty() {
            return None;
        }
        let slot = index.checked_sub(self.flat_window_base)? as usize;
        (slot < self.flat_window.len()).then_some(slot)
    }

    fn flat_window_page_range(&self) -> Option<std::ops::RangeInclusive<u32>> {
        if self.flat_window.is_empty() {
            return None;
        }
        let start = self.flat_window_base >> RADIX_BITS;
        let page_count = self.flat_window.len() / FANOUT;
        Some(start..=start + page_count as u32 - 1)
    }

    fn allocated_bytes(&self) -> usize {
        self.flat_prefix.capacity() * size_of::<Option<Location>>()
            + self.flat_window.capacity() * size_of::<Option<Location>>()
            + self.sparse_pages.allocated_bytes()
            + self.sparse_page_keys.len() * (size_of::<u32>() + size_of::<usize>() * 4)
    }
}

impl HashPageLocator {
    fn insert(&mut self, index: u32, location: Location) -> (bool, bool) {
        let page_index = index >> RADIX_BITS;
        let slot_index = (index as usize) & (FANOUT - 1);
        let (page, new_page) = match self.pages.entry(page_index) {
            std::collections::hash_map::Entry::Occupied(entry) => (entry.into_mut(), false),
            std::collections::hash_map::Entry::Vacant(entry) => {
                (entry.insert(Box::new(Page::default())), true)
            }
        };
        (page.insert(slot_index, location), new_page)
    }

    #[inline(always)]
    fn get(&self, index: u32) -> Option<Location> {
        let page_index = index >> RADIX_BITS;
        let slot_index = (index as usize) & (FANOUT - 1);
        self.pages.get(&page_index)?.rows[slot_index]
    }

    fn allocated_bytes(&self) -> usize {
        self.pages.capacity() * (size_of::<u32>() + size_of::<Box<Page>>() + size_of::<u8>())
            + self
                .pages
                .values()
                .map(|page| page.allocated_bytes())
                .sum::<usize>()
    }
}

struct OpenPageBucket {
    page_index: u32,
    page: Option<Box<Page>>,
}

impl OpenPageBucket {
    fn empty() -> Self {
        Self {
            page_index: EMPTY_PAGE_KEY,
            page: None,
        }
    }
}

#[derive(Default)]
struct OpenPageLocator {
    buckets: Vec<OpenPageBucket>,
    page_count: usize,
}

impl OpenPageLocator {
    fn insert(&mut self, index: u32, location: Location) {
        let page_index = index >> RADIX_BITS;
        let slot_index = (index as usize) & (FANOUT - 1);
        if self.buckets.is_empty() || (self.page_count + 1) * 2 > self.buckets.len() {
            self.grow();
        }
        let bucket_index = self.find_bucket(page_index);
        let bucket = &mut self.buckets[bucket_index];
        if bucket.page_index == EMPTY_PAGE_KEY {
            bucket.page_index = page_index;
            bucket.page = Some(Box::new(Page::default()));
            self.page_count += 1;
        }
        bucket
            .page
            .as_deref_mut()
            .expect("occupied page bucket")
            .insert(slot_index, location);
    }

    #[inline(always)]
    fn get(&self, index: u32) -> Option<Location> {
        if self.buckets.is_empty() {
            return None;
        }
        let page_index = index >> RADIX_BITS;
        let slot_index = (index as usize) & (FANOUT - 1);
        let bucket = &self.buckets[self.find_bucket(page_index)];
        (bucket.page_index == page_index)
            .then(|| bucket.page.as_deref().expect("occupied page bucket"))?
            .rows[slot_index]
    }

    fn grow(&mut self) {
        let new_capacity = (self.buckets.len() * 2).max(MIN_PAGE_BUCKETS);
        let old_buckets = std::mem::replace(
            &mut self.buckets,
            (0..new_capacity).map(|_| OpenPageBucket::empty()).collect(),
        );
        for bucket in old_buckets {
            if bucket.page_index == EMPTY_PAGE_KEY {
                continue;
            }
            let destination = self.find_bucket(bucket.page_index);
            self.buckets[destination] = bucket;
        }
    }

    #[inline(always)]
    fn find_bucket(&self, page_index: u32) -> usize {
        let mask = self.buckets.len() - 1;
        let mut bucket = page_index.wrapping_mul(PAGE_HASH_MULTIPLIER) as usize & mask;
        loop {
            let candidate = self.buckets[bucket].page_index;
            if candidate == EMPTY_PAGE_KEY || candidate == page_index {
                return bucket;
            }
            bucket = (bucket + 1) & mask;
        }
    }

    fn allocated_bytes(&self) -> usize {
        self.buckets.capacity() * size_of::<OpenPageBucket>() + self.page_count * size_of::<Page>()
    }
}

fn split(index: u32) -> [usize; 4] {
    index.to_be_bytes().map(usize::from)
}

fn lookup_sample(mut lookup: impl FnMut(u32) -> Option<u64>, indices: &[u32]) -> (Duration, u64) {
    let mut checksum = 0_u64;
    let started = Instant::now();
    for _ in 0..LOOKUP_REPEATS {
        for &index in indices {
            checksum =
                checksum.wrapping_add(black_box(lookup(black_box(index))).unwrap_or(u64::MAX));
        }
    }
    (started.elapsed(), checksum)
}

fn percentile_pair(mut samples: Vec<Duration>) -> (Duration, Duration) {
    samples.sort_unstable();
    let p50 = samples[samples.len() / 2];
    let p95 = samples[(samples.len() * 95 / 100).min(samples.len() - 1)];
    (p50, p95)
}

fn paired_lookup_samples(
    legacy: &LegacyLocator,
    hybrid: &HybridLocator,
    indices: &[u32],
) -> (Duration, Duration, Duration, Duration, u64) {
    let mut legacy_samples = Vec::with_capacity(PAIRED_LOOKUP_SAMPLES);
    let mut hybrid_samples = Vec::with_capacity(PAIRED_LOOKUP_SAMPLES);
    let mut checksum = 0_u64;
    for sample in 0..PAIRED_LOOKUP_SAMPLES {
        if sample % 2 == 0 {
            let (legacy_elapsed, legacy_checksum) = lookup_sample(
                |index| legacy.get(index).map(LegacyLocation::observation),
                indices,
            );
            let (hybrid_elapsed, hybrid_checksum) = lookup_sample(
                |index| hybrid.get(index).map(Location::observation),
                indices,
            );
            legacy_samples.push(legacy_elapsed);
            hybrid_samples.push(hybrid_elapsed);
            checksum = checksum.wrapping_add(legacy_checksum);
            assert_eq!(legacy_checksum, hybrid_checksum);
        } else {
            let (hybrid_elapsed, hybrid_checksum) = lookup_sample(
                |index| hybrid.get(index).map(Location::observation),
                indices,
            );
            let (legacy_elapsed, legacy_checksum) = lookup_sample(
                |index| legacy.get(index).map(LegacyLocation::observation),
                indices,
            );
            legacy_samples.push(legacy_elapsed);
            hybrid_samples.push(hybrid_elapsed);
            checksum = checksum.wrapping_add(legacy_checksum);
            assert_eq!(legacy_checksum, hybrid_checksum);
        }
    }
    let (legacy_p50, legacy_p95) = percentile_pair(legacy_samples);
    let (hybrid_p50, hybrid_p95) = percentile_pair(hybrid_samples);
    (legacy_p50, legacy_p95, hybrid_p50, hybrid_p95, checksum)
}

fn paired_open_lookup_samples(
    legacy: &LegacyLocator,
    open: &OpenPageLocator,
    indices: &[u32],
) -> (Duration, Duration, Duration, Duration, u64) {
    let mut legacy_samples = Vec::with_capacity(PAIRED_LOOKUP_SAMPLES);
    let mut open_samples = Vec::with_capacity(PAIRED_LOOKUP_SAMPLES);
    let mut checksum = 0_u64;
    for sample in 0..PAIRED_LOOKUP_SAMPLES {
        if sample % 2 == 0 {
            let (legacy_elapsed, legacy_checksum) = lookup_sample(
                |index| legacy.get(index).map(LegacyLocation::observation),
                indices,
            );
            let (open_elapsed, open_checksum) =
                lookup_sample(|index| open.get(index).map(Location::observation), indices);
            legacy_samples.push(legacy_elapsed);
            open_samples.push(open_elapsed);
            checksum = checksum.wrapping_add(legacy_checksum);
            assert_eq!(legacy_checksum, open_checksum);
        } else {
            let (open_elapsed, open_checksum) =
                lookup_sample(|index| open.get(index).map(Location::observation), indices);
            let (legacy_elapsed, legacy_checksum) = lookup_sample(
                |index| legacy.get(index).map(LegacyLocation::observation),
                indices,
            );
            legacy_samples.push(legacy_elapsed);
            open_samples.push(open_elapsed);
            checksum = checksum.wrapping_add(legacy_checksum);
            assert_eq!(legacy_checksum, open_checksum);
        }
    }
    let (legacy_p50, legacy_p95) = percentile_pair(legacy_samples);
    let (open_p50, open_p95) = percentile_pair(open_samples);
    (legacy_p50, legacy_p95, open_p50, open_p95, checksum)
}

fn reduction_percent(before: usize, after: usize) -> f64 {
    (before.saturating_sub(after)) as f64 * 100.0 / before as f64
}

fn run_partial_page_scenario(
    occupancy_label: &str,
    include: fn(u32) -> bool,
    query_indices: &[u32],
) {
    let mut legacy = LegacyLocator::default();
    let mut open = OpenPageLocator::default();
    let mut hybrid = HybridLocator::default();
    let mut live_rows = 0_u32;
    for &index in query_indices {
        if !include(index) {
            continue;
        }
        let location = Location::new(5, live_rows);
        legacy.insert(index, location);
        open.insert(index, location);
        hybrid.insert(index, location);
        live_rows += 1;
    }
    let (legacy_p50, legacy_p95, open_p50, open_p95, checksum) =
        paired_open_lookup_samples(&legacy, &open, query_indices);
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=paired_partial_open_pages occupancy={} query_count={} live_rows={} legacy_bytes={} open_bytes={} pairs={} repeats={} legacy_lookup_p50_ns={} legacy_lookup_p95_ns={} open_lookup_p50_ns={} open_lookup_p95_ns={} p50_regression_percent={:.4} p95_regression_percent={:.4} checksum={}",
        occupancy_label,
        query_indices.len(),
        live_rows,
        legacy.allocated_bytes(),
        open.allocated_bytes(),
        PAIRED_LOOKUP_SAMPLES,
        LOOKUP_REPEATS,
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        open_p50.as_nanos(),
        open_p95.as_nanos(),
        (open_p50.as_secs_f64() / legacy_p50.as_secs_f64() - 1.0) * 100.0,
        (open_p95.as_secs_f64() / legacy_p95.as_secs_f64() - 1.0) * 100.0,
        checksum,
    );
    let (legacy_p50, legacy_p95, hybrid_p50, hybrid_p95, checksum) =
        paired_lookup_samples(&legacy, &hybrid, query_indices);
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=paired_partial_hybrid occupancy={} query_count={} live_rows={} legacy_bytes={} hybrid_bytes={} pairs={} repeats={} legacy_lookup_p50_ns={} legacy_lookup_p95_ns={} hybrid_lookup_p50_ns={} hybrid_lookup_p95_ns={} p50_regression_percent={:.4} p95_regression_percent={:.4} checksum={}",
        occupancy_label,
        query_indices.len(),
        live_rows,
        legacy.allocated_bytes(),
        hybrid.allocated_bytes(),
        PAIRED_LOOKUP_SAMPLES,
        LOOKUP_REPEATS,
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        hybrid_p50.as_nanos(),
        hybrid_p95.as_nanos(),
        (hybrid_p50.as_secs_f64() / legacy_p50.as_secs_f64() - 1.0) * 100.0,
        (hybrid_p95.as_secs_f64() / legacy_p95.as_secs_f64() - 1.0) * 100.0,
        checksum,
    );
}

fn run_sparse_high_scenarios() {
    let query_indices =
        (HIGH_ENTITY_INDEX..HIGH_ENTITY_INDEX + SPARSE_HIGH_RANGE).collect::<Vec<_>>();
    let live_indices = query_indices
        .iter()
        .copied()
        .filter(|index| (index - HIGH_ENTITY_INDEX) % SPARSE_HIGH_STRIDE == 0)
        .collect::<Vec<_>>();
    let mut repeated_hit_indices = Vec::with_capacity(
        live_indices
            .len()
            .saturating_mul(SPARSE_HIGH_HIT_REPETITIONS),
    );
    for _ in 0..SPARSE_HIGH_HIT_REPETITIONS {
        repeated_hit_indices.extend_from_slice(&live_indices);
    }

    let mut legacy = LegacyLocator::default();
    let mut hybrid = HybridLocator::default();
    for (dense_row, &index) in live_indices.iter().enumerate() {
        let location = Location::new(13, dense_row as u32);
        legacy.insert(index, location);
        hybrid.insert(index, location);
    }
    assert!(hybrid.flat_prefix.is_empty());
    assert!(!hybrid.flat_window.is_empty());
    assert!(hybrid.sparse_pages.pages.is_empty());

    for (scenario, indices) in [
        ("sparse_high_mixed", query_indices.as_slice()),
        ("sparse_high_hits", repeated_hit_indices.as_slice()),
    ] {
        let (legacy_p50, legacy_p95, hybrid_p50, hybrid_p95, checksum) =
            paired_lookup_samples(&legacy, &hybrid, indices);
        println!(
            "SPARSE_LOCATOR_BENCH_V1 scenario={} query_count={} live_rows={} legacy_bytes={} hybrid_bytes={} flat_prefix_slots={} flat_window_slots={} sparse_pages={} pairs={} repeats={} legacy_lookup_p50_ns={} legacy_lookup_p95_ns={} hybrid_lookup_p50_ns={} hybrid_lookup_p95_ns={} p50_regression_percent={:.4} p95_regression_percent={:.4} checksum={}",
            scenario,
            indices.len(),
            live_indices.len(),
            legacy.allocated_bytes(),
            hybrid.allocated_bytes(),
            hybrid.flat_prefix.len(),
            hybrid.flat_window.len(),
            hybrid.sparse_pages.pages.len(),
            PAIRED_LOOKUP_SAMPLES,
            LOOKUP_REPEATS,
            legacy_p50.as_nanos(),
            legacy_p95.as_nanos(),
            hybrid_p50.as_nanos(),
            hybrid_p95.as_nanos(),
            (hybrid_p50.as_secs_f64() / legacy_p50.as_secs_f64() - 1.0) * 100.0,
            (hybrid_p95.as_secs_f64() / legacy_p95.as_secs_f64() - 1.0) * 100.0,
            checksum,
        );
    }
}

fn run_dual_span_scenario() {
    let mut legacy = LegacyLocator::default();
    let mut hybrid = HybridLocator::default();
    let mut span_indices = Vec::with_capacity(DUAL_SPAN_ROWS as usize * 2);
    for offset in 0..DUAL_SPAN_ROWS {
        let low = offset;
        let high = HIGH_ENTITY_INDEX + offset;
        legacy.insert(low, Location::new(17, low));
        hybrid.insert(low, Location::new(17, low));
        legacy.insert(high, Location::new(17, DUAL_SPAN_ROWS + offset));
        hybrid.insert(high, Location::new(17, DUAL_SPAN_ROWS + offset));
        span_indices.extend([low, high]);
    }
    let mut query_indices = Vec::with_capacity(
        span_indices
            .len()
            .saturating_mul(DUAL_SPAN_QUERY_REPETITIONS),
    );
    for _ in 0..DUAL_SPAN_QUERY_REPETITIONS {
        query_indices.extend_from_slice(&span_indices);
    }
    assert!(!hybrid.flat_prefix.is_empty());
    assert!(!hybrid.flat_window.is_empty());

    let (legacy_p50, legacy_p95, hybrid_p50, hybrid_p95, checksum) =
        paired_lookup_samples(&legacy, &hybrid, &query_indices);
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=dual_span_hits query_count={} legacy_bytes={} hybrid_bytes={} flat_prefix_slots={} flat_window_slots={} pairs={} repeats={} legacy_lookup_p50_ns={} legacy_lookup_p95_ns={} hybrid_lookup_p50_ns={} hybrid_lookup_p95_ns={} p50_regression_percent={:.4} p95_regression_percent={:.4} checksum={}",
        query_indices.len(),
        legacy.allocated_bytes(),
        hybrid.allocated_bytes(),
        hybrid.flat_prefix.len(),
        hybrid.flat_window.len(),
        PAIRED_LOOKUP_SAMPLES,
        LOOKUP_REPEATS,
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        hybrid_p50.as_nanos(),
        hybrid_p95.as_nanos(),
        (hybrid_p50.as_secs_f64() / legacy_p50.as_secs_f64() - 1.0) * 100.0,
        (hybrid_p95.as_secs_f64() / legacy_p95.as_secs_f64() - 1.0) * 100.0,
        checksum,
    );
}

fn main() {
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=layout pointer_bytes={} legacy_slot_bytes={} packed_slot_bytes={} page_slots={} prefix_promotion_factor={}",
        size_of::<usize>(),
        size_of::<Option<LegacyLocation>>(),
        size_of::<Option<Location>>(),
        FANOUT,
        PREFIX_PROMOTION_FACTOR,
    );
    let mut high_legacy = LegacyLocator::default();
    let mut high_radix = RadixLocator::default();
    let mut high_hash = HashPageLocator::default();
    let mut high_hybrid = HybridLocator::default();
    high_legacy.insert(HIGH_ENTITY_INDEX, Location::new(7, 0));
    high_radix.insert(HIGH_ENTITY_INDEX, Location::new(7, 0));
    high_hash.insert(HIGH_ENTITY_INDEX, Location::new(7, 0));
    high_hybrid.insert(HIGH_ENTITY_INDEX, Location::new(7, 0));
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=high_water index={} legacy_bytes={} radix_bytes={} hash_bytes={} hybrid_bytes={} radix_byte_reduction_percent={:.4} hash_byte_reduction_percent={:.4} hybrid_byte_reduction_percent={:.4}",
        HIGH_ENTITY_INDEX,
        high_legacy.allocated_bytes(),
        high_radix.allocated_bytes(),
        high_hash.allocated_bytes(),
        high_hybrid.allocated_bytes(),
        reduction_percent(high_legacy.allocated_bytes(), high_radix.allocated_bytes()),
        reduction_percent(high_legacy.allocated_bytes(), high_hash.allocated_bytes()),
        reduction_percent(
            high_legacy.allocated_bytes(),
            high_hybrid.allocated_bytes()
        ),
    );

    let mut legacy = LegacyLocator::default();
    let mut radix = RadixLocator::default();
    let mut hash = HashPageLocator::default();
    let mut hybrid = HybridLocator::default();
    let dense_indices = (0..DENSE_ENTITY_COUNT).collect::<Vec<_>>();
    for &index in &dense_indices {
        let location = Location::new(3, index);
        legacy.insert(index, location);
        radix.insert(index, location);
        hash.insert(index, location);
        hybrid.insert(index, location);
    }
    let last_dense_index = DENSE_ENTITY_COUNT - 1;
    let expected = legacy
        .get(last_dense_index)
        .map(LegacyLocation::observation);
    assert_eq!(
        radix.get(last_dense_index).map(Location::observation),
        expected
    );
    assert_eq!(
        hash.get(last_dense_index).map(Location::observation),
        expected
    );
    assert_eq!(
        hybrid.get(last_dense_index).map(Location::observation),
        expected
    );
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=dense_memory count={} legacy_bytes={} radix_bytes={} hash_bytes={} hybrid_bytes={}",
        DENSE_ENTITY_COUNT,
        legacy.allocated_bytes(),
        radix.allocated_bytes(),
        hash.allocated_bytes(),
        hybrid.allocated_bytes(),
    );
    let (
        paired_legacy_p50,
        paired_legacy_p95,
        paired_hybrid_p50,
        paired_hybrid_p95,
        paired_checksum,
    ) = paired_lookup_samples(&legacy, &hybrid, &dense_indices);
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=paired_dense count={} pairs={} repeats={} legacy_lookup_p50_ns={} legacy_lookup_p95_ns={} hybrid_lookup_p50_ns={} hybrid_lookup_p95_ns={} p50_regression_percent={:.4} p95_regression_percent={:.4} checksum={}",
        DENSE_ENTITY_COUNT,
        PAIRED_LOOKUP_SAMPLES,
        LOOKUP_REPEATS,
        paired_legacy_p50.as_nanos(),
        paired_legacy_p95.as_nanos(),
        paired_hybrid_p50.as_nanos(),
        paired_hybrid_p95.as_nanos(),
        (paired_hybrid_p50.as_secs_f64() / paired_legacy_p50.as_secs_f64() - 1.0) * 100.0,
        (paired_hybrid_p95.as_secs_f64() / paired_legacy_p95.as_secs_f64() - 1.0) * 100.0,
        paired_checksum,
    );

    run_partial_page_scenario("1_percent", |index| index % 100 == 0, &dense_indices);
    run_partial_page_scenario("5_percent", |index| index % 20 == 0, &dense_indices);
    run_partial_page_scenario("10_percent", |index| index % 10 == 0, &dense_indices);
    run_partial_page_scenario("25_percent", |index| index % 4 == 0, &dense_indices);
    run_partial_page_scenario("75_percent", |index| index % 4 != 0, &dense_indices);
    run_partial_page_scenario(
        "0.1_percent_gapped",
        |index| index % 1_000 == 0,
        &dense_indices,
    );
    run_sparse_high_scenarios();
    run_dual_span_scenario();

    let mut extreme = HybridLocator::default();
    extreme.insert(EXTREME_ENTITY_INDEX, Location::new(11, 0));
    let projected_legacy_bytes =
        (u64::from(EXTREME_ENTITY_INDEX) + 1) * size_of::<Option<LegacyLocation>>() as u64;
    println!(
        "SPARSE_LOCATOR_BENCH_V1 scenario=extreme index={} projected_legacy_bytes={} hybrid_bytes={} hit={}",
        EXTREME_ENTITY_INDEX,
        projected_legacy_bytes,
        extreme.allocated_bytes(),
        extreme.get(EXTREME_ENTITY_INDEX).is_some(),
    );
}
