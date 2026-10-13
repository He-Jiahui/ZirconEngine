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
fn config_store_changed_comparison_rechecks_a_concurrent_raw_replacement() {
    use std::cell::{Cell, RefCell};

    for concurrent_value in [1_u64, 2] {
        let store = ConfigStore::default();
        store.store_value("target", Value::from(0_u64));
        let previous = store.shared_value("target").unwrap();
        let compared_values = RefCell::new(Vec::new());
        let raw_owner = RefCell::new(None);
        let replaced = Cell::new(false);

        let changed =
            store.store_value_if_changed_by("target", Value::from(1_u64), |stored, incoming| {
                let guard = store
                    .values
                    .try_lock()
                    .expect("deep JSON comparison must not hold the config map mutex");
                drop(guard);
                compared_values.borrow_mut().push(stored.clone());
                if !replaced.replace(true) {
                    // Model a raw Core write between owner observation and publication.
                    store.store_value("target", Value::from(concurrent_value));
                    *raw_owner.borrow_mut() = store.shared_value("target");
                    assert_eq!(store.load::<u64>("target").unwrap(), concurrent_value);
                    assert_eq!(
                        store.snapshot_values().get("target"),
                        Some(&Value::from(concurrent_value))
                    );
                }
                stored == incoming
            });

        assert_eq!(changed, concurrent_value != 1);
        assert_eq!(
            compared_values.into_inner(),
            vec![Value::from(0_u64), Value::from(concurrent_value)]
        );
        let current = store.shared_value("target").unwrap();
        assert_eq!(current.as_ref(), &Value::from(1_u64));
        assert_eq!(previous.as_ref(), &Value::from(0_u64));
        assert_eq!(
            Arc::ptr_eq(&current, raw_owner.borrow().as_ref().unwrap()),
            concurrent_value == 1,
            "an equal raw replacement keeps its shared owner"
        );
    }
}

#[test]
fn config_store_equal_comparison_allows_reads_without_replacing_the_owner() {
    let store = ConfigStore::default();
    let original = Value::String("x".repeat(1024 * 1024));
    store.store_value("large", original.clone());
    let owner = store.shared_value("large").unwrap();
    let comparisons = std::cell::Cell::new(0);

    let changed = store.store_value_if_changed_by("large", original, |stored, incoming| {
        let guard = store
            .values
            .try_lock()
            .expect("equal JSON comparison must not hold the config map mutex");
        drop(guard);
        comparisons.set(comparisons.get() + 1);
        store.store_value("unrelated", Value::from(7_u64));
        assert_eq!(store.load::<u64>("unrelated").unwrap(), 7);
        assert_eq!(store.snapshot_values().get("large"), Some(stored));
        stored == incoming
    });

    assert!(!changed);
    assert_eq!(comparisons.get(), 1);
    assert!(Arc::ptr_eq(&owner, &store.shared_value("large").unwrap()));
}

#[test]
fn config_store_equal_comparison_rechecks_a_concurrent_raw_change() {
    use std::cell::{Cell, RefCell};

    for concurrent_value in [1_u64, 2] {
        let store = ConfigStore::default();
        store.store_value("target", Value::from(1_u64));
        let previous = store.shared_value("target").unwrap();
        let compared_values = RefCell::new(Vec::new());
        let raw_owner = RefCell::new(None);
        let replaced = Cell::new(false);

        let changed =
            store.store_value_if_changed_by("target", Value::from(1_u64), |stored, incoming| {
                let guard = store
                    .values
                    .try_lock()
                    .expect("equal JSON comparison must not hold the config map mutex");
                drop(guard);
                compared_values.borrow_mut().push(stored.clone());
                if !replaced.replace(true) {
                    store.store_value("target", Value::from(concurrent_value));
                    *raw_owner.borrow_mut() = store.shared_value("target");
                }
                stored == incoming
            });

        assert_eq!(changed, concurrent_value != 1);
        assert_eq!(
            compared_values.into_inner(),
            vec![Value::from(1_u64), Value::from(concurrent_value)]
        );
        let current = store.shared_value("target").unwrap();
        assert_eq!(current.as_ref(), &Value::from(1_u64));
        assert_eq!(previous.as_ref(), &Value::from(1_u64));
        assert_eq!(
            Arc::ptr_eq(&current, raw_owner.borrow().as_ref().unwrap()),
            concurrent_value == 1
        );
    }
}

#[test]
#[ignore = "Release Runtime03 real 1B/1MiB/100MiB equal/change store probe; bounded Jenkins execution"]
fn runtime03_config_store_equal_and_changed_large_value_samples() {
    use std::time::Instant;

    assert!(!cfg!(debug_assertions), "Release execution is required");
    for payload_bytes in [1_usize, 1_048_576, 104_857_600] {
        let store = ConfigStore::default();
        let initial = Value::String("x".repeat(payload_bytes));
        let mut changed = "x".repeat(payload_bytes - 1);
        changed.push('y');
        let changed = Value::String(changed);
        for sample_index in 0..5 {
            store.store_value("large", initial.clone());
            let before = store.shared_value("large").unwrap();
            let equal_input = initial.clone();
            let started = Instant::now();
            let equal_changed = store.store_value_if_changed("large", equal_input);
            let equal_ns = started.elapsed().as_nanos();
            assert!(!equal_changed);
            assert!(Arc::ptr_eq(&before, &store.shared_value("large").unwrap()));
            drop(before);

            let changed_input = changed.clone();
            let started = Instant::now();
            let did_change = store.store_value_if_changed("large", changed_input);
            let changed_ns = started.elapsed().as_nanos();
            assert!(did_change);
            let stored = store.shared_value("large").unwrap();
            assert_eq!(stored.as_ref(), &changed);
            assert_eq!(stored.as_str().unwrap().len(), payload_bytes);
            println!(
                "RUNTIME03_CONFIG_STORE_EQUAL_CHANGE_SAMPLE_V1 payload_bytes={payload_bytes} sample_index={sample_index} equal_ns={equal_ns} changed_ns={changed_ns} changed_byte=last caller_input_allocation=outside_window old_value_retirement=inside_changed_window numeric_performance_budget=unspecified"
            );
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
