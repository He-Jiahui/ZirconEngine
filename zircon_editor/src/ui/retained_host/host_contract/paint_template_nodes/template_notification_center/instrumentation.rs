#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct NotificationPaintCounters {
    pub metrics_resolutions: usize,
    pub palette_resolutions: usize,
    pub row_count_reads: usize,
    pub row_visits: usize,
    pub title_text_copies: usize,
    pub message_text_copies: usize,
}

thread_local! {
    static COUNTERS: std::cell::Cell<NotificationPaintCounters> =
        const { std::cell::Cell::new(NotificationPaintCounters {
            metrics_resolutions: 0,
            palette_resolutions: 0,
            row_count_reads: 0,
            row_visits: 0,
            title_text_copies: 0,
            message_text_copies: 0,
        }) };
}

fn update(update: impl FnOnce(&mut NotificationPaintCounters)) {
    COUNTERS.with(|counters| {
        let mut value = counters.get();
        update(&mut value);
        counters.set(value);
    });
}

pub(super) fn record_metrics_resolution() {
    update(|counters| counters.metrics_resolutions += 1);
}

pub(super) fn record_palette_resolution() {
    update(|counters| counters.palette_resolutions += 1);
}

pub(super) fn record_row_count_read() {
    update(|counters| counters.row_count_reads += 1);
}

pub(super) fn record_row_visit() {
    update(|counters| counters.row_visits += 1);
}

pub(super) fn record_title_text_copy() {
    update(|counters| counters.title_text_copies += 1);
}

pub(super) fn record_message_text_copy() {
    update(|counters| counters.message_text_copies += 1);
}

pub(super) fn reset() {
    COUNTERS.with(|counters| counters.set(NotificationPaintCounters::default()));
}

pub(super) fn snapshot() -> NotificationPaintCounters {
    COUNTERS.with(std::cell::Cell::get)
}
