use std::path::PathBuf;
use std::time::Duration;

use super::*;

#[test]
fn build_report_summary_uses_last_non_empty_diagnostic() {
    let report = BuildExecutionReport {
        status_code: Some(101),
        stdout: "Compiling zircon_hub\n".to_string(),
        stderr: "error: failed to compile\n\n".to_string(),
    };

    assert_eq!(report.summary_line(), "error: failed to compile");
    assert_eq!(
        report.recovery_hint(),
        "tools/zircon_build.py exited with code 101; open Build History and fix the first reported error before retrying"
    );
    assert_eq!(
        report.log_excerpt(),
        "error: failed to compile\nCompiling zircon_hub"
    );
    assert!(!report.process_exited_successfully());
}

#[test]
fn zero_process_exit_is_only_a_stage_fact_until_artifacts_are_qualified() {
    let report = BuildExecutionReport {
        status_code: Some(0),
        stdout: String::new(),
        stderr: String::new(),
    };

    assert!(report.process_exited_successfully());
}

#[test]
fn build_capture_collection_reads_only_the_bounded_report_tail() {
    let path = std::env::temp_dir().join(format!(
        "zircon-hub-build-report-tail-{}",
        std::process::id()
    ));
    let mut input = vec![b'x'; BUILD_REPORT_TAIL_BYTES as usize + 17];
    input.extend_from_slice(b"terminal diagnostic\n");
    fs::write(&path, &input).expect("write build report fixture");

    let output = read_capture_tail(&path).expect("read bounded build report tail");

    assert!(output.len() <= BUILD_REPORT_TAIL_BYTES as usize);
    assert!(output.ends_with(b"terminal diagnostic\n"));
    fs::remove_file(path).expect("remove build report fixture");
}

#[test]
fn cancellation_requested_before_start_does_not_spawn_the_build_command() {
    let cancellation = TaskCancellationToken::new(1);
    cancellation.request_cancellation();
    let command = fixture_command("missing-build-command.exe", Vec::new());

    assert_eq!(
        run_build_command(&command, &cancellation).unwrap(),
        TaskExecutionOutcome::Cancelled
    );
}

#[test]
fn cancellation_terminates_and_reaps_the_monitored_build_process() {
    let cancellation = TaskCancellationToken::new(1);
    let cancellation_request = cancellation.clone();
    let request = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        cancellation_request.request_cancellation();
    });
    let command = fixture_command(
        std::env::current_exe().expect("current test executable"),
        vec![
            "--exact".to_string(),
            "build::runner::tests::cancellation_fixture_process".to_string(),
            "--ignored".to_string(),
        ],
    );

    let outcome = run_build_command(&command, &cancellation)
        .expect("cancel monitored build fixture without an I/O failure");
    request.join().expect("join cancellation request");

    assert_eq!(outcome, TaskExecutionOutcome::Cancelled);
}

fn fixture_command(program: impl Into<PathBuf>, args: Vec<String>) -> BuildCommand {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    BuildCommand {
        program: program.into(),
        args,
        capture_dir: cwd.join(".codex").join("scratch").join("hub-build-capture"),
        cwd,
    }
}

#[test]
#[ignore = "child fixture for cancellation_terminates_and_reaps_the_monitored_build_process"]
fn cancellation_fixture_process() {
    thread::sleep(Duration::from_secs(30));
}
