use super::ViewportHighlightStore;
use crate::core::framework::render::{HighlightRenderAttributes, HighlightSet};

fn set(entities: impl IntoIterator<Item = u64>) -> HighlightSet {
    HighlightSet::new(
        entities,
        HighlightRenderAttributes::outlined([0.1, 0.2, 0.3, 1.0]),
    )
}

#[test]
fn rejects_stale_generation_without_cross_viewport_leakage() {
    let mut store = ViewportHighlightStore::default();
    assert!(store.submit(3, 7, set([8, 2])));
    assert!(store.submit(4, 1, set([11])));
    assert!(!store.submit(3, 6, set([99])));

    assert_eq!(store.get(3).unwrap().generation(), 7);
    assert_eq!(store.get(3).unwrap().set().entities(), &[2, 8]);
    assert_eq!(store.get(4).unwrap().set().entities(), &[11]);
}

#[test]
fn overlay_revision_tracks_payload_changes_independently_of_selection_generation() {
    let mut store = ViewportHighlightStore::default();
    let original = set([8, 2]);
    assert!(store.submit(3, 7, original.clone()));
    assert_eq!(store.get(3).unwrap().overlay_revision(), 1);

    assert!(store.submit(3, 7, original.clone()));
    assert!(store.submit(3, 8, original));
    assert_eq!(store.get(3).unwrap().generation(), 8);
    assert_eq!(store.get(3).unwrap().overlay_revision(), 1);

    assert!(store.submit(3, 8, set([8, 5])));
    assert_eq!(store.get(3).unwrap().overlay_revision(), 2);
    let tint_changed = HighlightSet::new(
        [8, 5],
        HighlightRenderAttributes::outlined([0.5, 0.2, 0.3, 1.0]),
    );
    assert!(store.submit(3, 8, tint_changed));
    assert_eq!(store.get(3).unwrap().overlay_revision(), 3);
    let outline_changed = HighlightSet::new(
        [8, 5],
        HighlightRenderAttributes {
            outline_enabled: false,
            tint_rgba: [0.5, 0.2, 0.3, 1.0],
        },
    );
    assert!(store.submit(3, 8, outline_changed));
    assert_eq!(store.get(3).unwrap().overlay_revision(), 4);

    assert!(!store.submit(3, 7, set([99])));
    assert_eq!(store.get(3).unwrap().generation(), 8);
    assert_eq!(store.get(3).unwrap().overlay_revision(), 4);
    assert_eq!(store.get(3).unwrap().set().entities(), &[5, 8]);

    assert!(store.submit(4, 1, set([11])));
    assert_eq!(store.get(4).unwrap().overlay_revision(), 1);
    assert_eq!(store.get(3).unwrap().overlay_revision(), 4);
}

#[test]
fn invalid_render_attributes_do_not_change_latest_value_or_revision() {
    let mut store = ViewportHighlightStore::default();
    let invalid = HighlightSet::new(
        [8],
        HighlightRenderAttributes::outlined([f32::NAN, 0.2, 0.3, 1.0]),
    );
    assert!(!store.submit(3, 7, invalid.clone()));
    assert!(store.get(3).is_none());

    assert!(store.submit(3, 7, set([8])));
    assert!(!store.submit(3, 8, invalid));
    assert_eq!(store.get(3).unwrap().generation(), 7);
    assert_eq!(store.get(3).unwrap().overlay_revision(), 1);
    assert_eq!(store.get(3).unwrap().set().entities(), &[8]);
}
