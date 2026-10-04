use std::io::Write;
use std::path::Path;

use super::{
    flush_frame_capture_writer, sync_frame_capture_writer, write_runtime_frame_png,
    FrameCaptureSync,
};
use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPath};

struct FlushFailureWriter;

impl Write for FlushFailureWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("flush unavailable"))
    }
}

struct SyncFailureWriter;

impl FrameCaptureSync for SyncFailureWriter {
    fn sync_frame_capture(&self) -> std::io::Result<()> {
        Err(std::io::Error::other("sync unavailable"))
    }
}

fn resolve_capture_path(path: &Path) -> ResolvedProjectPath {
    ProjectPaths::resolve_path(path).expect("capture path should resolve")
}

// 测试输出跟随受管理测试二进制的物理目录，避免落到当前目录或系统临时目录。
fn capture_test_root(case_name: &str) -> std::path::PathBuf {
    let executable = std::env::current_exe().expect("locate the frame-capture test executable");
    let binary_directory = executable
        .parent()
        .expect("frame-capture test executable must have a parent directory");
    let binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve the frame-capture test binary directory");

    binary_directory
        .operation_path()
        .join("zircon-mvp-fixtures")
        .join(format!(
            "zircon-runtime-frame-capture-{}-{case_name}",
            std::process::id()
        ))
}

#[test]
fn frame_capture_fixture_roots_follow_the_resolved_test_binary_directory() {
    let root = capture_test_root("physical-root");
    let executable = std::env::current_exe().expect("locate the frame-capture test executable");
    let binary_directory = executable
        .parent()
        .expect("frame-capture test executable must have a parent directory");
    let resolved_binary_directory = ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve frame-capture test binary directory");

    assert!(
        root.starts_with(resolved_binary_directory.operation_path()),
        "frame-capture fixture output must retain the test binary's physical output root"
    );
}

fn partial_capture_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(root)
        .expect("capture test root should remain readable")
        .map(|entry| {
            entry
                .expect("capture test entry should remain readable")
                .path()
        })
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().contains(".partial-"))
        })
        .collect()
}

#[test]
fn runtime_frame_png_encoder_roundtrips_rgba_pixels() {
    let root = capture_test_root("png-roundtrip");
    let path = root.join("runtime-first-frame.png");
    let rgba = [
        255, 0, 0, 255, // red
        0, 255, 0, 128, // green with alpha
    ];
    let resolved_path = resolve_capture_path(&path);

    write_runtime_frame_png(&resolved_path, 2, 1, &rgba).expect("frame capture PNG should encode");
    let decoded = image::open(&path)
        .expect("written frame capture PNG should decode")
        .to_rgba8();
    std::fs::remove_dir_all(root).expect("remove frame capture PNG fixture");

    assert_eq!(decoded.dimensions(), (2, 1));
    assert_eq!(decoded.as_raw(), &rgba);
}

#[test]
fn runtime_frame_png_encoder_rejects_mismatched_rgba_without_writing_evidence() {
    let root = capture_test_root("invalid-rgba");
    let path = root.join("runtime-first-frame.png");
    let resolved_path = resolve_capture_path(&path);

    let error = write_runtime_frame_png(&resolved_path, 2, 1, &[255, 0, 0, 255])
        .expect_err("truncated RGBA frame must not produce PNG evidence");

    assert!(error.contains("does not match 2x1 output"));
    assert!(!path.exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn runtime_frame_png_encoder_cleans_staging_file_when_commit_fails() {
    let root = capture_test_root("commit-failure");
    let path = root.join("runtime-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&path).unwrap();
    let resolved_path = resolve_capture_path(&path);

    let error = write_runtime_frame_png(&resolved_path, 1, 1, &[255, 0, 0, 255])
        .expect_err("a directory cannot be committed as PNG evidence");

    assert!(error.contains("commit frame capture"), "{error}");
    assert!(path.is_dir(), "failed commit must preserve the destination");
    assert!(partial_capture_files(&root).is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_frame_png_encoder_cleans_staging_file_when_encoding_fails() {
    let root = capture_test_root("encode-failure");
    let path = root.join("runtime-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let resolved_path = resolve_capture_path(&path);

    let error = write_runtime_frame_png(&resolved_path, 0, 1, &[])
        .expect_err("zero-width PNG evidence must fail during encoding");

    assert!(error.contains("encode frame capture"), "{error}");
    assert!(!path.exists());
    assert!(partial_capture_files(&root).is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_frame_png_encoder_replaces_existing_evidence_only_after_success() {
    let root = capture_test_root("replace-existing");
    let path = root.join("runtime-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(&path, b"stale evidence").unwrap();
    let resolved_path = resolve_capture_path(&path);

    write_runtime_frame_png(&resolved_path, 1, 1, &[1, 2, 3, 255])
        .expect("complete PNG should atomically replace stale evidence");
    let decoded = image::open(&path).unwrap().to_rgba8();

    assert_eq!(decoded.dimensions(), (1, 1));
    assert_eq!(decoded.as_raw(), &[1, 2, 3, 255]);
    assert!(partial_capture_files(&root).is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn frame_capture_flush_and_sync_failures_are_not_reported_as_success() {
    let path = resolve_capture_path(Path::new("runtime-first-frame.png"));
    let flush_error = flush_frame_capture_writer(&mut FlushFailureWriter, &path)
        .expect_err("flush failure must block frame capture commit");
    let sync_error = sync_frame_capture_writer(&SyncFailureWriter, &path)
        .expect_err("sync failure must block frame capture commit");

    assert!(flush_error.contains("flush frame capture"));
    assert!(flush_error.contains("flush unavailable"));
    assert!(sync_error.contains("sync frame capture"));
    assert!(sync_error.contains("sync unavailable"));
}
