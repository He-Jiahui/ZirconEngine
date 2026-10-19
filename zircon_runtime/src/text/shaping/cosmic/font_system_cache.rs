//! 为当前线程复用一组 locale 对应的 Cosmic FontSystem 与同一集合快照。
//! 非空 shaping 请求在尝试直接后端前就进入此层；这里不选择最终 shaping 后端。
//! 容量约束是实例条目数，不限制字体数据、内部缓存或实际内存。

use std::cell::RefCell;

use glyphon::cosmic_text::{fontdb, FontSystem};

use crate::text::default_text_locale;
use crate::text::font::{FontCollectionSnapshot, FontDatabase};

/// 每线程保留的 locale 实例总上限，包含默认条目。
const MAX_LOCALE_FONT_SYSTEMS: usize = 4;

thread_local! {
    // Option 将较重的初始化推迟到该线程第一次访问；可变状态不跨线程共用。
    static FONT_SYSTEMS: RefCell<Option<LocaleFontSystemCache>> = RefCell::new(None);
}

/// 一个精确 locale 键及其自有字体系统，不借用调用方字符串。
struct LocaleFontSystemEntry {
    /// 按字符串值匹配；缓存层不做语言标签规范化。
    locale: String,
    /// 自有 Cosmic 状态，包含其数据库和内部匹配、shaping 缓存。
    font_system: FontSystem,
    /// 本 cache 内最近访问的饱和逻辑序号，不是时间戳或耗时。
    last_used: u64,
}

/// 当前线程、当前集合 revision 下的一组 locale 实例。
/// 切换集合或 generation 时重建各实例，locale 列表及访问序号仍保留。
struct LocaleFontSystemCache {
    /// 从当前快照复制的后端数据库，供后续 locale 实例再次克隆。
    seed_database: fontdb::Database,
    /// 自有快照克隆持有数据库 Arc，调用方释放原快照后本缓存仍可使用。
    font_collection: FontCollectionSnapshot,
    /// 构造时确定的默认 locale；None 使用它，淘汰搜索优先保留它。
    fallback_locale: String,
    /// 自有条目向量；正常构造、插入路径使键唯一并保持容量上限。
    entries: Vec<LocaleFontSystemEntry>,
    /// 单调不减的访问序号；饱和后相同值不再区分严格新旧次序。
    use_order: u64,
}

