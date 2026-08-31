use std::time::{Duration, Instant};

const TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES: [&str; 12] = [
    "text_document_grapheme_query_count",
    "text_document_grapheme_query_nanos",
    "text_document_grapheme_binary_search_count",
    "text_document_grapheme_index_hit_count",
    "text_document_grapheme_index_rebuild_count",
    "text_document_grapheme_index_rebuild_input_bytes",
    "text_document_grapheme_index_rebuild_boundary_count",
    "text_document_grapheme_index_rebuild_nanos",
    "text_document_grapheme_index_incremental_update_count",
    "text_document_grapheme_index_incremental_update_input_bytes",
    "text_document_grapheme_index_incremental_update_boundary_count",
    "text_document_grapheme_index_incremental_update_nanos",
];

pub(super) fn start_query() -> Option<Instant> {
    if !profile_metrics_enabled() {
        return None;
    }
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[0],
        1
    );
    Some(Instant::now())
}

pub(super) fn finish_query(started: Option<Instant>) {
    let Some(started) = started else {
        return;
    };
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[1],
        duration_to_nanos(started.elapsed())
    );
}

pub(super) fn record_binary_searches(count: usize) {
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[2],
        count
    );
}

pub(super) fn record_index_hit() {
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[3],
        1
    );
}

pub(super) fn start_index_rebuild() -> Option<Instant> {
    if !profile_metrics_enabled() {
        return None;
    }
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[4],
        1
    );
    Some(Instant::now())
}

pub(super) fn finish_index_rebuild(
    input_bytes: usize,
    boundary_count: usize,
    started: Option<Instant>,
) {
    let Some(started) = started else {
        return;
    };
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[5],
        input_bytes
    );
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[6],
        boundary_count
    );
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[7],
        duration_to_nanos(started.elapsed())
    );
}

pub(super) fn start_incremental_update() -> Option<Instant> {
    if !profile_metrics_enabled() {
        return None;
    }
    Some(Instant::now())
}

pub(super) fn finish_incremental_update(
    input_bytes: usize,
    boundary_count: usize,
    started: Option<Instant>,
) {
    let Some(started) = started else {
        return;
    };
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[8],
        1
    );
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[9],
        input_bytes
    );
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[10],
        boundary_count
    );
    crate::profile_counter!(
        "runtime",
        TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES[11],
        duration_to_nanos(started.elapsed())
    );
}

fn duration_to_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES;

    #[test]
    fn grapheme_index_profile_uses_only_fixed_names() {
        let unique = TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(unique.len(), 12);
        assert!(
            unique
                .iter()
                .all(|name| name.starts_with("text_document_grapheme_"))
        );
    }
}
