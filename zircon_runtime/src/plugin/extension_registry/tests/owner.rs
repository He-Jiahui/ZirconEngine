use std::sync::Arc;

use super::PluginModuleInterner;

#[test]
fn interner_indexes_and_clones_share_name_storage() {
    let mut interner = PluginModuleInterner::default();
    let id = interner
        .intern("weather.runtime")
        .expect("valid module name should intern");
    let (indexed_name, indexed_id) = interner
        .ids_by_name
        .get_key_value("weather.runtime")
        .expect("interned module should be indexed");

    assert_eq!(*indexed_id, id);
    assert!(Arc::ptr_eq(&interner.names[id.index()], indexed_name));

    let cloned = interner.clone();
    let (cloned_indexed_name, _) = cloned
        .ids_by_name
        .get_key_value("weather.runtime")
        .expect("cloned interner should preserve the index");
    assert!(Arc::ptr_eq(
        &interner.names[id.index()],
        &cloned.names[id.index()],
    ));
    assert!(Arc::ptr_eq(&cloned.names[id.index()], cloned_indexed_name,));
    assert_eq!(cloned.name(id), Some("weather.runtime"));
}
