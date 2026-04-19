use notify::{Event, RecommendedWatcher};

pub(super) fn recommended_watcher(
    handler: impl FnMut(notify::Result<Event>) + Send + 'static,
) -> Result<RecommendedWatcher, notify::Error> {
    notify::recommended_watcher(handler)
}
