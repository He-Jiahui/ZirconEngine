use crate::text::font::{FallbackCacheRequestProfile, FontDatabase};

/// 为本次字体后备解析开启请求级锁统计；返回的开关必须在同一请求结束时传回 finish。
/// 即使解析返回错误，也先取走统计状态，避免线程上的下一请求继承前次画像。
pub(super) fn begin(database: &FontDatabase) -> bool {
    let enabled = profile_metrics_enabled();
    if enabled {
        database.begin_fallback_cache_profile_request();
    }
    enabled
}

pub(super) fn finish(database: &FontDatabase, active: bool) {
    if !active {
        return;
    }
    let Some(profile) = database.take_fallback_cache_profile_request() else {
        return;
    };
    record(profile);
}

fn record(profile: FallbackCacheRequestProfile) {
    crate::profile_counter!(
        "runtime",
        "text_font_fallback_cache_state_lock_acquire_count",
        profile.state_lock_acquire_count
    );
    crate::profile_counter!(
        "runtime",
        "text_font_fallback_cache_state_lock_wait_nanos",
        profile.state_lock_wait_nanos
    );
    crate::profile_counter!(
        "runtime",
        "text_font_fallback_cache_state_lock_hold_nanos",
        profile.state_lock_hold_nanos
    );
}

fn profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}
