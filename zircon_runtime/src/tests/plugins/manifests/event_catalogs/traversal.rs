use std::path::Path;

use super::non_empty_string_value;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_event_catalogs(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    let Some(catalogs) = event_catalog_array(table, relative_path) else {
        return;
    };

    for catalog in catalogs {
        let catalog = catalog.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} event catalog should be a table")
        });
        let namespace =
            non_empty_string_value(catalog, relative_path, "event catalog", "namespace");
        let catalog_context = format!("event catalog `{namespace}`");
        visit(catalog, &catalog_context);
    }
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn event_catalog_array<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
) -> Option<&'a Vec<toml::Value>> {
    let Some(catalogs) = table.get("event_catalogs") else {
        return None;
    };
    Some(catalogs.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} event_catalogs should be an array")
    }))
}

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn visit_event_rows(
    catalog: &toml::Table,
    relative_path: &Path,
    catalog_context: &str,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    let Some(events) = catalog.get("events") else {
        return;
    };
    let events = events.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} {catalog_context} events should be an array")
    });
    for event in events {
        let event = event.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} {catalog_context} event should be a table")
        });
        let event_id = non_empty_string_value(event, relative_path, catalog_context, "id");
        let event_context = format!("{catalog_context} event `{event_id}`");
        visit(event, &event_context);
    }
}
