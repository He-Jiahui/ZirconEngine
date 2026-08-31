use std::mem;
use std::sync::Mutex;

use super::{
    MaterializedPlayScene, PlayBackend, PlayBackendPoll, PlayBackendRetireReport,
    PlayBackendStartFailure, PlayBackendStartReport, PlayBackendStopReport, PlaySnapshotStore,
    PlayStartRequest,
};

mod session_contract;

pub use session_contract::{
    PlaySessionFactory, PlaySessionLaunchRequest, PlaySessionLease, PlaySessionRetireReport,
    SharedPlaySessionFactory,
};

enum ActiveEmbeddedPlaySession {
    Idle,
    Starting,
    Running {
        lease: Box<dyn PlaySessionLease>,
        scene: MaterializedPlayScene,
    },
    Stopped {
        lease: Box<dyn PlaySessionLease>,
        scene: MaterializedPlayScene,
    },
    CleanupPending {
        scene: MaterializedPlayScene,
    },
    Retiring,
}

pub struct EmbeddedPlayBackend {
    factory: SharedPlaySessionFactory,
    snapshots: PlaySnapshotStore,
    active: Mutex<ActiveEmbeddedPlaySession>,
}

impl EmbeddedPlayBackend {
    pub fn new(factory: SharedPlaySessionFactory) -> Self {
        Self {
            factory,
            snapshots: PlaySnapshotStore::default(),
            active: Mutex::new(ActiveEmbeddedPlaySession::Idle),
        }
    }

    fn restore_idle(&self) {
        *self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = ActiveEmbeddedPlaySession::Idle;
    }
}

