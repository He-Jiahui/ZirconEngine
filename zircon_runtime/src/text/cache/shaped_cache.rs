//! 成形产物的拥有式缓存、借用查询键和按帧操作报告。
//! 指纹只缩小候选范围；命中仍核对身份载荷与共享产物中的原文。

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    mem::size_of,
    sync::Arc,
};

use super::index::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

use crate::core::framework::text::{TextDirection, TextFontCollectionHandle};
use crate::text::font::{shared_font_collection_handle, shared_font_database_generation};
use crate::text::{
    BackendShapeRequest, EphemeralCacheHash, EphemeralCacheHasher, OpenTypeFeature, ShapedGlyphRun,
    TextOrientation, UnicodeDataSnapshotId, VerticalMode,
};
use crate::text::{TextRange, TextStyle};

#[path = "shaped_cache/memory.rs"]
mod memory;

use memory::estimated_entry_bytes;

pub(crate) const DEFAULT_SHAPED_RUN_CACHE_CAPACITY: usize = 1024;
pub(crate) const DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;

/// 成形缓存的拥有式身份键；保存文本指纹与源区间，不保存全文。
///
/// 字符串与特征载荷在从查询键转换时复制；原文另由条目的共享产物保留。
/// # 约束
/// 请求须已规范化；构造仅以调试断言检查前提，不执行规范化。
/// 字体集合、字体代次与 Unicode 快照参与身份，不负责冻结字体数据库。
/// 参考 [`ShapedRunCacheLookupKey`] 与 [`ShapedRunCache::get_with_lookup`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShapedRunCacheKey {
    pub(crate) text_hash: EphemeralCacheHash,
    pub(crate) source_range: TextRange,
    pub(crate) font_asset: Option<String>,
    pub(crate) font_family: Option<String>,
    pub(crate) font_weight: u16,
    pub(crate) font_italic: bool,
    pub(crate) font_size_bits: u32,
    pub(crate) line_height_bits: u32,
    pub(crate) tab_size_bits: u32,
    pub(crate) base_direction: TextDirection,
    pub(crate) orientation: TextOrientation,
    pub(crate) vertical_mode: VerticalMode,
    include_kerning: bool,
    pub(crate) features_hash: EphemeralCacheHash,
    features: Arc<[OpenTypeFeature]>,
    pub(crate) language: Option<String>,
    pub(crate) font_collection: TextFontCollectionHandle,
    pub(crate) font_database_generation: u64,
    pub(crate) unicode_data_snapshot: UnicodeDataSnapshotId,
}

/// 仅在请求借用期间使用的查询元数据视图，避免查询前构造拥有式键。
///
/// 借用字符串及特征切片；其生命周期不随命中返回的共享产物延长。
/// 文本仍由调用者另传给查询方法核对，不由此视图持有。
/// 参考 [`ShapedRunCacheKey`]。
#[derive(Clone, Copy, Debug)]
pub(crate) struct ShapedRunCacheLookupKey<'a> {
    text_hash: EphemeralCacheHash,
    source_range: TextRange,
    font_asset: Option<&'a str>,
    font_family: Option<&'a str>,
    font_weight: u16,
    font_italic: bool,
    font_size_bits: u32,
    line_height_bits: u32,
    tab_size_bits: u32,
    base_direction: TextDirection,
    orientation: TextOrientation,
    vertical_mode: VerticalMode,
    include_kerning: bool,
    features_hash: EphemeralCacheHash,
    features: &'a [OpenTypeFeature],
    language: Option<&'a str>,
    font_collection: TextFontCollectionHandle,
    font_database_generation: u64,
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

impl ShapedRunCacheKey {
    /// 从已规范化请求复制身份载荷；共享字体集合及当前代次取自进程入口。
    pub(crate) fn from_request(request: &BackendShapeRequest<'_>) -> Self {
        Self::from_lookup(&ShapedRunCacheLookupKey::from_request(request))
    }

