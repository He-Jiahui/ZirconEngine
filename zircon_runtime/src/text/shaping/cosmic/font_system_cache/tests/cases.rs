//! 检查调用方字体快照的保留契约与可选 profiling 事件。
//! 直接构造局部 cache，不覆盖线程隔离、重入、容量淘汰或计时表现。

use super::*;
#[cfg(feature = "profiling")]
use crate::core::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};
use crate::text::font::{
    force_publish_shared_font_database, shared_font_collection_snapshot,
    shared_font_database_snapshot, shared_font_database_test_serial_guard,
};

/// 保存原数据库内容供离开测试作用域时重新发布，不恢复旧 generation 数值。
/// 唯一字段为自有数据库副本，随恢复器一起存活。
struct SharedFontDatabaseRestore(FontDatabase);

impl Drop for SharedFontDatabaseRestore {
    /// 在正常离域或展开清理时发布保存的数据库副本。
    /// `self` 仅在析构期间独占借用；helper 返回的新 generation 在此被丢弃。
    fn drop(&mut self) {
        force_publish_shared_font_database(&self.0);
    }
}

/// 发布不同默认字体后，仍用 retired 快照建立 cache 并检查回调数据库。
/// 不直接检查 FontSystem 内部数据库，也不经过 TLS 外层入口。
#[test]
fn locale_font_system_cache_uses_the_supplied_generation_snapshot() {
    let _font_database_guard = shared_font_database_test_serial_guard();
    let retired = shared_font_collection_snapshot();
    let retired_family = retired
        .database()
        .default_ui_family_for_test()
        .map(str::to_owned);
    let (_, original_database) = shared_font_database_snapshot();
    // 恢复器晚于串行 guard 构造，先发布原内容，再释放测试间的串行窗口。
    let _restore = SharedFontDatabaseRestore(original_database.clone());
    let mut replacement = original_database.clone();
    assert!(replacement.set_default_ui_family("Generation Snapshot Test UI"));
    let replacement_generation = force_publish_shared_font_database(&replacement);
    assert!(replacement_generation > retired.generation());

    let mut cache = LocaleFontSystemCache::new(&retired);
    cache.with_font_system(&retired, Some("en-US"), |_, database| {
        assert_eq!(
            database.default_ui_family_for_test().map(str::to_owned),
            retired_family,
            "cosmic shaping must read the database retained by the supplied snapshot"
        );
    });
}

/// 在采集开始后触发新 locale 与 generation 刷新，核对事件名称存在。
/// 仅 profiling feature 参与；不检查数值、事件顺序、耗时或淘汰行为。
#[cfg(feature = "profiling")]
#[test]
fn locale_font_system_cache_profiles_insert_and_generation_refresh() {
    let _capture_guard = test_capture_lock();
    let _font_database_guard = shared_font_database_test_serial_guard();
    let initial_snapshot = shared_font_collection_snapshot();
    // 初始 cache 在启动采集前构造，下面只检索后续插入与刷新事件。
    let mut cache = LocaleFontSystemCache::new(&initial_snapshot);
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "text01-locale-font-system-cache".to_owned();
    config.max_spans = 4;
    config.max_counters = 8;
    assert!(start_capture(config).active);

    cache.with_font_system(&initial_snapshot, Some("zh-Hans"), |_, _| ());
    let (_, database) = shared_font_database_snapshot();
    // 测试发布 helper 即便内容相同也推进 generation，下一次访问因此重建实例。
    force_publish_shared_font_database(&database);
    let reloaded_snapshot = shared_font_collection_snapshot();
    cache.with_font_system(&reloaded_snapshot, Some("ja-JP"), |_, _| ());

    // 正常路径先复制本次采集结果再清空 recorder；锁 guard 本身不恢复旧采集内容。
    let profile = snapshot();
    assert!(!reset_capture().active);
    for name in ["locale_insert", "generation_refresh"] {
        assert!(
            profile.spans.iter().any(|span| {
                span.stream == "runtime"
                    && span.category == "text.font_system_cache"
                    && span.name == name
            }),
            "locale font-system cache must expose {name} work to profiling"
        );
    }
    for name in [
        "text.font_system_cache.locale_insert_entry_count",
        "text.font_system_cache.generation_refresh_entry_count",
        "text.font_system_cache.generation_refresh_face_count",
    ] {
        assert!(
            profile
                .counters
                .iter()
                .any(|counter| counter.stream == "runtime" && counter.name == name),
            "locale font-system cache must emit {name}"
        );
    }
}
