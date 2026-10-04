#[cfg(test)]
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
#[cfg(test)]
use std::sync::{MutexGuard, TryLockError};

type SessionOwner = Arc<Mutex<()>>;
type SessionOwnerRegistry = Mutex<HashMap<PathBuf, Weak<Mutex<()>>>>;

static SESSION_OWNERS: OnceLock<SessionOwnerRegistry> = OnceLock::new();

#[cfg(test)]
thread_local! {
    static AFTER_OWNER_WOULD_BLOCK_HOOK: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);
}

#[cfg(test)]
pub(super) fn after_next_owner_would_block(hook: impl FnOnce() + 'static) {
    AFTER_OWNER_WOULD_BLOCK_HOOK.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(hook));
    });
}

// 规范化目录用作进程内互斥键；此注册表不协调其他进程对同一目录的写入。
pub(super) fn with_session_owner<T>(directory: &Path, action: impl FnOnce() -> T) -> io::Result<T> {
    let identity = fs::canonicalize(directory)?;
    let owner = owner_for(identity);
    #[cfg(test)]
    let _guard = lock_owner_for_test(&owner);
    #[cfg(not(test))]
    let _guard = owner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Ok(action())
}

#[cfg(test)]
fn lock_owner_for_test(owner: &Mutex<()>) -> MutexGuard<'_, ()> {
    match owner.try_lock() {
        Ok(guard) => guard,
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(TryLockError::WouldBlock) => {
            AFTER_OWNER_WOULD_BLOCK_HOOK.with(|slot| {
                let hook = slot.borrow_mut().take();
                if let Some(hook) = hook {
                    hook();
                }
            });
            owner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        }
    }
}

fn owner_for(identity: PathBuf) -> SessionOwner {
    let registry = SESSION_OWNERS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut owners = registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Only active exports retain a strong owner. Old session keys are reclaimed
    // without holding the registry across artifact I/O.
    owners.retain(|_, owner| owner.strong_count() > 0);
    if let Some(owner) = owners.get(&identity).and_then(Weak::upgrade) {
        return owner;
    }
    let owner = Arc::new(Mutex::new(()));
    owners.insert(identity, Arc::downgrade(&owner));
    owner
}