    fn from_lookup(lookup: &ShapedRunCacheLookupKey<'_>) -> Self {
        Self {
            text_hash: lookup.text_hash,
            source_range: lookup.source_range,
            font_asset: lookup.font_asset.map(ToOwned::to_owned),
            font_family: lookup.font_family.map(ToOwned::to_owned),
            font_weight: lookup.font_weight,
            font_italic: lookup.font_italic,
            font_size_bits: lookup.font_size_bits,
            line_height_bits: lookup.line_height_bits,
            tab_size_bits: lookup.tab_size_bits,
            base_direction: lookup.base_direction,
            orientation: lookup.orientation,
            vertical_mode: lookup.vertical_mode,
            include_kerning: lookup.include_kerning,
            features_hash: lookup.features_hash,
            features: Arc::from(lookup.features),
            language: lookup.language.map(ToOwned::to_owned),
            font_collection: lookup.font_collection,
            font_database_generation: lookup.font_database_generation,
            unicode_data_snapshot: lookup.unicode_data_snapshot,
        }
    }

    fn lookup(&self) -> ShapedRunCacheLookupKey<'_> {
        ShapedRunCacheLookupKey {
            text_hash: self.text_hash,
            source_range: self.source_range,
            font_asset: self.font_asset.as_deref(),
            font_family: self.font_family.as_deref(),
            font_weight: self.font_weight,
            font_italic: self.font_italic,
            font_size_bits: self.font_size_bits,
            line_height_bits: self.line_height_bits,
            tab_size_bits: self.tab_size_bits,
            base_direction: self.base_direction,
            orientation: self.orientation,
            vertical_mode: self.vertical_mode,
            include_kerning: self.include_kerning,
            features_hash: self.features_hash,
            features: self.features.as_ref(),
            language: self.language.as_deref(),
            font_collection: self.font_collection,
            font_database_generation: self.font_database_generation,
            unicode_data_snapshot: self.unicode_data_snapshot,
        }
    }

    pub(crate) fn matches_lookup(&self, lookup: &ShapedRunCacheLookupKey<'_>) -> bool {
        self.text_hash == lookup.text_hash
            && self.source_range == lookup.source_range
            && self.font_asset.as_deref() == lookup.font_asset
            && self.font_family.as_deref() == lookup.font_family
            && self.font_weight == lookup.font_weight
            && self.font_italic == lookup.font_italic
            && self.font_size_bits == lookup.font_size_bits
            && self.line_height_bits == lookup.line_height_bits
            && self.tab_size_bits == lookup.tab_size_bits
            && self.base_direction == lookup.base_direction
            && self.orientation == lookup.orientation
            && self.vertical_mode == lookup.vertical_mode
            && self.include_kerning == lookup.include_kerning
            && self.features_hash == lookup.features_hash
            && self.features.as_ref() == lookup.features
            && self.language.as_deref() == lookup.language
            && self.font_collection == lookup.font_collection
            && self.font_database_generation == lookup.font_database_generation
            && self.unicode_data_snapshot == lookup.unicode_data_snapshot
    }

    pub(crate) const fn font_database_generation(&self) -> u64 {
        self.font_database_generation
    }
}

impl<'a> ShapedRunCacheLookupKey<'a> {
    pub(crate) fn from_request(request: &'a BackendShapeRequest<'a>) -> Self {
        Self::from_request_in_font_collection(
            request,
            shared_font_collection_handle(),
            shared_font_database_generation(),
        )
    }

    /// 借用已规范化请求，并记录所给字体集合及代次。
    ///
    /// # 参数与返回
    /// `request` 供本次查询使用；集合句柄、代次及 Unicode 快照只组成身份值。
    /// 返回不拥有原文字节的借用视图；调试断言不能替代调用方规范化。
    /// 参考 [`ShapedRunCache::get_with_lookup`]。
    pub(crate) fn from_request_in_font_collection(
        request: &'a BackendShapeRequest<'a>,
        font_collection: TextFontCollectionHandle,
        font_database_generation: u64,
    ) -> Self {
        debug_assert!(request.features_are_normalized());
        debug_assert!(request.language_is_canonical());
        let font_size = request.style.font_size.max(1.0);
        let line_height = request.style.line_height.max(font_size);

        Self {
            text_hash: hash_text(request.text),
            source_range: request.source_range,
            font_asset: cache_font_asset_ref(request.style.font.as_deref()),
            font_family: cache_text_identity_ref(request.style.font_family.as_deref()),
            font_weight: TextStyle::normalized_font_weight(request.style.font_weight),
            font_italic: request.style.italic,
            font_size_bits: normalized_f32_bits(font_size),
            line_height_bits: normalized_f32_bits(line_height),
            tab_size_bits: normalized_f32_bits(request.style.tab_size),
            base_direction: request.base_direction,
            orientation: request.orientation,
            vertical_mode: request.vertical_mode,
            include_kerning: request.include_kerning,
            features_hash: shaping_features_hash(request),
            features: request.features(),
            language: request.language,
            font_collection,
            font_database_generation,
            unicode_data_snapshot: request.unicode_data_snapshot(),
        }
    }

