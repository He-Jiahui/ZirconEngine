//! One real public late error; the parent owns the fixture and exact child lifetime.

use std::ffi::c_void;
use std::io::{Read, Write};
use std::mem::ManuallyDrop;
use std::os::windows::io::AsRawHandle;
use std::process::{Child, Command, ExitStatus, Stdio};

use super::*;

const SELECTOR: &str = "dynamic_api::session::construction::error_cleanup_tests::receipt_publication::app_core_shutdown_owner_public_late_play_error_waits_for_receipt_publication_without_reinvocation";
const CHILD_ROLE: &str = "ZIRCON_TEST_RECEIPT_PUBLICATION_CHILD_NONCE";
const CHILD_ROOT: &str = "ZIRCON_TEST_RECEIPT_PUBLICATION_ROOT";
const CHILD_PARENT: &str = "ZIRCON_TEST_RECEIPT_PUBLICATION_PARENT";
const CHILD_WATCHDOG: Duration = Duration::from_secs(30);
const CHILD_REAP_GRACE: Duration = Duration::from_secs(2);
const MAX_OUTPUT_BYTES: u64 = 64 * 1024;
const MAX_WITNESS_BYTES: u64 = 32 * 1024;

#[link(name = "kernel32")]
extern "system" {
    fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
}

