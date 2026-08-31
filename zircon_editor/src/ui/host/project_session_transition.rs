use std::sync::{LockResult, Mutex, MutexGuard};

/// Serializes the complete manager-owned project activation and close transactions.
///
/// Session admission, runtime effects, Ready publication, close compensation, and final lease
/// release form one lifecycle transaction. Serializing only the retained `SessionGuard` slot would
/// allow two activations, or an activation and close, to mutate the runtime concurrently.
#[derive(Debug, Default)]
pub(super) struct ProjectSessionTransitionGate {
    transition: Mutex<()>,
}

impl ProjectSessionTransitionGate {
    pub(super) fn enter(&self) -> LockResult<MutexGuard<'_, ()>> {
        self.transition.lock()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::Duration;

    use super::ProjectSessionTransitionGate;

    #[test]
    fn project_session_transition_gate_serializes_complete_lifecycle_transactions() {
        let gate = Arc::new(ProjectSessionTransitionGate::default());
        let first_transition = gate.enter().unwrap();
        let (attempted_tx, attempted_rx) = mpsc::channel();
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let contender_gate = Arc::clone(&gate);

        let contender = thread::spawn(move || {
            attempted_tx.send(()).unwrap();
            let _transition = contender_gate.enter().unwrap();
            acquired_tx.send(()).unwrap();
        });

        attempted_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(matches!(
            acquired_rx.recv_timeout(Duration::from_millis(25)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        drop(first_transition);
        acquired_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        contender.join().unwrap();
    }

    #[test]
    fn project_session_transition_panic_poison_fails_closed() {
        let gate = Arc::new(ProjectSessionTransitionGate::default());
        let panic_gate = Arc::clone(&gate);

        assert!(thread::spawn(move || {
            let _transition = panic_gate.enter().unwrap();
            panic!("project session transition fault injection");
        })
        .join()
        .is_err());

        assert!(gate.enter().is_err());
    }
}
