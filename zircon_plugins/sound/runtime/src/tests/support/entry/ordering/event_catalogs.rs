use super::super::super::StaticEventCatalog;

pub(super) fn sort_static_event_catalogs(event_catalogs: &mut [StaticEventCatalog]) {
    event_catalogs.sort_unstable();
}