#[test]
fn app_core_shutdown_owner_public_late_play_error_waits_for_receipt_publication_without_reinvocation(
) {
    if let Some(nonce) = std::env::var_os(CHILD_ROLE) {
        child_case(nonce.into_string().expect("UTF-8 exact-child nonce"));
        return;
    }
    let fixture = PlaySceneProject::create();
    let nonce = fixture
        .root
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let mut run = ChildRun::spawn(fixture, &nonce);
    let status = run.wait_until(Instant::now() + CHILD_WATCHDOG);
    assert!(
        !run.timed_out,
        "exact child watchdog expired; this is not a valid red witness"
    );
    let witness: serde_json::Value = serde_json::from_slice(&read_bounded(
        &run.fixture.root.join("receipt-witness.json"),
        MAX_WITNESS_BYTES,
    ))
    .expect("complete matching durable child witness");
    let stderr = String::from_utf8(read_bounded(
        &run.fixture.root.join("child.stderr"),
        MAX_OUTPUT_BYTES,
    ))
    .expect("UTF-8 exact-child stderr");
    let _stdout = read_bounded(&run.fixture.root.join("child.stdout"), MAX_OUTPUT_BYTES);
    assert_eq!(witness["nonce"], nonce);
    assert!(witness["owner"].as_u64().unwrap() > 0);
    assert!(witness["receipt"].as_u64().unwrap() > 0);
    assert!(witness["worker"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_eq!(witness["primary"]["variant"], "ProjectStep/ReadPlayScene");
    assert_eq!(witness["primary"]["step"], "load Play scene override");
    assert_eq!(witness["primary"]["read_kind"], "InvalidData");
    assert!(!witness["primary"]["read_source"]
        .as_str()
        .unwrap()
        .is_empty());
    assert!(!witness["primary"]["read_source_debug"]
        .as_str()
        .unwrap()
        .is_empty());
    assert!(witness["primary"]["read_os_error"].is_null());
    let read_path = PathBuf::from(witness["primary"]["read_path"].as_str().unwrap());
    assert_eq!(
        std::fs::canonicalize(read_path).unwrap(),
        std::fs::canonicalize(run.fixture.root.join(PLAY_SCENE)).unwrap()
    );
    assert!(witness["primary"]["allocated_handle"].as_u64().unwrap() > 0);
    assert_eq!(witness["primary"]["records"], 2);
    assert_eq!(witness["admitted"], 1);
    assert_eq!(witness["callback_entries"], 1);
    assert_eq!(witness["callback_completions"], 1);
    assert_eq!(witness["callback_completed"], true);
    assert_eq!(witness["same_original_deadline"], true);
    assert_eq!(witness["publisher_locked"], true);
    assert!(witness["actual_would_block"].as_u64().unwrap() > 0);
    assert_eq!(witness["would_block_state"], 0);
    assert!(
        witness["allocated_handle_published"].is_null(),
        "no Store lookup inside public create"
    );
    assert_eq!(witness["actual_core"]["all_modules_unloaded"], true);
    assert_eq!(
        witness["actual_core"]["all_owned_graph_workers_joined"],
        true
    );
    if !status.success() {
        assert_eq!(
            witness["owner_receipt"], "Pending",
            "different child failure is not this regression's red"
        );
        assert!(
            stderr.contains("fatal runtime session owner startup teardown incomplete"),
            "specific existing fatal policy missing: {stderr}"
        );
        panic!("actual unpublished receipt Mutex contention returned Pending and aborted; the public late error must return after the same owner's real join");
    }
    assert_eq!(witness["owner_receipt"], "Joined");
    assert_eq!(witness["actual_terminal"], true);
    assert_eq!(witness["tls_entered"], true);
    assert_eq!(witness["tls_completed"], true);
    assert_eq!(witness["tls_same_worker"], true);
    assert!(!stderr.contains("fatal runtime session owner startup teardown incomplete"));
    let returned: serde_json::Value = serde_json::from_slice(&read_bounded(
        &run.fixture.root.join("public-return.json"),
        MAX_WITNESS_BYTES,
    ))
    .expect("real public return witness");
    for key in ["nonce", "owner", "receipt", "primary"] {
        assert_eq!(returned["observation"][key], witness[key]);
    }
    assert_eq!(returned["code_is_error"], true);
    assert_eq!(returned["output_is_invalid"], true);
    assert_eq!(returned["observation"]["allocated_handle_published"], false);
    assert_eq!(returned["diagnostic"], witness["primary"]["diagnostic"]);
    run.fixture.assert_removable();
}

fn child_case(nonce: String) {
    assert!(!nonce.is_empty() && nonce.len() <= 200);
    assert!(nonce
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'));
    let supplied_root =
        PathBuf::from(std::env::var_os(CHILD_ROOT).expect("parent-owned child root"));
    let root =
        std::fs::canonicalize(&supplied_root).expect("borrow the existing parent-owned fixture");
    let parent = std::fs::canonicalize(PathBuf::from(
        std::env::var_os(CHILD_PARENT).expect("parent guard"),
    ))
    .unwrap();
    assert_eq!(root, supplied_root);
    assert_eq!(root.parent(), Some(parent.as_path()));
    assert_eq!(root.file_name().unwrap().to_str().unwrap(), nonce);
    // No PlaySceneProject is constructed here: only the parent can remove this root.
    let root_text = root.to_str().expect("UTF-8 parent fixture");
    let api = public_runtime_api();
    let create = api.create_session.expect("real public create");
    let destroy = api.destroy_session.expect("real public destroy");
    let mut healthy = ZrRuntimeSessionHandle::invalid();
    let status = unsafe { create(public_startup_config(root_text), &mut healthy) };
    assert_eq!(
        status.status_code(),
        ZrStatusCode::Ok,
        "{}",
        copy_status_diagnostic(status)
    );
    assert!(healthy.is_valid() && session_handle_is_published_for_test(healthy));
    let status = unsafe { destroy(healthy) };
    assert_eq!(
        status.status_code(),
        ZrStatusCode::Ok,
        "{}",
        copy_status_diagnostic(status)
    );
    assert!(!session_handle_is_published_for_test(healthy));
    std::fs::write(root.join(PLAY_SCENE), [0xff]).expect("corrupt this same valid Play snapshot");
    let play_path = std::fs::canonicalize(root.join(PLAY_SCENE)).unwrap();
    let witness_path = root.join("receipt-witness.json");
    assert!(!witness_path.exists());
    let outer_deadline = Instant::now() + CHILD_WATCHDOG;
    let ((code, output, diagnostic, observation), witness) =
        RuntimeStartupObservation::with_receipt_publication_for_test(
            nonce,
            play_path,
            witness_path,
            outer_deadline,
            || {
                let mut output = healthy;
                let (status, observation) = observe_runtime_startup_for_test(|| unsafe {
                    create(public_startup_config(root_text), &mut output)
                });
                (
                    status.status_code(),
                    output,
                    copy_status_diagnostic(status),
                    observation,
                )
            },
        );
    assert_eq!(code, ZrStatusCode::Error);
    assert_eq!(output, ZrRuntimeSessionHandle::invalid());
    assert_eq!(
        diagnostic,
        observation.primary_diagnostic.as_deref().unwrap()
    );
    assert_eq!(
        diagnostic,
        witness["primary"]["diagnostic"].as_str().unwrap()
    );
    assert_public_startup_closed(&observation, "load Play scene override");
    assert_eq!(witness["owner_receipt"], "Joined");
    assert_eq!(witness["actual_terminal"], true);
    assert_eq!(witness["tls_entered"], true);
    assert_eq!(witness["tls_completed"], true);
    assert_eq!(witness["tls_same_worker"], true);
    assert_eq!(witness["allocated_handle_published"], false);
    let raw = serde_json::to_vec(&serde_json::json!({"code_is_error": code == ZrStatusCode::Error, "output_is_invalid": output == ZrRuntimeSessionHandle::invalid(), "diagnostic": diagnostic, "observation": witness})).unwrap();
    assert!(raw.len() as u64 <= MAX_WITNESS_BYTES);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("public-return.json"))
        .expect("exclusive public return witness");
    file.write_all(&raw).unwrap();
    file.flush().unwrap();
    file.sync_all().unwrap();
}

fn read_bounded(path: &Path, maximum: u64) -> Vec<u8> {
    let mut raw = Vec::new();
    std::fs::File::open(path)
        .expect("owned child output")
        .take(maximum + 1)
        .read_to_end(&mut raw)
        .expect("bounded owned child output");
    assert!(
        raw.len() as u64 <= maximum,
        "owned child output exceeds bound"
    );
    raw
}

struct ChildRun {
    child: Option<Child>,
    fixture: ManuallyDrop<PlaySceneProject>,
    reaped: bool,
    timed_out: bool,
    terminal_observed: bool,
    cleanup_attempted: bool,
}

impl ChildRun {
    fn spawn(fixture: PlaySceneProject, nonce: &str) -> Self {
        let mut run = Self {
            child: None,
            fixture: ManuallyDrop::new(fixture),
            reaped: false,
            timed_out: false,
            terminal_observed: false,
            cleanup_attempted: false,
        };
        let root = std::fs::canonicalize(&run.fixture.root).expect("same newly created root");
        assert_eq!(root, run.fixture.created_root);
        assert_eq!(root.parent(), Some(run.fixture.fixture_parent.as_path()));
        let stdout = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("child.stdout"))
            .unwrap();
        let stderr = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("child.stderr"))
            .unwrap();
        run.child = Some(
            Command::new(std::env::current_exe().expect("this exact libtest executable"))
                .args(["--exact", SELECTOR, "--nocapture", "--test-threads=1"])
                .env(CHILD_ROLE, nonce)
                .env(CHILD_ROOT, &root)
                .env(CHILD_PARENT, &run.fixture.fixture_parent)
                .stdin(Stdio::null())
                .stdout(Stdio::from(stdout))
                .stderr(Stdio::from(stderr))
                .spawn()
                .expect("spawn exact isolated libtest child"),
        );
        run
    }

    fn wait_handle_until(&mut self, deadline: Instant) -> Result<bool, String> {
        let child = self.child.as_ref().expect("exact owned Child");
        let remaining = deadline.saturating_duration_since(Instant::now());
        let milliseconds = if remaining.is_zero() {
            0
        } else {
            remaining
                .as_millis()
                .saturating_add(1)
                .min((u32::MAX - 1) as u128) as u32
        };
        // A finite wait on this owned process handle; u32::MAX (INFINITE) is excluded.
        match unsafe { WaitForSingleObject(child.as_raw_handle().cast(), milliseconds) } {
            0 => {
                self.terminal_observed = true;
                Ok(true)
            }
            0x102 => Ok(false),
            other => Err(format!(
                "exact Child handle wait failed: {other:#x}: {}",
                std::io::Error::last_os_error()
            )),
        }
    }

    fn reap_signaled(&mut self) -> Result<ExitStatus, String> {
        if !self.terminal_observed {
            return Err("refuse Child::wait without signaled terminal evidence".to_owned());
        }
        self.cleanup_attempted = true;
        // Windows process termination is permanent. This wait only collects its exit status.
        let status = self
            .child
            .as_mut()
            .expect("exact owned Child")
            .wait()
            .map_err(|error| {
                format!("signaled exact Child exit status unavailable; preserve fixture: {error}")
            })?;
        self.reaped = true;
        Ok(status)
    }

    fn terminate_and_reap(&mut self) -> Result<ExitStatus, String> {
        if self.cleanup_attempted {
            return Err(
                "one bounded cleanup attempt already consumed; preserve fixture".to_owned(),
            );
        }
        self.cleanup_attempted = true;
        // This parent-only grace is independent of the original engine five-second deadline.
        // A subsequent Drop never restarts it or repeats this cleanup attempt.
        let cleanup_deadline = Instant::now()
            .checked_add(CHILD_REAP_GRACE)
            .unwrap_or_else(Instant::now);
        if !self.wait_handle_until(Instant::now())? {
            let kill_result = self.child.as_mut().expect("exact owned Child").kill();
            let terminal = self
                .wait_handle_until(cleanup_deadline)
                .map_err(|error| format!("{error}; kill={kill_result:?}; preserve fixture"))?;
            if !terminal {
                return Err(format!("exact Child exit remains unknown after bounded cleanup; kill={kill_result:?}; preserve fixture"));
            }
            if let Err(error) = kill_result {
                eprintln!("exact Child kill returned {error}, but its owned handle subsequently confirmed terminal exit");
            }
        }
        self.reap_signaled()
    }

    fn wait_until(&mut self, deadline: Instant) -> ExitStatus {
        match self.wait_handle_until(deadline) {
            Ok(true) => self
                .reap_signaled()
                .expect("collect signaled exact Child exit"),
            Ok(false) => {
                self.timed_out = true;
                self.terminate_and_reap()
                    .expect("bounded exact Child timeout cleanup")
            }
            Err(error) => panic!("parent watchdog could not confirm exact Child exit: {error}"),
        }
    }
}

impl Drop for ChildRun {
    fn drop(&mut self) {
        if !self.reaped && self.child.is_some() && !self.cleanup_attempted {
            if let Err(error) = self.terminate_and_reap() {
                eprintln!("bounded exact Child cleanup failed: {error}");
            }
        }
        if self.reaped || self.child.is_none() {
            // Successful reaping or a child that was never spawned permits fixture removal.
            unsafe { ManuallyDrop::drop(&mut self.fixture) };
        } else {
            eprintln!("preserve owned fixture {}: terminal_observed={}, reaped=false; no unbounded wait or cleanup retry", self.fixture.root.display(), self.terminal_observed);
        }
    }
}