impl PlayBackend for EmbeddedPlayBackend {
    fn start(
        &self,
        request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        {
            let mut active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match &mut *active {
                ActiveEmbeddedPlaySession::Idle => {
                    *active = ActiveEmbeddedPlaySession::Starting;
                }
                ActiveEmbeddedPlaySession::Starting => {
                    return Err(PlayBackendStartFailure::new(
                        "embedded play session is already starting",
                    ));
                }
                ActiveEmbeddedPlaySession::Running { .. } => {
                    return Err(PlayBackendStartFailure::new(
                        "embedded play session is already running",
                    ));
                }
                ActiveEmbeddedPlaySession::Stopped { .. } => {
                    return Err(PlayBackendStartFailure::retirement_pending(
                        "embedded play session must retire before another session starts"
                            .to_string(),
                    ));
                }
                ActiveEmbeddedPlaySession::CleanupPending { scene } => {
                    scene.cleanup().map_err(|error| {
                        PlayBackendStartFailure::retirement_pending(format!(
                            "embedded play snapshot cleanup is pending before another session starts: {error}"
                        ))
                    })?;
                    *active = ActiveEmbeddedPlaySession::Starting;
                }
                ActiveEmbeddedPlaySession::Retiring => {
                    return Err(PlayBackendStartFailure::retirement_pending(
                        "embedded play session is retiring",
                    ));
                }
            }
        }

        let result = (|| {
            let project_root = request.project_root.as_deref().ok_or_else(|| {
                PlayBackendStartFailure::new("embedded play requires an open project root")
            })?;
            let source = request.scene_source.as_ref().ok_or_else(|| {
                PlayBackendStartFailure::new(
                    "embedded play requires a persisted scene or world snapshot",
                )
            })?;
            let mut scene = match self.snapshots.materialize(project_root, source) {
                Ok(scene) => scene,
                Err(failure) => {
                    let (cleanup_owner, message) = failure.into_parts();
                    if let Some(scene) = cleanup_owner {
                        *self
                            .active
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                            ActiveEmbeddedPlaySession::CleanupPending { scene };
                        return Err(PlayBackendStartFailure::retirement_pending(message));
                    }
                    return Err(PlayBackendStartFailure::new(message));
                }
            };
            let launch = PlaySessionLaunchRequest::new(project_root, scene.relative_path().clone());
            let lease = match self.factory.create(&launch) {
                Ok(lease) => lease,
                Err(error) => {
                    if let Err(cleanup_error) = scene.cleanup() {
                        *self
                            .active
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                            ActiveEmbeddedPlaySession::CleanupPending { scene };
                        return Err(PlayBackendStartFailure::retirement_pending(format!(
                            "{error}; embedded snapshot cleanup remains pending: {cleanup_error}"
                        )));
                    }
                    return Err(PlayBackendStartFailure::new(error));
                }
            };
            let gateway = lease.gateway();
            let session = gateway.session_identity();
            let mut active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !matches!(&*active, ActiveEmbeddedPlaySession::Starting) {
                *active = ActiveEmbeddedPlaySession::Stopped { lease, scene };
                return Err(PlayBackendStartFailure::retirement_pending(
                    "embedded play startup ownership changed unexpectedly",
                ));
            }
            *active = ActiveEmbeddedPlaySession::Running { lease, scene };
            Ok(PlayBackendStartReport::with_gateway(
                vec![format!(
                    "embedded.session={} scene={}",
                    session.runtime_session().raw(),
                    launch.scene().as_str()
                )],
                gateway,
            ))
        })();

        if result.is_err() {
            let mut active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if matches!(&*active, ActiveEmbeddedPlaySession::Starting) {
                *active = ActiveEmbeddedPlaySession::Idle;
            }
        }
        result
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = mem::replace(&mut *active, ActiveEmbeddedPlaySession::Retiring);
        match previous {
            ActiveEmbeddedPlaySession::Running { lease, scene } => {
                *active = ActiveEmbeddedPlaySession::Stopped { lease, scene };
                Ok(PlayBackendStopReport {
                    diagnostics: vec!["embedded.session=stopped".to_string()],
                    retirement_pending: true,
                })
            }
            ActiveEmbeddedPlaySession::Idle => {
                *active = ActiveEmbeddedPlaySession::Idle;
                Ok(PlayBackendStopReport::default())
            }
            ActiveEmbeddedPlaySession::Stopped { lease, scene } => {
                *active = ActiveEmbeddedPlaySession::Stopped { lease, scene };
                Ok(PlayBackendStopReport {
                    diagnostics: Vec::new(),
                    retirement_pending: true,
                })
            }
            ActiveEmbeddedPlaySession::CleanupPending { scene } => {
                *active = ActiveEmbeddedPlaySession::CleanupPending { scene };
                Err("embedded play snapshot cleanup is still pending".to_string())
            }
            ActiveEmbeddedPlaySession::Starting => {
                *active = ActiveEmbeddedPlaySession::Starting;
                Err("embedded play session is still starting".to_string())
            }
            ActiveEmbeddedPlaySession::Retiring => {
                *active = ActiveEmbeddedPlaySession::Retiring;
                Err("embedded play session retirement is already in progress".to_string())
            }
        }
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        let (mut lease, mut scene) = {
            let mut active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let previous = mem::replace(&mut *active, ActiveEmbeddedPlaySession::Retiring);
            match previous {
                ActiveEmbeddedPlaySession::Stopped { lease, scene } => (lease, scene),
                ActiveEmbeddedPlaySession::CleanupPending { mut scene } => match scene.cleanup() {
                    Ok(()) => {
                        *active = ActiveEmbeddedPlaySession::Idle;
                        return Ok(PlayBackendRetireReport::default());
                    }
                    Err(error) => {
                        *active = ActiveEmbeddedPlaySession::CleanupPending { scene };
                        return Err(error);
                    }
                },
                ActiveEmbeddedPlaySession::Idle => {
                    *active = ActiveEmbeddedPlaySession::Idle;
                    return Ok(PlayBackendRetireReport::default());
                }
                ActiveEmbeddedPlaySession::Running { lease, scene } => {
                    *active = ActiveEmbeddedPlaySession::Running { lease, scene };
                    return Err("embedded play session must stop before retirement".to_string());
                }
                ActiveEmbeddedPlaySession::Starting => {
                    *active = ActiveEmbeddedPlaySession::Starting;
                    return Err("embedded play session is still starting".to_string());
                }
                ActiveEmbeddedPlaySession::Retiring => {
                    *active = ActiveEmbeddedPlaySession::Retiring;
                    return Err(
                        "embedded play session retirement is already in progress".to_string()
                    );
                }
            }
        };

        match lease.retire() {
            Ok(report) => {
                if let Err(error) = scene.cleanup() {
                    let mut active = self
                        .active
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    *active = ActiveEmbeddedPlaySession::CleanupPending { scene };
                    return Err(format!("embedded.snapshot_cleanup={error}"));
                }
                self.restore_idle();
                Ok(PlayBackendRetireReport {
                    diagnostics: report.diagnostics,
                })
            }
            Err(error) => {
                let mut active = self
                    .active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *active = ActiveEmbeddedPlaySession::Stopped { lease, scene };
                Err(error)
            }
        }
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*active {
            ActiveEmbeddedPlaySession::Running { .. } => Ok(PlayBackendPoll::Running {
                diagnostics: Vec::new(),
            }),
            ActiveEmbeddedPlaySession::Idle => {
                Err("embedded play session is not active while Play is running".to_string())
            }
            ActiveEmbeddedPlaySession::Starting => {
                Err("embedded play session is still starting".to_string())
            }
            ActiveEmbeddedPlaySession::Stopped { .. } => {
                Err("embedded play session is stopped and awaiting retirement".to_string())
            }
            ActiveEmbeddedPlaySession::CleanupPending { .. } => {
                Err("embedded play snapshot cleanup is pending".to_string())
            }
            ActiveEmbeddedPlaySession::Retiring => {
                Err("embedded play session is retiring".to_string())
            }
        }
    }
}
