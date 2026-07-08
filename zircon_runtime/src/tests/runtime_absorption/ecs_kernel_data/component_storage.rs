pub(super) fn assert_component_storage_private_reexport_cleanup() {
    let mod_source = include_str!("../../../scene/ecs/storage/component_storage/mod.rs");
    assert!(
        mod_source.contains("pub use location::ComponentStorageLocation;"),
        "component_storage/mod.rs should keep ComponentStorageLocation as a public re-export"
    );
    assert!(
        mod_source.contains("pub use store::ComponentStorage;"),
        "component_storage/mod.rs should keep ComponentStorage as a public re-export"
    );

    for forbidden_reexport in [
        "pub(super) use entry::",
        "pub(super) use sparse::",
        "pub(super) use table::",
        "pub(super) use component_results::",
        "pub(in crate::scene::ecs::storage) use entry::",
        "pub(in crate::scene::ecs::storage) use sparse::",
        "pub(in crate::scene::ecs::storage) use table::",
        "pub(in crate::scene::ecs::storage) use component_results::",
    ] {
        assert!(
            !mod_source.contains(forbidden_reexport),
            "component_storage/mod.rs should not return to parent private re-export hub form: `{forbidden_reexport}`"
        );
    }

    let sparse_source = include_str!("../../../scene/ecs/storage/component_storage/sparse.rs");
    let table_source = include_str!("../../../scene/ecs/storage/component_storage/table.rs");
    let component_results_source =
        include_str!("../../../scene/ecs/storage/component_storage/component_results.rs");
    let store_source = include_str!("../../../scene/ecs/storage/component_storage/store.rs");

    assert!(
        sparse_source.contains("use super::entry::{RawRemoveResult, StoredComponent};"),
        "sparse.rs should import erased entry/remove-result owners directly"
    );
    assert!(
        table_source.contains("use super::entry::{RawRemoveResult, StoredComponent};"),
        "table.rs should import erased entry/remove-result owners directly"
    );
    assert!(
        component_results_source.contains("use super::entry::StoredComponent;"),
        "component_results.rs should import StoredComponent from the entry owner directly"
    );

    for required_import in [
        "use super::location::ComponentStorageLocation;",
        "use super::sparse::SparseComponentStorage;",
        "use super::table::TableComponentStorage;",
        "use super::component_results::{downcast_component, sort_component_ids_if_needed};",
    ] {
        assert!(
            store_source.contains(required_import),
            "store.rs should retain sibling owner import `{required_import}`"
        );
    }

    for (file_name, source, forbidden_import) in [
        (
            "sparse.rs",
            sparse_source,
            "use super::{RawRemoveResult, StoredComponent};",
        ),
        (
            "table.rs",
            table_source,
            "use super::{RawRemoveResult, StoredComponent};",
        ),
        (
            "component_results.rs",
            component_results_source,
            "use super::StoredComponent;",
        ),
        (
            "store.rs",
            store_source,
            "use super::{downcast_component, sort_component_ids_if_needed};",
        ),
    ] {
        assert!(
            !source.contains(forbidden_import),
            "{file_name} should not depend on parent private re-export `{forbidden_import}`"
        );
    }
}
