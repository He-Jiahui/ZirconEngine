use std::collections::HashMap;

use super::*;

#[test]
fn poisoned_mutex_still_returns_guard_for_text_cache_paths() {
    let cache = Mutex::new(HashMap::from([(1_u8, 2_u8)]));

    let poison_result = std::panic::catch_unwind(|| {
        let mut guard = cache.lock().expect("test text cache lock");
        guard.insert(3, 4);
        panic!("poison text cache");
    });

    assert!(poison_result.is_err());
    let guard = lock_recovering_poison(&cache);
    assert_eq!(guard.get(&1), Some(&2));
    assert_eq!(guard.get(&3), Some(&4));
}
