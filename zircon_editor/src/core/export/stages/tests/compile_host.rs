use super::*;

#[test]
fn system_runner_streams_full_logs_and_bounds_memory_tails() {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-system-build-output-{}-{:x}",
        std::process::id(),
        fixture_nonce()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let log_path = root.join("stdout.log");
    let bytes = (0..(MAX_COMMAND_OUTPUT_TAIL_BYTES + 8192))
        .map(|index| (index % 251) as u8)
        .collect::<Vec<_>>();

    fs::write(&log_path, &bytes).unwrap();
    let captured = capture_output_stream(File::open(&log_path).unwrap()).unwrap();

    assert_eq!(captured.byte_count, bytes.len() as u64);
    assert_eq!(captured.digest, blake3::hash(&bytes));
    assert_eq!(captured.tail.len(), MAX_COMMAND_OUTPUT_TAIL_BYTES);
    assert_eq!(
        captured.tail,
        bytes[bytes.len() - MAX_COMMAND_OUTPUT_TAIL_BYTES..]
    );
    assert_eq!(fs::read(&log_path).unwrap(), bytes);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn system_runner_redirects_process_output_to_owned_files_without_reader_threads() {
    let source = include_str!("../compile_host.rs");
    let runner = source
        .split("impl ZirconBuildCommandRunner for SystemZirconBuildCommandRunner")
        .nth(1)
        .expect("system build runner")
        .split("pub enum ZirconBuildCommandError")
        .next()
        .expect("system build runner body");

    assert!(runner.contains(".stdout(Stdio::from("));
    assert!(runner.contains(".stderr(Stdio::from("));
    assert!(!runner.contains("std::thread::spawn"));
    assert!(!runner.contains("join_output_capture"));
}

#[cfg(windows)]
#[test]
fn system_runner_preserves_both_redirected_output_logs_and_manifest_digests() {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-redirected-build-output-{}-{:x}",
        std::process::id(),
        fixture_nonce()
    ));
    fs::create_dir(&root).unwrap();
    let command = ZirconBuildCommand {
        program: OsString::from("cmd.exe"),
        args: vec![
            OsString::from("/C"),
            OsString::from("echo stdout-marker && echo stderr-marker 1>&2"),
        ],
        working_directory: root.clone(),
        stdout_log: root.join("stdout.log"),
        stderr_log: root.join("stderr.log"),
        output_manifest: root.join("output-log.json"),
    };

    let result = SystemZirconBuildCommandRunner.run(&command).unwrap();
    let stdout = fs::read(&command.stdout_log).unwrap();
    let stderr = fs::read(&command.stderr_log).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&command.output_manifest).unwrap()).unwrap();
    assert!(String::from_utf8_lossy(&stdout).contains("stdout-marker"));
    assert!(String::from_utf8_lossy(&stderr).contains("stderr-marker"));
    assert_eq!(result.stdout, stdout);
    assert_eq!(result.stderr, stderr);
    let stdout_digest = blake3::hash(&stdout).to_hex().to_string();
    let stderr_digest = blake3::hash(&stderr).to_hex().to_string();
    assert_eq!(
        manifest["stdout"]["byte_count"].as_u64(),
        Some(stdout.len() as u64)
    );
    assert_eq!(
        manifest["stderr"]["byte_count"].as_u64(),
        Some(stderr.len() as u64)
    );
    assert_eq!(
        manifest["stdout"]["digest"].as_str(),
        Some(stdout_digest.as_str())
    );
    assert_eq!(
        manifest["stderr"]["digest"].as_str(),
        Some(stderr_digest.as_str())
    );
    fs::remove_dir_all(root).unwrap();
}

fn fixture_nonce() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}
