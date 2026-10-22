use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::EditorStartupStage;

pub(super) const CHILD_ARGUMENT: &str = "--zircon-startup-splash-child";
static ACTIVE: Mutex<Option<Weak<SplashSession>>> = Mutex::new(None);

struct SplashSession {
    input: Mutex<Option<ChildStdin>>,
    stage: Mutex<EditorStartupStage>,
    shutdown: mpsc::Sender<()>,
    started: Instant,
    timing: Arc<Mutex<SplashTiming>>,
}

#[derive(Default)]
struct SplashTiming {
    visible: Option<Instant>,
    closed: Option<Instant>,
}

impl SplashTiming {
    fn visible_duration(&self, now: Instant) -> Option<Duration> {
        self.visible.map(|visible| {
            self.closed
                .unwrap_or(now)
                .saturating_duration_since(visible)
        })
    }
}

/// Owns exactly one splash child. Drop also covers early startup failure/unwinding.
pub struct EditorStartupSplash {
    session: Arc<SplashSession>,
    reaper: Option<JoinHandle<()>>,
}

impl EditorStartupSplash {
    pub fn launch(project_requested: bool) -> Option<Self> {
        if std::env::var("ZIRCON_EDITOR_STARTUP_SPLASH").is_ok_and(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "0" | "false" | "off" | "no"
            )
        }) {
            return None;
        }
        let mut active = ACTIVE.lock().unwrap_or_else(|error| error.into_inner());
        if active.as_ref().and_then(Weak::upgrade).is_some() {
            return None;
        }
        let started = Instant::now();
        let mut child = match spawn_child(project_requested) {
            Ok(child) => child,
            Err(error) => {
                eprintln!("[zircon_editor] startup splash unavailable: {error}");
                return None;
            }
        };
        let input = child.stdin.take();
        let output = child.stdout.take();
        let timing = Arc::new(Mutex::new(SplashTiming::default()));
        let child_timing = timing.clone();
        let reader = thread::spawn(move || {
            if let Some(output) = output {
                for line in BufReader::new(output).lines().map_while(Result::ok) {
                    if line == "visible" {
                        child_timing
                            .lock()
                            .unwrap_or_else(|error| error.into_inner())
                            .visible = Some(Instant::now());
                    } else if line == "closed" {
                        let mut timing = child_timing
                            .lock()
                            .unwrap_or_else(|error| error.into_inner());
                        timing.closed = Some(Instant::now());
                        if let Some(duration) = timing.visible_duration(Instant::now()) {
                            eprintln!(
                                "[zircon_editor] startup_splash event=hidden visible_ms={:.2}",
                                duration.as_secs_f64() * 1000.0
                            );
                        }
                    }
                    if matches!(line.as_str(), "visible" | "resized" | "closed") {
                        eprintln!(
                            "[zircon_editor] startup_splash event={line} elapsed_ms={:.2}",
                            started.elapsed().as_secs_f64() * 1000.0
                        );
                    }
                }
            }
        });
        let (shutdown, receiver) = mpsc::channel();
        let reaper = thread::spawn(move || reap_child(child, receiver, reader));
        let session = Arc::new(SplashSession {
            input: Mutex::new(input),
            stage: Mutex::new(EditorStartupStage::RuntimeLoad),
            shutdown,
            started,
            timing,
        });
        *active = Some(Arc::downgrade(&session));
        eprintln!(
            "[zircon_editor] startup_splash event=launched elapsed_ms={:.2}",
            started.elapsed().as_secs_f64() * 1000.0
        );
        Some(Self {
            session,
            reaper: Some(reaper),
        })
    }

    pub fn set_stage(stage: EditorStartupStage) {
        with_active(|session| {
            let mut current = session
                .stage
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if stage <= *current || stage == EditorStartupStage::Ready {
                return;
            }
            *current = stage;
            if let Some(input) = session
                .input
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_mut()
            {
                let _ = writeln!(input, "{}", stage.wire());
                let _ = input.flush();
            }
            eprintln!(
                "[zircon_editor] startup_splash event=stage stage={} elapsed_ms={:.2}",
                stage.wire(),
                session.started.elapsed().as_secs_f64() * 1000.0
            );
        });
    }

    /// Called only by the retained host's successful first-present callback.
    pub fn first_frame_presented() {
        with_active(|session| {
            let mut stage = session
                .stage
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if *stage == EditorStartupStage::Ready {
                return;
            }
            *stage = EditorStartupStage::Ready;
            let visible_ms = session
                .timing
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .visible_duration(Instant::now())
                .map(|duration| duration.as_secs_f64() * 1000.0);
            eprintln!(
                "[zircon_editor] startup_splash event=first_frame elapsed_ms={:.2} visible_ms={}",
                session.started.elapsed().as_secs_f64() * 1000.0,
                visible_ms.map_or_else(|| "unobserved".to_owned(), |ms| format!("{ms:.2}"))
            );
            close_session(session, true);
        });
    }
}

impl Drop for EditorStartupSplash {
    fn drop(&mut self) {
        close_session(&self.session, false);
        if let Some(reaper) = self.reaper.take() {
            let _ = reaper.join();
        }
    }
}

fn spawn_child(project_requested: bool) -> std::io::Result<Child> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg(CHILD_ARGUMENT)
        .arg(if project_requested {
            "project"
        } else {
            "welcome"
        })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: the child owns its native surface.
    }
    command.spawn()
}

fn with_active(action: impl FnOnce(&SplashSession)) {
    let session = ACTIVE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .and_then(Weak::upgrade);
    if let Some(session) = session {
        action(&session);
    }
}

fn close_session(session: &SplashSession, ready: bool) {
    if let Some(mut input) = session
        .input
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take()
    {
        let _ = writeln!(input, "{}", if ready { "ready" } else { "close" });
        let _ = input.flush();
        // EOF is also the child cleanup signal when the parent exits unexpectedly.
    }
    let _ = session.shutdown.send(());
}

fn reap_child(mut child: Child, shutdown: mpsc::Receiver<()>, reader: JoinHandle<()>) {
    let mut deadline = None;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
            Ok(None) => {}
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        if shutdown.recv_timeout(Duration::from_millis(50)).is_ok() && deadline.is_none() {
            deadline = Some(Instant::now() + Duration::from_secs(2));
        }
    }
    let _ = reader.join();
}

#[cfg(test)]
#[path = "tests/process.rs"]
mod tests;