    pub(crate) fn exact_fingerprint(&self) -> EphemeralCacheHash {
        self.full_fingerprint()
    }

    pub(crate) const fn font_database_generation(&self) -> u64 {
        self.font_database_generation
    }

    // 指纹只选择候选桶；命中仍须比较身份字段、特征载荷及原文。
    fn full_fingerprint(&self) -> EphemeralCacheHash {
        let mut hasher = EphemeralCacheHasher::new();
        hasher.write(&self.text_hash);
        hasher.write(&self.source_range.start);
        hasher.write(&self.source_range.end);
        hasher.write(&self.font_asset.map(hash_text));
        hasher.write(&self.font_family.map(hash_text));
        hasher.write(&self.font_weight);
        hasher.write(&self.font_italic);
        hasher.write(&self.font_size_bits);
        hasher.write(&self.line_height_bits);
        hasher.write(&self.tab_size_bits);
        hasher.write(&std::mem::discriminant(&self.base_direction));
        hasher.write(&std::mem::discriminant(&self.orientation));
        hasher.write(&std::mem::discriminant(&self.vertical_mode));
        hasher.write(&self.include_kerning);
        hasher.write(&self.features_hash);
        hasher.write(&self.language);
        hasher.write(&self.font_collection);
        hasher.write(&self.font_database_generation);
        hasher.write(&self.unicode_data_snapshot);
        hasher.finish()
    }

    // 别名桶仅省略基础方向；方向转换是否合法由候选的命中条件单独判定。
    fn direction_alias_fingerprint(&self) -> EphemeralCacheHash {
        let mut hasher = EphemeralCacheHasher::new();
        hasher.write(&self.text_hash);
        hasher.write(&self.source_range.start);
        hasher.write(&self.source_range.end);
        hasher.write(&self.font_asset.map(hash_text));
        hasher.write(&self.font_family.map(hash_text));
        hasher.write(&self.font_weight);
        hasher.write(&self.font_italic);
        hasher.write(&self.font_size_bits);
        hasher.write(&self.line_height_bits);
        hasher.write(&self.tab_size_bits);
        hasher.write(&std::mem::discriminant(&self.orientation));
        hasher.write(&std::mem::discriminant(&self.vertical_mode));
        hasher.write(&self.include_kerning);
        hasher.write(&self.features_hash);
        hasher.write(&self.language);
        hasher.write(&self.font_collection);
        hasher.write(&self.font_database_generation);
        hasher.write(&self.unicode_data_snapshot);
        hasher.finish()
    }

    fn owned_key_byte_len(&self) -> usize {
        self.font_asset
            .map_or(0, str::len)
            .saturating_add(self.font_family.map_or(0, str::len))
            .saturating_add(self.language.map_or(0, str::len))
            .saturating_add(
                self.features
                    .len()
                    .saturating_mul(size_of::<OpenTypeFeature>()),
            )
    }
}

impl Hash for ShapedRunCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text_hash.hash(state);
        self.source_range.start.hash(state);
        self.source_range.end.hash(state);
        self.font_asset.hash(state);
        self.font_family.hash(state);
        self.font_weight.hash(state);
        self.font_italic.hash(state);
        self.font_size_bits.hash(state);
        self.line_height_bits.hash(state);
        self.tab_size_bits.hash(state);
        std::mem::discriminant(&self.base_direction).hash(state);
        std::mem::discriminant(&self.orientation).hash(state);
        std::mem::discriminant(&self.vertical_mode).hash(state);
        self.include_kerning.hash(state);
        self.features_hash.hash(state);
        self.features.hash(state);
        self.language.hash(state);
        self.font_collection.hash(state);
        self.font_database_generation.hash(state);
        self.unicode_data_snapshot.hash(state);
    }
}

