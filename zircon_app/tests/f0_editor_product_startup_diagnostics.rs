#![cfg(feature = "target-editor-host")]

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const EDITOR_EXE: &str = env!("CARGO_BIN_EXE_zircon_editor");
const RUNTIME_LIBRARY_ENV: &str = "ZIRCON_RUNTIME_LIBRARY";
const FIRST_FRAME_CAPTURE_ENV: &str = "ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG";
const LOG_ROOT_ENV: &str = "ZIRCON_LOG_ROOT";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[test]
fn missing_runtime_override_fails_with_the_requested_artifact_and_recovery() {
    let scratch = TestScratch::new("missing_runtime");
    let runtime_library = scratch
        .root()
        .join("missing-runtime")
        .join(format!("zircon_runtime{}", std::env::consts::DLL_SUFFIX));
    let runtime_manifest = runtime_manifest_path(&runtime_library);
    assert!(!runtime_library.exists());
    assert!(!runtime_manifest.exists());

    let output = run_editor(editor_command(&scratch, &runtime_library), &scratch);
    let stderr = &output.stderr;

    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.contains(
            "editor startup diagnostic: component=runtime_build_set requested=workspace:welcome"
        ),
        "the product error must identify the failing startup component and mode: {stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "requested_path={RUNTIME_LIBRARY_ENV}={}",
            runtime_library.display()
        )),
        "the product error must retain the exact override path: {stderr}"
    );
    assert!(
        stderr.contains("cause=runtime BuildSet preflight failed")
            && stderr.contains("runtime artifact manifest")
            && stderr.contains("recovery=stage a runtime library"),
        "the product error must include the preflight cause and staging recovery: {stderr}"
    );
    assert_no_panic_diagnostics("stdout", &output.stdout);
    assert_no_panic_diagnostics("stderr", stderr);
}

#[test]
fn incompatible_startup_arguments_fail_before_runtime_preflight() {
    let scratch = TestScratch::new("argument_conflict");
    let runtime_library = scratch
        .root()
        .join("missing-runtime")
        .join(format!("zircon_runtime{}", std::env::consts::DLL_SUFFIX));
    let project_path = scratch
        .root()
        .join("missing-project")
        .join("project.zrproj");
    assert!(!runtime_library.exists());
    assert!(!runtime_manifest_path(&runtime_library).exists());
    assert!(!project_path.exists());

    let mut command = editor_command(&scratch, &runtime_library);
    command
        .arg("--project")
        .arg(&project_path)
        .arg("--builtin-view")
        .arg("editor.scene");
    let output = run_editor(command, &scratch);
    let stderr = &output.stderr;

    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.contains("editor startup diagnostic: component=editor_app")
            && stderr.contains("requested=--project")
            && stderr.contains("--builtin-view editor.scene")
            && stderr.contains("cause=--project cannot be combined with --builtin-view")
            && stderr.contains("recovery=provide one valid editor startup mode"),
        "the product error must explain the argument conflict and recovery: {stderr}"
    );
    assert!(
        !stderr.contains("component=runtime_build_set")
            && !stderr.contains("runtime startup diagnostic"),
        "argument routing must reject the conflict before consulting the runtime override: {stderr}"
    );
    assert_no_panic_diagnostics("stdout", &output.stdout);
    assert_no_panic_diagnostics("stderr", stderr);
}

fn editor_command(scratch: &TestScratch, runtime_library: &Path) -> Command {
    let mut command = Command::new(EDITOR_EXE);
    command
        .env_remove(RUNTIME_LIBRARY_ENV)
        .env_remove(FIRST_FRAME_CAPTURE_ENV)
        .env_remove("ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME")
        .env_remove("ZIRCON_LOG_ROOT")
        .env_remove("ZIRCON_LOG_FILTER")
        .env_remove("ZIRCON_LOG")
        .env_remove("RUST_LOG")
        .env_remove("ZIRCON_LOG_LEVEL")
        .env(RUNTIME_LIBRARY_ENV, runtime_library)
        .env(LOG_ROOT_ENV, scratch.log_root())
        .stdout(Stdio::from(
            File::create(scratch.stdout_path()).expect("create captured stdout file"),
        ))
        .stderr(Stdio::from(
            File::create(scratch.stderr_path()).expect("create captured stderr file"),
        ));
    command
}

fn run_editor(mut command: Command, scratch: &TestScratch) -> CapturedOutput {
    let child = command.spawn().expect("launch the editor product binary");
    drop(command);
    let mut child = ReapOnDrop::new(child);
    let deadline = Instant::now() + STARTUP_TIMEOUT;

    let status = loop {
        match child.process_mut().try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let kill_result = child.process_mut().kill();
                let reap_result = child.process_mut().wait();
                let output = read_captured_streams(scratch);
                panic!(
                    "editor product did not exit within {STARTUP_TIMEOUT:?}; kill={kill_result:?}, reap={reap_result:?}; stdout: {}; stderr: {}",
                    output.stdout,
                    output.stderr,
                );
            }
            Err(wait_error) => {
                let kill_result = child.process_mut().kill();
                let reap_result = child.process_mut().wait();
                let output = read_captured_streams(scratch);
                panic!(
                    "failed while waiting for editor product: {wait_error}; kill={kill_result:?}, reap={reap_result:?}; stdout: {}; stderr: {}",
                    output.stdout,
                    output.stderr,
                );
            }
        }
    };

    let streams = read_captured_streams(scratch);
    CapturedOutput {
        status,
        stdout: streams.stdout,
        stderr: streams.stderr,
    }
}

fn read_captured_streams(scratch: &TestScratch) -> CapturedStreams {
    let stdout = fs::read(scratch.stdout_path()).expect("read captured stdout file");
    let stderr = fs::read(scratch.stderr_path()).expect("read captured stderr file");
    CapturedStreams {
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    }
}

fn assert_no_panic_diagnostics(stream: &str, output: &str) {
    assert!(
        !output.contains("panicked") && !output.contains("stack backtrace:"),
        "startup failures must not panic or emit a stack backtrace on {stream}: {output}"
    );
}

fn runtime_manifest_path(library_path: &Path) -> PathBuf {
    let file_name = library_path
        .file_name()
        .expect("runtime override must have a file name")
        .to_string_lossy();
    library_path.with_file_name(format!("{file_name}.manifest.json"))
}

struct TestScratch {
    root: PathBuf,
}

impl TestScratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_editor_f0_{label}_{}_{}",
            std::process::id(),
            nonce
        ));
        fs::create_dir(&root).expect("create isolated editor startup test directory");
        let scratch = Self { root };
        fs::create_dir_all(scratch.log_root()).expect("create isolated diagnostic log root");
        scratch
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn log_root(&self) -> PathBuf {
        self.root.join("logs")
    }

    fn stdout_path(&self) -> PathBuf {
        self.root.join("stdout.txt")
    }

    fn stderr_path(&self) -> PathBuf {
        self.root.join("stderr.txt")
    }
}

impl Drop for TestScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct ReapOnDrop {
    child: Child,
}

impl ReapOnDrop {
    fn new(child: Child) -> Self {
        Self { child }
    }

    fn process_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}

impl Drop for ReapOnDrop {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

struct CapturedOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

struct CapturedStreams {
    stdout: String,
    stderr: String,
}
