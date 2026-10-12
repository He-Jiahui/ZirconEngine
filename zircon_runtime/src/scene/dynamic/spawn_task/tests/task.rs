use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;

#[test]
fn dynamic_scene_spawn_task_accessors_recover_poisoned_locks() {
    let result = Arc::new(Mutex::new(Some(Err(DynamicSceneError::Parse {
        reason: "decode failed".to_string(),
    }))));

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = result.lock().unwrap();
        panic!("poison dynamic scene spawn task result lock");
    }));

    let recovered = lock_spawn_result(&result)
        .take()
        .expect("result should remain available after poison recovery");
    assert!(matches!(recovered, Err(DynamicSceneError::Parse { .. })));
}