/// 当前帧的缓存操作计数及当前条目数、估算体积快照。
///
/// 帧开始重置操作计数；载荷复制量仅统计专门的查询键拥有化入口，
/// 不表示所有分配、分配器实测或物理驻留量。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShapedRunCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) max_bytes: usize,
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) lookup_candidate_count: u64,
    pub(crate) owned_key_allocation_bytes: u64,
    pub(crate) eviction_scan_count: u64,
    pub(crate) entry_move_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) trim_count: u64,
    pub(crate) clear_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct ShapedRunCacheEntry {
    key: ShapedRunCacheKey,
    run: Arc<ShapedGlyphRun>,
    estimated_bytes: usize,
}

impl IndexedTextCacheEntry<ShapedRunCacheKey> for ShapedRunCacheEntry {
    fn cache_key(&self) -> &ShapedRunCacheKey {
        &self.key
    }
}

/// 单个拥有者维护的成形缓存，使用稳定槽与两类候选桶辅助查找和淘汰。
///
/// 容量与字节限制在插入或更新后整理，不是分配前准入门。
/// 返回的 `Arc` 可独立持有，淘汰和清空不会撤销外部引用。
/// 就绪状态与字体代次的一致性由调用方把关，缓存不自行冻结字体资源。
/// 参考 [`ShapedRunCacheReport`]。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShapedRunCache {
    index: IndexedTextCache<ShapedRunCacheKey, ShapedRunCacheEntry>,
    lookup_buckets: HashMap<EphemeralCacheHash, Vec<TextCacheSlot>>,
    lookup_bucket_positions: HashMap<TextCacheSlot, usize>,
    direction_alias_buckets: HashMap<EphemeralCacheHash, Vec<TextCacheSlot>>,
    direction_alias_bucket_positions: HashMap<TextCacheSlot, usize>,
    capacity: usize,
    max_bytes: usize,
    estimated_bytes: usize,
    frame_report: ShapedRunCacheReport,
}

impl Default for ShapedRunCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapedRunCache {
    pub(crate) fn new() -> Self {
        Self::with_limits(
            DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
            DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
        )
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::with_limits(capacity, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES)
    }

    /// 以条目数和启发式字节上限创建空缓存。
    ///
    /// # 参数与返回
    /// 两个上限均可为零；返回拥有独立索引和空报告的缓存。
    /// 插入后再淘汰，可能立刻移除新条目，但该次返回的共享产物仍有效。
    /// 参考 [`Self::insert_ready`]。
    pub(crate) fn with_limits(capacity: usize, max_bytes: usize) -> Self {
        let mut cache = Self {
            index: IndexedTextCache::new(),
            lookup_buckets: HashMap::new(),
            lookup_bucket_positions: HashMap::new(),
            direction_alias_buckets: HashMap::new(),
            direction_alias_bucket_positions: HashMap::new(),
            capacity,
            max_bytes,
            estimated_bytes: 0,
            frame_report: ShapedRunCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache.frame_report.max_bytes = max_bytes;
        cache
    }

    /// 开始指定帧并重置操作报告，保留已有缓存条目。
    ///
    /// `frame_index` 仅写入报告，不据此清空条目或改变外部引用。
    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.frame_report = ShapedRunCacheReport {
            frame_index,
            capacity: self.capacity,
            max_bytes: self.max_bytes,
            entry_count: self.index.len(),
            estimated_bytes: self.estimated_bytes,
            ..ShapedRunCacheReport::default()
        };
    }

    /// 执行帧尾限制整理，保留该帧已累计的计数。
    pub(crate) fn finish_frame(&mut self) {
        self.trim_to_limits();
    }

    /// 移除全部条目和辅助索引，更新体积并累计清空、逐出数。
    ///
    /// 不重置同帧的其余操作计数，也不撤销调用方已持有的 `Arc`。
    pub(crate) fn clear(&mut self) {
        self.frame_report.evicted_count = self
            .frame_report
            .evicted_count
            .saturating_add(self.index.len() as u64);
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.index.clear();
        self.lookup_buckets.clear();
        self.lookup_bucket_positions.clear();
        self.direction_alias_buckets.clear();
        self.direction_alias_bucket_positions.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub(crate) fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }

    /// 按值返回操作计数，并以当前条目数和估算体积刷新快照。
    pub(crate) fn report(&self) -> ShapedRunCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.index.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    /// 只检查精确键及原文，不走方向别名，也不触碰使用顺序或查询计数。
    ///
    /// `text` 须对应键的原文；返回布尔值，不交出产物引用。
    pub(crate) fn contains_exact(&self, key: &ShapedRunCacheKey, text: &str) -> bool {
        self.index
            .find_slot(key, |entry| entry.run.source_text.as_ref() == text)
            .slot
            .is_some()
    }