impl LocaleFontSystemCache {
    /// 首次使用时建立默认 locale 条目及其集合资源 lease。
    ///
    /// # 参数
    /// - `font_collection`：只借用本次构造；缓存保存其自有克隆，不改写外部集合。
    ///
    /// # 返回与约束
    /// 返回仅有一个默认条目的 Self，初始访问序号为 1。
    /// 后端数据库和 FontSystem 都是自有值；快照内的 Arc 仍可共享字体资源。
    /// 参见 FontCollectionSnapshot 的 publication 生命周期约束。
    fn new(font_collection: &FontCollectionSnapshot) -> Self {
        let fallback_locale = default_text_locale();
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let face_count = font_collection.database().face_count();
        let (seed_database, font_system) = {
            crate::profile_scope!("runtime", "text.font_system_cache", "thread_initialize");
            let seed_database = font_collection.database().backend_database_snapshot();
            // Keep cosmic on the process-level FontDatabase from first use. Constructing a
            // standalone FontSystem here asks fontdb to rediscover the operating-system fonts,
            // which both violates the text-plan ownership boundary and stalls the first UI layout.
            let font_system =
                FontSystem::new_with_locale_and_db(fallback_locale.clone(), seed_database.clone());
            (seed_database, font_system)
        };
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.thread_initialize_face_count",
            face_count
        );
        Self {
            seed_database,
            font_collection: font_collection.clone(),
            fallback_locale: fallback_locale.clone(),
            entries: vec![LocaleFontSystemEntry {
                locale: fallback_locale,
                font_system,
                last_used: 1,
            }],
            use_order: 1,
        }
    }

    /// 在指定快照与 locale 的缓存实例上同步执行一次回调。
    ///
    /// # 参数
    /// - `self`：独占借用 cache；本次调用可能刷新所有实例或淘汰一个条目。
    /// - `font_collection`：期望使用的集合 revision，变化时据它重建状态。
    /// - `language`：调用期间借用的精确 locale 键；None 选择默认值，新增键复制为 String。
    /// - `use_system`：按值消费的 FnOnce，正常路径只调用一次；接收字体系统与数据库借用。
    ///
    /// # 返回与约束
    /// 原样返回回调的 R；R 可来自外部捕获，不能携带对本次缓存参数的借用。
    /// 回调数据库来自保留的快照，与用于重建 FontSystem 的 revision 一致。
    fn with_font_system<R>(
        &mut self,
        font_collection: &FontCollectionSnapshot,
        language: Option<&str>,
        use_system: impl FnOnce(&mut FontSystem, &FontDatabase) -> R,
    ) -> R {
        self.refresh_font_collection(font_collection);
        let locale = language.unwrap_or(self.fallback_locale.as_str());
        self.use_order = self.use_order.saturating_add(1);
        let existing_index = self.entries.iter().position(|entry| entry.locale == locale);
        let index = if let Some(index) = existing_index {
            index
        } else {
            let locale = locale.to_owned();
            self.insert_locale(locale)
        };
        self.entries[index].last_used = self.use_order;
        use_system(
            &mut self.entries[index].font_system,
            self.font_collection.database(),
        )
    }

    /// 根据集合 ID 与 generation 同步全部 locale 的字体系统。
    ///
    /// # 参数
    /// - `self`：独占修改，已有 locale 和访问序号保持不变。
    /// - `font_collection`：只借用调用方快照；实际刷新结束后保存其克隆。
    ///
    /// # 返回与约束
    /// 无返回值；先替换 seed 并逐项重建，再切换持有的快照。
    /// 这里没有事务提交或异常回滚机制；参见 FontCollectionService 的 revision 发布规则。
    fn refresh_font_collection(&mut self, font_collection: &FontCollectionSnapshot) {
        // Equal generations have equivalent shaping/raster inputs even when a
        // diagnostic-only mutation published a different Arc.
        if self.font_collection.collection_id() == font_collection.collection_id()
            && self.font_collection.generation() == font_collection.generation()
        {
            return;
        }
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let refreshed_entry_count = self.entries.len();
        {
            crate::profile_scope!("runtime", "text.font_system_cache", "generation_refresh");
            self.seed_database = font_collection.database().backend_database_snapshot();
            for entry in &mut self.entries {
                entry.font_system = FontSystem::new_with_locale_and_db(
                    entry.locale.clone(),
                    self.seed_database.clone(),
                );
            }
        }
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let refreshed_face_count = font_collection.database().face_count();
        // 到这里各实例已使用新 seed；最后切换保留快照的资源 lease。
        self.font_collection = font_collection.clone();
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.generation_refresh_entry_count",
            refreshed_entry_count
        );
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.generation_refresh_face_count",
            refreshed_face_count
        );
    }

    /// 为缺失的 locale 建立实例，必要时腾出一个非默认条目。
    ///
    /// # 参数
    /// - `self`：独占借用；调用方已确认该键不存在。
    /// - `locale`：消费自有字符串，一份保留为键，另一份交给 FontSystem。
    ///
    /// # 返回与约束
    /// 返回新条目的向量下标，供同一次访问立即定位；后续插入、移除可能使旧下标失效。
    /// 正常路径默认条目始终存在，满额时非默认候选必然存在。
    /// 最小访问序号优先淘汰；饱和或相同序号时不承诺严格全序。
    fn insert_locale(&mut self, locale: String) -> usize {
        let (index, evicted_entry) = {
            crate::profile_scope!("runtime", "text.font_system_cache", "locale_insert");
            let mut evicted_entry = false;
            if self.entries.len() >= MAX_LOCALE_FONT_SYSTEMS {
                let eviction_index = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| entry.locale != self.fallback_locale)
                    .min_by_key(|(_, entry)| entry.last_used)
                    .map(|(index, _)| index)
                    .unwrap_or(0);
                self.entries.remove(eviction_index);
                evicted_entry = true;
            }
            self.entries.push(LocaleFontSystemEntry {
                locale: locale.clone(),
                font_system: FontSystem::new_with_locale_and_db(locale, self.seed_database.clone()),
                last_used: self.use_order,
            });
            (self.entries.len() - 1, evicted_entry)
        };
        // 这些计数描述条目数量和本次是否淘汰，不代表字体或内存分配统计。
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let entry_count = self.entries.len();
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.locale_insert_entry_count",
            entry_count
        );
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "text.font_system_cache.locale_insert_evicted_entry",
            u8::from(evicted_entry)
        );
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = evicted_entry;
        index
    }
}

/// 借用当前线程的缓存，在调用方提供的字体集合上运行同步回调。
///
/// # 参数
/// - `font_collection`：期望的快照；首次访问会克隆它，后续按 revision 判断刷新。
/// - `language`：仅在调用期间借用的 locale；缓存只为新条目保存自有字符串。
/// - `use_system`：消费的 FnOnce，接收可变 FontSystem 与共享 FontDatabase 的临时借用。
///
/// # 返回与约束
/// 返回回调的 R，不另加错误或异步层；实际调用方返回 shaping 的 Result。
/// 同线程回调不得重入此入口；RefCell 的独占借用直到回调返回才释放。
/// 参见 cosmic::shape_with_cosmic；缓存内引用不能随 R 逃逸。
pub(super) fn with_font_system<R>(
    font_collection: &FontCollectionSnapshot,
    language: Option<&str>,
    use_system: impl FnOnce(&mut FontSystem, &FontDatabase) -> R,
) -> R {
    FONT_SYSTEMS.with(|systems| {
        let mut systems = systems.borrow_mut();
        let cache = systems.get_or_insert_with(|| LocaleFontSystemCache::new(font_collection));
        cache.with_font_system(font_collection, language, use_system)
    })
}

#[cfg(test)]
#[path = "font_system_cache/tests/cases.rs"]
mod tests;
