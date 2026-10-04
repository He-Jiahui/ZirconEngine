use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;

fn poison_values_lock(store: &ConfigStore) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = store.values.lock().unwrap();
        panic!("poison config store values lock");
    }));
    assert!(result.is_err());
}

#[test]
fn config_store_accessors_recover_poisoned_values_lock() {
    let store = ConfigStore::default();

    store.store_value("before", Value::from(1));
    poison_values_lock(&store);

    store.store_value("after", Value::from(2));
    assert_eq!(store.load_value("before"), Some(Value::from(1)));
    assert_eq!(store.load::<u64>("after").unwrap(), 2);

    let snapshot = store.snapshot_values();
    assert_eq!(snapshot.get("before"), Some(&Value::from(1)));
    assert_eq!(snapshot.get("after"), Some(&Value::from(2)));
}

#[test]
fn config_store_typed_load_borrows_shared_json_storage() {
    let source = include_str!("../config_store.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("config store implementation");

    assert!(implementation.contains("HashMap<String, Arc<Value>>"));
    assert!(implementation.contains("T::deserialize(value.as_ref())"));
    assert!(!implementation.contains(".load_value(key)"));
}
