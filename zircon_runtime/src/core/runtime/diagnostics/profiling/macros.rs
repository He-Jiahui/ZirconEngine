#[macro_export]
/// 启用对应 feature 时分别展开 recorder RAII scope 与 Tracy span。
macro_rules! profile_scope {
    ($stream:expr, $category:expr, $name:expr $(,)?) => {
        #[cfg(feature = "profiling")]
        let _zr_profile_scope =
            $crate::core::diagnostics::profiling::ProfileScope::enter($stream, $category, $name);
        #[cfg(feature = "profiling-tracy")]
        let _zr_profile_tracy_span = tracing::info_span!(
            "zircon.profile.scope",
            stream = $stream,
            category = $category,
            name = $name,
        )
        .entered();
    };
}

#[macro_export]
/// 普通 profiling 仅在采集活动时求值动态名称；Tracy 路径持有同一名称供两路记录复用。
macro_rules! profile_dynamic_scope {
    ($stream:expr, $category:expr, $name:expr $(,)?) => {
        #[cfg(feature = "profiling-tracy")]
        let _zr_profile_dynamic_scope_name: String = ($name).into();
        #[cfg(feature = "profiling-tracy")]
        let _zr_profile_dynamic_tracy_span = tracing::info_span!(
            "zircon.profile.scope",
            stream = $stream,
            category = $category,
            name = %_zr_profile_dynamic_scope_name,
        )
        .entered();
        #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
        let _zr_profile_dynamic_scope = $crate::core::diagnostics::profiling::capture_active()
            .then(|| {
                $crate::core::diagnostics::profiling::ProfileScope::enter_named(
                    $stream, $category, $name,
                )
            });
        #[cfg(all(feature = "profiling", feature = "profiling-tracy"))]
        let _zr_profile_dynamic_scope = $crate::core::diagnostics::profiling::capture_active()
            .then(|| {
                $crate::core::diagnostics::profiling::ProfileScope::enter_named(
                    $stream,
                    $category,
                    _zr_profile_dynamic_scope_name,
                )
            });
    };
}

#[macro_export]
/// recorder 帧作用域退出时收口；Tracy frame marker/span 是并行观测。
macro_rules! profile_frame {
    ($stream:expr, $name:expr $(,)?) => {
        #[cfg(feature = "profiling")]
        let _zr_profile_frame =
            $crate::core::diagnostics::profiling::ProfileFrameScope::enter($stream, $name);
        #[cfg(feature = "profiling-tracy")]
        let _zr_profile_tracy_frame_mark =
            $crate::core::diagnostics::profiling::TracyFrameScope::enter($stream, $name);
        #[cfg(feature = "profiling-tracy")]
        let _zr_profile_tracy_frame =
            tracing::info_span!("zircon.profile.frame", stream = $stream, name = $name,).entered();
    };
}

#[macro_export]
/// recorder 只在采集活动时接收计数；Tracy feature 还独立发出 tracing counter。
macro_rules! profile_counter {
    ($stream:expr, $name:expr, $value:expr $(,)?) => {{
        #[cfg(feature = "profiling-tracy")]
        let _zr_profile_counter_value = $value as f64;
        #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
        if $crate::core::diagnostics::profiling::capture_active() {
            $crate::core::diagnostics::profiling::record_counter($stream, $name, $value as f64);
        }
        #[cfg(all(feature = "profiling", feature = "profiling-tracy"))]
        if $crate::core::diagnostics::profiling::capture_active() {
            $crate::core::diagnostics::profiling::record_counter(
                $stream,
                $name,
                _zr_profile_counter_value,
            );
        }
        #[cfg(feature = "profiling-tracy")]
        tracing::info!(
            target: "zircon.profile.counter",
            stream = $stream,
            name = $name,
            value = _zr_profile_counter_value,
        );
    }};
}

#[cfg(test)]
#[path = "macros/tests/dynamic_name_handoff_tests.rs"]
mod dynamic_name_handoff_tests;
