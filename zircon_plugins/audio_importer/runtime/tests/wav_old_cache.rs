use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;

#[path = "support/wav_old_cache.rs"]
mod support;

#[path = "support/wav_fixture.rs"]
mod wav_fixture;

use support::{capture, check_version_control, open, produce_v1, producer_basis, scan};

const V1: u32 = 1;
const TEST_VERSION_OVERRIDE: u32 = 2;
const REOPEN: &str = "reopen-v1";
const VERSION_CONTROL: &str = "descriptor-version-control";
const WITNESS: &str = "warm-cold-v1";
const RETAINED: &str = "retained-v1";
const OVERRIDE: &str = "version-control";
const CHILD_TEST: &str = "wav_cache_reopen_child";
const RUN_ENV: &str = "ZIRCON_WAV156_CHILD_RUN";
const PHASE_ENV: &str = "ZIRCON_WAV156_CHILD_PHASE";
const MARKER: &str = "wav156-run.txt";

#[test]
#[ignore = "explicit historical WAV v1 disk-cache probe; retains specimens"]
fn wav156_v1_cache_reopens_in_fresh_process() {
    let (id, run) = new_run(REOPEN);
    let witness = run.join(WITNESS);
    let retained = run.join(RETAINED);
    let baseline = produce_v1(&witness, true);
    let untouched = produce_v1(&retained, false);
    baseline.assert_distinct_ids(&untouched);

    run_child(&id, REOPEN);
    assert_eq!(baseline, capture(&witness, "parent-after-child", V1));
    assert_eq!(untouched, capture(&retained, "retained-v1", V1));
    eprintln!("wav156 sampled v1 bypass; retained old projects at {run:?}");
}

#[test]
#[ignore = "explicit historical WAV v1 disk-cache probe; retains specimens"]
fn wav156_descriptor_version_control_reimports_in_fresh_process() {
    let (id, run) = new_run(VERSION_CONTROL);
    produce_v1(&run.join(OVERRIDE), false);
    run_child(&id, VERSION_CONTROL);
    eprintln!("wav156 sampled WAV version-field-only control at {run:?}");
}

#[test]
#[ignore = "child entry point launched by the wav156 parent tests"]
fn wav_cache_reopen_child() {
    let id = setup(env::var(RUN_ENV), "child run ID");
    let phase = setup(env::var(PHASE_ENV), "child phase");
    let (pid, time) = id.split_once('-').expect("setup child run ID shape");
    let producer: u32 = setup(pid.parse(), "producer PID");
    let time: u128 = setup(time.parse(), "producer clock");
    assert_eq!(id, format!("{producer}-{time}"), "setup child run ID");
    assert_ne!(producer, process::id(), "setup requires a new process");
    let (folder, version, calls_per_wav) = match phase.as_str() {
        REOPEN => (WITNESS, V1, 0),
        VERSION_CONTROL => (OVERRIDE, TEST_VERSION_OVERRIDE, 1),
        _ => panic!("setup unknown child phase {phase:?}"),
    };

    let run = run_path(&id, &phase);
    assert_owned_directory(&run);
    let marker = setup(fs::read_to_string(run.join(MARKER)), "read run marker");
    assert_eq!(marker, marker_text(&id, &phase), "setup child marker");
    eprintln!(
        "wav156 child pid={} producer={producer} phase={phase} root={run:?}",
        process::id()
    );

    let project = run.join(folder);
    assert_owned_directory(&project);
    // Read the *persisted* lib:// locators before opening the new service or scanning.
    let before = capture(&project, "child-old-lib-read", V1);
    let producer_build = producer_basis(&project, &before);
    let (mut pm, calls) = open(&project, version);
    let (after, build) = scan(&mut pm, &calls, &phase, version, calls_per_wav);
    if version == V1 {
        assert_eq!(
            before, after,
            "sampled cold hit must retain old identity and bytes"
        );
    } else {
        assert_eq!(
            build.as_deref(),
            Some(producer_build.as_str()),
            "version control build inputs"
        );
        check_version_control(&before, &after);
    }
    // libtest can leave its test-name prefix on the current stdout line.
    println!("\n{}", completion_line(&id, &phase));
}

fn run_child(id: &str, phase: &str) {
    let output = setup(
        Command::new(executable())
            .args([
                "--ignored",
                "--exact",
                CHILD_TEST,
                "--nocapture",
                "--test-threads=1",
            ])
            .env(RUN_ENV, id)
            .env(PHASE_ENV, phase)
            .output(),
        "launch fresh test process",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!(
        "wav156 child phase={phase} status={}\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let completed = completion_line(id, phase);
    assert!(
        output.status.success() && stdout.lines().any(|line| line == completed.as_str()),
        "child must finish the selected probe ({phase})"
    );
}

fn completion_line(id: &str, phase: &str) -> String {
    format!("wav156 child completed {id} {phase}")
}

fn new_run(phase: &str) -> (String, PathBuf) {
    let time = setup(
        SystemTime::now().duration_since(UNIX_EPOCH),
        "read producer clock",
    );
    let id = format!("{}-{}", process::id(), time.as_nanos());
    let run = run_path(&id, phase);
    eprintln!("wav156 producer root={run:?} executable={:?}", executable());
    setup(fs::create_dir(&run), "create new run root");
    assert_owned_directory(&run);
    setup(
        fs::write(run.join(MARKER), marker_text(&id, phase)),
        "write run marker",
    );
    (id, run)
}

fn run_path(id: &str, phase: &str) -> PathBuf {
    let temp = physical(&env::temp_dir());
    assert!(temp.is_dir(), "setup temp base must exist: {temp:?}");
    assert!(
        !temp.ancestors().any(|path| path.join(".git").exists()),
        "setup needs a temp directory outside a checkout: {temp:?}"
    );
    temp.join(format!("zr-wav156-{id}-{phase}"))
}

fn marker_text(id: &str, phase: &str) -> String {
    format!("reference_head=1e74ec6b674d0d7c04d5ee7f1b29c8e55ed03e96\nrun={id}\nphase={phase}\nexe={:?}\n", executable())
}

fn assert_owned_directory(path: &Path) {
    let entry = setup(fs::symlink_metadata(path), "check run directory entry");
    assert!(
        entry.file_type().is_dir(),
        "setup expected an actual directory: {path:?}"
    );
    assert_eq!(
        physical(path).as_path(),
        path,
        "setup encountered a directory alias"
    );
}

fn physical(path: &Path) -> PathBuf {
    setup(ProjectPaths::resolve_root(path), "resolve physical path")
}

fn executable() -> PathBuf {
    let exe = setup(env::current_exe(), "locate test executable");
    setup(exe.canonicalize(), "resolve test executable")
}

#[track_caller]
fn setup<T, E: std::fmt::Debug>(result: Result<T, E>, operation: &str) -> T {
    result.unwrap_or_else(|error| panic!("wav156 {operation}: {error:?}"))
}
