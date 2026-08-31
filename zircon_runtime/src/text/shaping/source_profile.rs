use std::sync::Arc;

use crate::text::BackendShapeRequest;

/// Materializes the current exact request source at the shaped-artifact boundary. Profiling is
/// intentionally request-level: no source label, glyph event, or loop-local observer is emitted.
pub(in crate::text::shaping) fn materialize_source_text(
    request: BackendShapeRequest<'_>,
) -> Arc<str> {
    let reuses_owner = request.has_exact_source_owner();
    let source = request.shared_source_text();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    {
        crate::profile_counter!("runtime", "text_shape_source_materialization_count", 1);
        crate::profile_counter!(
            "runtime",
            "text_shape_source_owner_reuse_count",
            reuses_owner as usize
        );
        crate::profile_counter!(
            "runtime",
            "text_shape_source_allocation_count",
            (!reuses_owner) as usize
        );
        crate::profile_counter!(
            "runtime",
            "text_shape_source_allocation_byte_count",
            if reuses_owner { 0 } else { source.len() }
        );
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = reuses_owner;
    source
}
