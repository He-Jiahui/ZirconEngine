use std::cell::RefCell;
use std::rc::Rc;

use super::PreviewSubject;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewPlayback {
    pub playing: bool,
    pub looping: bool,
    pub rate: f32,
    pub time_seconds: f32,
}

impl PreviewPlayback {
    pub const fn new(playing: bool, looping: bool, rate: f32, time_seconds: f32) -> Self {
        Self {
            playing,
            looping,
            rate,
            time_seconds,
        }
    }

    pub const fn paused(time_seconds: f32) -> Self {
        Self::new(false, false, 1.0, time_seconds)
    }

    pub const fn playing(rate: f32, looping: bool, time_seconds: f32) -> Self {
        Self::new(true, looping, rate, time_seconds)
    }

    fn is_valid(self) -> bool {
        self.rate.is_finite() && self.time_seconds.is_finite()
    }
}

pub trait PreviewSceneBackend {
    type SessionId: Clone;
    type Error: std::error::Error + Send + Sync + 'static;

    fn create_secondary_session(&mut self) -> Result<Self::SessionId, Self::Error>;
    fn destroy_secondary_session(&mut self, session: &Self::SessionId) -> Result<(), Self::Error>;
    fn set_subject(
        &mut self,
        session: &Self::SessionId,
        subject: Option<&PreviewSubject>,
    ) -> Result<(), Self::Error>;
    fn set_playback(
        &mut self,
        session: &Self::SessionId,
        playback: PreviewPlayback,
    ) -> Result<(), Self::Error>;
    fn invalidate_views(&mut self, session: &Self::SessionId) -> Result<(), Self::Error>;
    fn focus_views(&mut self, session: &Self::SessionId) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum PreviewSceneError<BackendError> {
    Closed,
    InvalidPlayback,
    Backend(BackendError),
}

impl<BackendError> std::fmt::Display for PreviewSceneError<BackendError>
where
    BackendError: std::fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => formatter.write_str("preview scene is closed"),
            Self::InvalidPlayback => formatter.write_str("preview playback is invalid"),
            Self::Backend(error) => write!(formatter, "preview scene backend: {error}"),
        }
    }
}

impl<BackendError> std::error::Error for PreviewSceneError<BackendError>
where
    BackendError: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Backend(error) => Some(error),
            Self::Closed | Self::InvalidPlayback => None,
        }
    }
}

/// One runtime-backed preview world shared by an animation-family toolkit group.
pub struct PreviewScene<Backend>
where
    Backend: PreviewSceneBackend,
{
    backend: Backend,
    session: Option<Backend::SessionId>,
    subject: Option<PreviewSubject>,
    playback: PreviewPlayback,
}

pub type SharedPreviewScene<Backend> = Rc<RefCell<PreviewScene<Backend>>>;

impl<Backend> PreviewScene<Backend>
where
    Backend: PreviewSceneBackend,
{
    pub fn open(mut backend: Backend) -> Result<Self, PreviewSceneError<Backend::Error>> {
        let session = backend
            .create_secondary_session()
            .map_err(PreviewSceneError::Backend)?;
        Ok(Self {
            backend,
            session: Some(session),
            subject: None,
            playback: PreviewPlayback::paused(0.0),
        })
    }

    pub fn shared(self) -> SharedPreviewScene<Backend> {
        Rc::new(RefCell::new(self))
    }

    pub fn session_id(&self) -> Option<&Backend::SessionId> {
        self.session.as_ref()
    }

    pub fn subject(&self) -> Option<&PreviewSubject> {
        self.subject.as_ref()
    }

    pub fn playback(&self) -> PreviewPlayback {
        self.playback
    }

    pub fn set_subject(
        &mut self,
        subject: Option<PreviewSubject>,
    ) -> Result<bool, PreviewSceneError<Backend::Error>> {
        if self.subject == subject {
            return Ok(false);
        }
        let session = self.active_session()?;
        self.backend
            .set_subject(&session, subject.as_ref())
            .map_err(PreviewSceneError::Backend)?;
        self.subject = subject;
        Ok(true)
    }

    pub fn set_playback(
        &mut self,
        playback: PreviewPlayback,
    ) -> Result<bool, PreviewSceneError<Backend::Error>> {
        if !playback.is_valid() {
            return Err(PreviewSceneError::InvalidPlayback);
        }
        if self.playback == playback {
            return Ok(false);
        }
        let session = self.active_session()?;
        self.backend
            .set_playback(&session, playback)
            .map_err(PreviewSceneError::Backend)?;
        self.playback = playback;
        Ok(true)
    }

    pub fn invalidate_views(&mut self) -> Result<(), PreviewSceneError<Backend::Error>> {
        let session = self.active_session()?;
        self.backend
            .invalidate_views(&session)
            .map_err(PreviewSceneError::Backend)
    }

    pub fn focus_views(&mut self) -> Result<(), PreviewSceneError<Backend::Error>> {
        let session = self.active_session()?;
        self.backend
            .focus_views(&session)
            .map_err(PreviewSceneError::Backend)
    }

    /// Closes once. If backend destruction fails, the session is retained so the caller can retry
    /// rather than losing the only handle to a possibly-live runtime world.
    pub fn close(&mut self) -> Result<bool, PreviewSceneError<Backend::Error>> {
        let Some(session) = self.session.take() else {
            return Ok(false);
        };
        if let Err(error) = self.backend.destroy_secondary_session(&session) {
            self.session = Some(session);
            return Err(PreviewSceneError::Backend(error));
        }
        self.subject = None;
        Ok(true)
    }

    fn active_session(&self) -> Result<Backend::SessionId, PreviewSceneError<Backend::Error>> {
        self.session.clone().ok_or(PreviewSceneError::Closed)
    }
}

impl<Backend> Drop for PreviewScene<Backend>
where
    Backend: PreviewSceneBackend,
{
    fn drop(&mut self) {
        let _ = self.close();
    }
}
