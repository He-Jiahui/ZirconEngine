use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Barrier;

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
    assert!(!store.store_value_if_changed("after", Value::from(2)));
    assert!(store.store_value_if_changed("atomic_after", Value::from(3)));
    assert_eq!(store.load_value("atomic_after"), Some(Value::from(3)));
    assert_eq!(store.load_value("before"), Some(Value::from(1)));
    assert_eq!(store.load::<u64>("after").unwrap(), 2);

    let snapshot = store.snapshot_values();
    assert_eq!(snapshot.get("before"), Some(&Value::from(1)));
    assert_eq!(snapshot.get("after"), Some(&Value::from(2)));
    assert_eq!(snapshot.get("atomic_after"), Some(&Value::from(3)));
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

#[test]
fn config_store_set_if_changed_keeps_the_unchanged_shared_owner() {
    let store = ConfigStore::default();
    let initial = Value::String("x".repeat(64 * 1024));
    let initial_allocation = initial.as_str().unwrap().as_ptr();
    store.store_value("large", initial);
    let before = store.shared_value("large").unwrap();

    assert!(!store.store_value_if_changed("large", Value::String("x".repeat(64 * 1024))));
    let after = store.shared_value("large").unwrap();

    assert!(Arc::ptr_eq(&before, &after));
    assert_eq!(after.as_str().unwrap().as_ptr(), initial_allocation);
}

#[test]
fn config_store_set_if_changed_replaces_owned_values_without_changing_snapshots() {
    let store = ConfigStore::default();
    assert!(store.store_value_if_changed("null", Value::Null));
    assert!(!store.store_value_if_changed("null", Value::Null));
    assert!(store.store_value_if_changed("retained", Value::String("old".repeat(32 * 1024))));
    let previous = store.shared_value("retained").unwrap();
    let previous_weak = Arc::downgrade(&previous);
    let mut snapshot = store.snapshot_values();
    let previous_load = store.load_value("retained").unwrap();
    let replacement = Value::String("new".repeat(32 * 1024));
    let replacement_allocation = replacement.as_str().unwrap().as_ptr();

    assert!(store.store_value_if_changed("retained", replacement));
    let current = store.shared_value("retained").unwrap();

    assert!(!Arc::ptr_eq(&previous, &current));
    assert_eq!(current.as_str().unwrap().as_ptr(), replacement_allocation);
    assert_eq!(previous.as_str(), Some("old".repeat(32 * 1024).as_str()));
    assert_eq!(&previous_load, previous.as_ref());
    assert_eq!(snapshot.get("retained"), Some(previous.as_ref()));

    let mut loaded = store.load_value("retained").unwrap();
    let Value::String(text) = &mut loaded else {
        panic!("expected an owned JSON string");
    };
    text.push_str("-local");
    snapshot.insert("retained".to_owned(), Value::Bool(false));
    assert_eq!(
        store.load_value("retained").as_ref(),
        Some(current.as_ref())
    );
    assert_ne!(loaded, *current);

    drop(previous);
    assert!(previous_weak.upgrade().is_none());
    let current_weak = Arc::downgrade(&current);
    drop(current);
    drop(store);
    assert!(current_weak.upgrade().is_none());
}

#[test]
fn config_store_concurrent_equal_writers_publish_once_per_value() {
    const THREAD_COUNT: usize = 8;

    let store = ConfigStore::default();
    let mut previous: Option<Arc<Value>> = None;
    for character in ['a', 'b'] {
        let barrier = Arc::new(Barrier::new(THREAD_COUNT + 1));
        let threads = (0..THREAD_COUNT)
            .map(|_| {
                let store = store.clone();
                let barrier = Arc::clone(&barrier);
                let value = Value::String(character.to_string().repeat(64 * 1024));
                std::thread::spawn(move || {
                    barrier.wait();
                    let changed = store.store_value_if_changed("concurrent", value);
                    (changed, store.shared_value("concurrent").unwrap())
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        let results = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>();
        let current = store.shared_value("concurrent").unwrap();

        assert_eq!(results.iter().filter(|(changed, _)| *changed).count(), 1);
        assert_eq!(
            current.as_str(),
            Some(character.to_string().repeat(64 * 1024).as_str())
        );
        for (_, observed) in results {
            assert!(Arc::ptr_eq(&current, &observed));
        }
        if let Some(previous) = previous.replace(Arc::clone(&current)) {
            assert!(!Arc::ptr_eq(&previous, &current));
        }
    }
}

#[test]
fn config_store_public_store_converts_reentrant_keys_before_locking() {
    struct ReentrantKey<'a>(&'a ConfigStore);

    impl<'a> From<ReentrantKey<'a>> for String {
        fn from(key: ReentrantKey<'a>) -> Self {
            // Fail before public reentry on the old implementation instead of blocking the test.
            let guard = key
                .0
                .values
                .try_lock()
                .expect("public key conversion must run without the config store lock");
            drop(guard);
            assert_eq!(
                key.0.load_value("target"),
                Some(Value::String("old".to_owned()))
            );
            key.0.store_value("nested", Value::from(7_u64));
            assert_eq!(key.0.load::<u64>("nested").unwrap(), 7_u64);
            "target".to_owned()
        }
    }

    let store = ConfigStore::default();
    store.store_value("target", Value::String("old".to_owned()));
    let old_owner = Arc::downgrade(&store.shared_value("target").unwrap());
    let snapshot = store.snapshot_values();

    store.store_value(ReentrantKey(&store), Value::String("new".to_owned()));

    assert_eq!(
        store.load_value("target"),
        Some(Value::String("new".to_owned()))
    );
    assert_eq!(store.load_value("nested"), Some(Value::from(7_u64)));
    assert_eq!(
        snapshot.get("target"),
        Some(&Value::String("old".to_owned()))
    );
    assert!(!snapshot.contains_key("nested"));
    assert!(old_owner.upgrade().is_none());
}
