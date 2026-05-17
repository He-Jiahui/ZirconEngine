#[macro_export]
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
macro_rules! profile_dynamic_scope {
    ($stream:expr, $category:expr, $name:expr $(,)?) => {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let _zr_profile_dynamic_scope_name: String = ($name).into();
        #[cfg(feature = "profiling")]
        let _zr_profile_dynamic_scope =
            $crate::core::diagnostics::profiling::ProfileScope::enter_named(
                $stream,
                $category,
                _zr_profile_dynamic_scope_name.clone(),
            );
        #[cfg(feature = "profiling-tracy")]
        let _zr_profile_dynamic_tracy_span = tracing::info_span!(
            "zircon.profile.scope",
            stream = $stream,
            category = $category,
            name = %_zr_profile_dynamic_scope_name,
        )
        .entered();
    };
}

#[macro_export]
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
macro_rules! profile_counter {
    ($stream:expr, $name:expr, $value:expr $(,)?) => {
        #[cfg(feature = "profiling")]
        $crate::core::diagnostics::profiling::record_counter($stream, $name, $value as f64);
        #[cfg(feature = "profiling-tracy")]
        tracing::info!(
            target: "zircon.profile.counter",
            stream = $stream,
            name = $name,
            value = $value as f64,
        );
    };
}
