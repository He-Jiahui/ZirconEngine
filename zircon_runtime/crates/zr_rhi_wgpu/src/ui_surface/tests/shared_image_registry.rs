use super::{shared_image_admission_plan, ImageCacheAdmissionAction};

#[test]
fn shared_registry_evicts_the_least_recent_cross_window_texture() {
    let entries = [
        ("older", 1, 3, 32 * 1024 * 1024, true, false),
        ("newer", 1, 8, 32 * 1024 * 1024, true, false),
    ];

    let action =
        shared_image_admission_plan(entries.into_iter(), 3, 80 * 1024 * 1024, 16 * 1024 * 1024);

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Admit {
            evict_keys: vec![("older".to_owned(), 1)],
        }
    );
}

#[test]
fn shared_registry_never_evicts_the_resource_being_replaced() {
    let entries = [("target", 7, 1, 60 * 1024 * 1024, true, true)];

    let action =
        shared_image_admission_plan(entries.into_iter(), 1, 65 * 1024 * 1024, 65 * 1024 * 1024);

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Reject {
            entry_saturated: false,
        }
    );
}

#[test]
fn shared_registry_counts_pinned_evictions_as_live_device_bytes() {
    let entries = [
        ("pinned", 1, 1, 32 * 1024 * 1024, false, false),
        ("releasable", 1, 2, 32 * 1024 * 1024, true, false),
    ];

    let action =
        shared_image_admission_plan(entries.into_iter(), 3, 80 * 1024 * 1024, 16 * 1024 * 1024);

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Admit {
            evict_keys: vec![("pinned".to_owned(), 1), ("releasable".to_owned(), 1),],
        }
    );
}

#[test]
fn shared_registry_rejects_when_only_surface_pinned_bytes_remain() {
    let entries = [("pinned", 1, 1, 64 * 1024 * 1024, false, false)];

    let action = shared_image_admission_plan(entries.into_iter(), 2, 65 * 1024 * 1024, 1024 * 1024);

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Reject {
            entry_saturated: false,
        }
    );
}