    /// 以拥有式键借出的查询视图查找；命中语义见 [`Self::get_with_lookup`]。
    pub(crate) fn get(
        &mut self,
        key: &ShapedRunCacheKey,
        text: &str,
    ) -> Option<Arc<ShapedGlyphRun>> {
        self.get_with_lookup(&key.lookup(), text)
    }

    /// 以借用键及原文查找共享产物；命中后推进条目的使用顺序。
    ///
    /// # 参数与返回
    /// `lookup` 决定候选桶及身份比较，`text` 另行精确核对全文；两者仅借用至返回。
    /// 返回独立共享引用或未命中；精确候选失败后仅尝试受限方向别名。
    /// 别名要求无规定的行、段分隔符，缓存为自动或混合方向，请求为左右定向且产物方向一致。
    /// 参考 [`ShapedRunCacheKey::matches_lookup`]。
    pub(crate) fn get_with_lookup(
        &mut self,
        lookup: &ShapedRunCacheLookupKey<'_>,
        text: &str,
    ) -> Option<Arc<ShapedGlyphRun>> {
        let mut collision_seen = false;
        let exact_lookup = self
            .lookup_buckets
            .get(&lookup.full_fingerprint())
            .map(|candidates| {
                self.index.find_in_slots(candidates, |entry| {
                    if !entry.key.matches_lookup(lookup) {
                        return false;
                    }
                    if entry.run.source_text.as_ref() == text {
                        true
                    } else {
                        collision_seen = true;
                        false
                    }
                })
            })
            .unwrap_or_default();
        self.record_lookup(exact_lookup.candidate_count);
        let hit_slot = if exact_lookup.slot.is_some() || !is_single_text_paragraph(text) {
            exact_lookup.slot
        } else {
            let alias_lookup = self
                .direction_alias_buckets
                .get(&lookup.direction_alias_fingerprint())
                .map(|candidates| {
                    self.index.find_in_slots(candidates, |entry| {
                        entry
                            .key
                            .can_reuse_resolved_direction_for_lookup(lookup, &entry.run)
                            && entry.run.source_text.as_ref() == text
                    })
                })
                .unwrap_or_default();
            self.record_lookup(alias_lookup.candidate_count);
            alias_lookup.slot
        };

        let Some(slot) = hit_slot else {
            self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
            if collision_seen {
                self.frame_report.collision_miss_count =
                    self.frame_report.collision_miss_count.saturating_add(1);
            }
            return None;
        };

        let run = Arc::clone(&self.index.entry(slot)?.run);
        self.index.touch(slot);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(run)
    }

    /// 复制查询键中的字符串与特征载荷，生成可脱离查询借用持有的键。
    ///
    /// 返回键仍只保存文本指纹，不复制或持有原文。
    /// 报告只累加本入口载荷长度，未计容量、控制块或其他分配。
    /// 参考 [`ShapedRunCacheLookupKey`]。
    pub(crate) fn own_lookup_key(
        &mut self,
        lookup: &ShapedRunCacheLookupKey<'_>,
    ) -> ShapedRunCacheKey {
        self.frame_report.owned_key_allocation_bytes = self
            .frame_report
            .owned_key_allocation_bytes
            .saturating_add(lookup.owned_key_byte_len() as u64);
        ShapedRunCacheKey::from_lookup(lookup)
    }

    /// 将拥有式产物包装为共享引用后插入；返回语义见 [`Self::insert_ready`]。
    pub(crate) fn insert(
        &mut self,
        key: ShapedRunCacheKey,
        run: ShapedGlyphRun,
    ) -> Arc<ShapedGlyphRun> {
        self.insert_ready(key, Arc::new(run))
    }

