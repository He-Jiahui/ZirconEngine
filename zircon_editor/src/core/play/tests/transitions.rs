use super::*;
use std::sync::mpsc;
use std::time::Duration;

struct BlockingBackend {
    entered: mpsc::Sender<&'static str>,
    release: Mutex<mpsc::Receiver<()>>,
    fail_start: bool,
    fail_stop: bool,
}

impl PlayBackend for BlockingBackend {
    fn start(
        &self,
        _: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        self.entered.send("start").unwrap();
        self.release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        if self.fail_start {
            Err(PlayBackendStartFailure::new("start failed"))
        } else {
            Ok(PlayBackendStartReport::default())
        }
    }
    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        self.entered.send("stop").unwrap();
        self.release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        if self.fail_stop {
            Err("stop failed".into())
        } else {
            Ok(PlayBackendStopReport::default())
        }
    }
    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        Ok(PlayBackendRetireReport::default())
    }
    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}

#[test]
fn starting_and_stopping_are_observable_without_waiting_for_the_backend() {
    let controller = Arc::new(PlaySessionController::new());
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    controller.set_play_backend(Arc::new(BlockingBackend {
        entered: entered_tx,
        release: Mutex::new(release_rx),
        fail_start: false,
        fail_stop: false,
    }));
    let owner = controller.clone();
    let start = std::thread::spawn(move || {
        owner.request_play(PlayStartRequest::immediate(PlayKind::Play, None))
    });
    assert_eq!(
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        "start"
    );
    assert_eq!(controller.mode(), PlayModeKind::Starting);
    assert!(!controller.mode().has_active_runtime());
    release_tx.send(()).unwrap();
    assert_eq!(start.join().unwrap().unwrap().mode, PlayModeKind::Playing);
    let owner = controller.clone();
    let stop = std::thread::spawn(move || owner.request_stop());
    assert_eq!(
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        "stop"
    );
    assert_eq!(controller.mode(), PlayModeKind::Stopping);
    assert!(!controller.mode().has_active_runtime());
    release_tx.send(()).unwrap();
    assert_eq!(stop.join().unwrap().unwrap().mode, PlayModeKind::Edit);
}

#[test]
fn a_failed_start_restores_edit_and_a_failed_stop_retains_playing() {
    for fail_start in [true, false] {
        let controller = Arc::new(PlaySessionController::new());
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        controller.set_play_backend(Arc::new(BlockingBackend {
            entered: entered_tx,
            release: Mutex::new(release_rx),
            fail_start,
            fail_stop: !fail_start,
        }));
        let owner = controller.clone();
        let start = std::thread::spawn(move || {
            owner.request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        });
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        release_tx.send(()).unwrap();
        if fail_start {
            assert!(start.join().unwrap().is_err());
            assert_eq!(controller.mode(), PlayModeKind::Edit);
        } else {
            start.join().unwrap().unwrap();
            let owner = controller.clone();
            let stop = std::thread::spawn(move || owner.request_stop());
            entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            release_tx.send(()).unwrap();
            assert!(stop.join().unwrap().is_err());
            assert_eq!(controller.mode(), PlayModeKind::Playing);
        }
    }
}
