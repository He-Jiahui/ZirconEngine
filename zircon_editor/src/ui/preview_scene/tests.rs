use std::cell::RefCell;
use std::rc::Rc;

use super::{
    PreviewPlayback, PreviewScene, PreviewSceneBackend, PreviewSceneError, PreviewSubject,
};
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct BackendEvents {
    created: usize,
    destroyed: usize,
    subject_updates: usize,
    playback_updates: usize,
    invalidations: usize,
    focus_requests: usize,
}

struct FakePreviewBackend {
    events: Rc<RefCell<BackendEvents>>,
    fail_open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FakePreviewBackendError {
    Unavailable,
}

impl fmt::Display for FakePreviewBackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("fake preview backend unavailable")
    }
}

impl std::error::Error for FakePreviewBackendError {}

impl FakePreviewBackend {
    fn new(events: Rc<RefCell<BackendEvents>>) -> Self {
        Self {
            events,
            fail_open: false,
        }
    }

    fn failing_open(events: Rc<RefCell<BackendEvents>>) -> Self {
        Self {
            events,
            fail_open: true,
        }
    }
}

impl PreviewSceneBackend for FakePreviewBackend {
    type SessionId = usize;
    type Error = FakePreviewBackendError;

    fn create_secondary_session(&mut self) -> Result<Self::SessionId, Self::Error> {
        if self.fail_open {
            return Err(FakePreviewBackendError::Unavailable);
        }
        let mut events = self.events.borrow_mut();
        events.created += 1;
        Ok(events.created)
    }

    fn destroy_secondary_session(&mut self, _session: &Self::SessionId) -> Result<(), Self::Error> {
        self.events.borrow_mut().destroyed += 1;
        Ok(())
    }

    fn set_subject(
        &mut self,
        _session: &Self::SessionId,
        _subject: Option<&PreviewSubject>,
    ) -> Result<(), Self::Error> {
        self.events.borrow_mut().subject_updates += 1;
        Ok(())
    }

    fn set_playback(
        &mut self,
        _session: &Self::SessionId,
        _playback: PreviewPlayback,
    ) -> Result<(), Self::Error> {
        self.events.borrow_mut().playback_updates += 1;
        Ok(())
    }

    fn invalidate_views(&mut self, _session: &Self::SessionId) -> Result<(), Self::Error> {
        self.events.borrow_mut().invalidations += 1;
        Ok(())
    }

    fn focus_views(&mut self, _session: &Self::SessionId) -> Result<(), Self::Error> {
        self.events.borrow_mut().focus_requests += 1;
        Ok(())
    }
}

fn subject() -> PreviewSubject {
    PreviewSubject::new("/characters/hero.mesh")
        .with_animation_asset("/animations/hero_idle.zranim")
        .with_parameter_override("speed", "1.25")
}

#[test]
fn preview_scene_forwards_subject_playback_and_view_requests_to_one_secondary_session() {
    let events = Rc::new(RefCell::new(BackendEvents::default()));
    let mut scene = PreviewScene::open(FakePreviewBackend::new(Rc::clone(&events))).unwrap();

    assert_eq!(scene.session_id(), Some(&1));
    assert!(scene.set_subject(Some(subject())).unwrap());
    assert!(scene
        .set_playback(PreviewPlayback::playing(1.0, true, 0.5))
        .unwrap());
    scene.invalidate_views().unwrap();
    scene.focus_views().unwrap();
    scene.close().unwrap();

    assert_eq!(
        *events.borrow(),
        BackendEvents {
            created: 1,
            destroyed: 1,
            subject_updates: 1,
            playback_updates: 1,
            invalidations: 1,
            focus_requests: 1,
        }
    );
}

#[test]
fn shared_preview_scene_preserves_one_scene_for_multiple_toolkits() {
    let events = Rc::new(RefCell::new(BackendEvents::default()));
    let shared = PreviewScene::open(FakePreviewBackend::new(Rc::clone(&events)))
        .unwrap()
        .shared();
    let second_toolkit = Rc::clone(&shared);

    shared.borrow_mut().set_subject(Some(subject())).unwrap();
    assert_eq!(second_toolkit.borrow().subject(), Some(&subject()));
    second_toolkit.borrow_mut().close().unwrap();

    assert_eq!(events.borrow().created, 1);
    assert_eq!(events.borrow().destroyed, 1);
}

#[test]
fn ten_preview_scene_lifecycles_destroy_every_secondary_session() {
    let events = Rc::new(RefCell::new(BackendEvents::default()));
    for _ in 0..10 {
        let mut scene = PreviewScene::open(FakePreviewBackend::new(Rc::clone(&events))).unwrap();
        scene.close().unwrap();
    }

    assert_eq!(events.borrow().created, 10);
    assert_eq!(events.borrow().destroyed, 10);
}

#[test]
fn invalid_playback_rate_is_rejected_before_it_reaches_the_runtime_backend() {
    let events = Rc::new(RefCell::new(BackendEvents::default()));
    let mut scene = PreviewScene::open(FakePreviewBackend::new(Rc::clone(&events))).unwrap();

    assert!(scene
        .set_playback(PreviewPlayback::playing(f32::NAN, false, 0.0))
        .is_err());
    assert_eq!(events.borrow().playback_updates, 0);
}

#[test]
fn preview_scene_preserves_the_backend_error_type_at_its_public_boundary() {
    let events = Rc::new(RefCell::new(BackendEvents::default()));
    let result = PreviewScene::open(FakePreviewBackend::failing_open(events));

    assert!(matches!(
        result,
        Err(PreviewSceneError::Backend(
            FakePreviewBackendError::Unavailable
        ))
    ));
}