    /// 复用传入共享产物，更新同键同原文的条目或新增条目，再执行限制整理。
    ///
    /// 返回所传产物的共享引用，即使条目在整理中被逐出仍可持有。
    /// 调试断言仅核对 Unicode 快照，不检查所有字体代次或就绪状态。
    /// 调用方需先验证上述状态；原文字节从产物读取。
    /// 参考 [`Self::with_limits`]。
    /// Inserts a successful shaping artifact without copying its glyph storage.
    pub(crate) fn insert_ready(
        &mut self,
        key: ShapedRunCacheKey,
        run: Arc<ShapedGlyphRun>,
    ) -> Arc<ShapedGlyphRun> {
        debug_assert_eq!(
            key.unicode_data_snapshot, run.unicode_data_snapshot,
            "shaped-cache key and artifact must share one Unicode snapshot"
        );
        let estimated_bytes = estimated_entry_bytes(&key, run.as_ref());

        let lookup = self
            .index
            .find_slot(&key, |entry| entry.run.source_text == run.source_text);
        self.record_lookup(lookup.candidate_count);
        if let Some(slot) = lookup.slot {
            let updated = if let Some(entry) = self.index.entry_mut(slot) {
                self.estimated_bytes = self
                    .estimated_bytes
                    .saturating_sub(entry.estimated_bytes)
                    .saturating_add(estimated_bytes);
                entry.run = Arc::clone(&run);
                entry.estimated_bytes = estimated_bytes;
                true
            } else {
                false
            };
            if updated {
                self.index.touch(slot);
                self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
                self.trim_to_limits();
                return run;
            }
        }

        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        let lookup = key.lookup();
        let lookup_fingerprint = lookup.full_fingerprint();
        let alias_fingerprint = lookup.direction_alias_fingerprint();
        let slot = self.index.insert(ShapedRunCacheEntry {
            key,
            run: Arc::clone(&run),
            estimated_bytes,
        });
        Self::insert_bucket_slot(
            &mut self.lookup_buckets,
            &mut self.lookup_bucket_positions,
            lookup_fingerprint,
            slot,
        );
        Self::insert_bucket_slot(
            &mut self.direction_alias_buckets,
            &mut self.direction_alias_bucket_positions,
            alias_fingerprint,
            slot,
        );
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.trim_to_limits();
        run
    }

    /// 先查缓存；仅未命中时调用一次 `shape`，并将其返回产物插入。
    /// 调用方负责让键、`text` 和闭包产物描述同一已规范化请求。
    pub(crate) fn get_or_insert_with(
        &mut self,
        key: ShapedRunCacheKey,
        text: &str,
        shape: impl FnOnce() -> ShapedGlyphRun,
    ) -> Arc<ShapedGlyphRun> {
        if let Some(run) = self.get(&key, text) {
            return run;
        }
        self.insert(key, shape())
    }

    // 逐出时同步移除两个辅助桶的槽映射，避免留下已失效的候选。
    fn trim_to_limits(&mut self) {
        let mut evicted = 0_u64;
        while self.over_limits() {
            let Some((slot, removed, work)) = self.index.pop_oldest_with_slot_and_work() else {
                break;
            };
            self.frame_report.eviction_scan_count = self
                .frame_report
                .eviction_scan_count
                .saturating_add(work.scan_count as u64);
            self.frame_report.entry_move_count = self
                .frame_report
                .entry_move_count
                .saturating_add(work.entry_move_count as u64);
            let lookup = removed.key.lookup();
            self.remove_lookup_slot(lookup.full_fingerprint(), slot);
            self.remove_direction_alias_slot(lookup.direction_alias_fingerprint(), slot);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(removed.estimated_bytes);
            evicted = evicted.saturating_add(1);
        }

        if evicted > 0 {
            self.frame_report.evicted_count =
                self.frame_report.evicted_count.saturating_add(evicted);
            self.frame_report.trim_count = self.frame_report.trim_count.saturating_add(1);
        }
        self.refresh_report_size();
    }

    fn over_limits(&self) -> bool {
        self.index.len() > self.capacity || self.estimated_bytes > self.max_bytes
    }

    fn record_lookup(&mut self, candidate_count: usize) {
        self.frame_report.lookup_candidate_count = self
            .frame_report
            .lookup_candidate_count
            .saturating_add(candidate_count as u64);
    }

    fn remove_lookup_slot(&mut self, fingerprint: EphemeralCacheHash, slot: TextCacheSlot) {
        let remove_bucket = if let Some(candidates) = self.lookup_buckets.get_mut(&fingerprint) {
            Self::remove_bucket_slot(candidates, &mut self.lookup_bucket_positions, slot);
            candidates.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.lookup_buckets.remove(&fingerprint);
        }
    }

    fn remove_direction_alias_slot(
        &mut self,
        fingerprint: EphemeralCacheHash,
        slot: TextCacheSlot,
    ) {
        let remove_bucket = if let Some(candidates) =
            self.direction_alias_buckets.get_mut(&fingerprint)
        {
            Self::remove_bucket_slot(candidates, &mut self.direction_alias_bucket_positions, slot);
            candidates.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.direction_alias_buckets.remove(&fingerprint);
        }
    }

    fn insert_bucket_slot(
        buckets: &mut HashMap<EphemeralCacheHash, Vec<TextCacheSlot>>,
        positions: &mut HashMap<TextCacheSlot, usize>,
        fingerprint: EphemeralCacheHash,
        slot: TextCacheSlot,
    ) {
        let candidates = buckets.entry(fingerprint).or_default();
        let position = candidates.len();
        candidates.push(slot);
        positions.insert(slot, position);
    }

    // 用保存的位置作交换删除；尾槽移入后立即修正位置，保持下一次删除可定位。
    fn remove_bucket_slot(
        candidates: &mut Vec<TextCacheSlot>,
        positions: &mut HashMap<TextCacheSlot, usize>,
        slot: TextCacheSlot,
    ) {
        let Some(position) = positions.remove(&slot) else {
            return;
        };
        let removed = candidates.swap_remove(position);
        debug_assert_eq!(removed, slot);
        if let Some(&moved_slot) = candidates.get(position) {
            positions.insert(moved_slot, position);
        }
    }

    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.index.len();
        self.frame_report.estimated_bytes = self.estimated_bytes;
    }
}

impl ShapedRunCacheKey {
    // 方向别名只放宽基础方向；其余身份及特征仍作精确比较。
    fn can_reuse_resolved_direction_for_lookup(
        &self,
        requested: &ShapedRunCacheLookupKey<'_>,
        cached_run: &ShapedGlyphRun,
    ) -> bool {
        matches!(
            self.base_direction,
            TextDirection::Auto | TextDirection::Mixed
        ) && matches!(
            requested.base_direction,
            TextDirection::LeftToRight | TextDirection::RightToLeft
        ) && cached_run.direction == requested.base_direction
            && self.matches_lookup_except_base_direction(requested)
    }

    fn matches_lookup_except_base_direction(&self, lookup: &ShapedRunCacheLookupKey<'_>) -> bool {
        self.text_hash == lookup.text_hash
            && self.source_range == lookup.source_range
            && self.font_asset.as_deref() == lookup.font_asset
            && self.font_family.as_deref() == lookup.font_family
            && self.font_weight == lookup.font_weight
            && self.font_italic == lookup.font_italic
            && self.font_size_bits == lookup.font_size_bits
            && self.line_height_bits == lookup.line_height_bits
            && self.tab_size_bits == lookup.tab_size_bits
            && self.orientation == lookup.orientation
            && self.vertical_mode == lookup.vertical_mode
            && self.include_kerning == lookup.include_kerning
            && self.features_hash == lookup.features_hash
            && self.features.as_ref() == lookup.features
            && self.language.as_deref() == lookup.language
            && self.font_collection == lookup.font_collection
            && self.font_database_generation == lookup.font_database_generation
            && self.unicode_data_snapshot == lookup.unicode_data_snapshot
    }
}

/// 将规定的行、段分隔字符视为禁止方向别名的边界。
fn is_single_text_paragraph(text: &str) -> bool {
    !text.chars().any(|ch| {
        matches!(
            ch,
            '\n' | '\r'
                | '\u{001c}'
                | '\u{001d}'
                | '\u{001e}'
                | '\u{0085}'
                | '\u{2028}'
                | '\u{2029}'
        )
    })
}

fn hash_text(text: &str) -> EphemeralCacheHash {
    EphemeralCacheHash::from_hashable(text)
}

fn shaping_features_hash(request: &BackendShapeRequest<'_>) -> EphemeralCacheHash {
    let mut hasher = EphemeralCacheHasher::new();
    hasher.write(b"shaped-run-features-v2");
    hasher.write(&request.include_kerning);
    hasher.write(request.features());
    hasher.finish()
}

// 家族名称的空白归一化不适用于字体资源身份；后者仅排除空串。
fn cache_text_identity_ref(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn cache_font_asset_ref(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.is_empty())
}

// 仅统一正负零的位表示，其余位模式原样参与身份。
fn normalized_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else {
        value.to_bits()
    }
}

#[cfg(test)]
#[path = "shaped_cache/tests/cases.rs"]
mod tests;
