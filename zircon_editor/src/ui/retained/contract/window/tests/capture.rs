use std::io::Write;
use std::path::Path;

use super::{
    flush_editor_capture_writer, sync_editor_capture_writer, write_editor_frame_png,
    EditorCaptureSync,
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

impl EditorCaptureSync for SyncFailureWriter {
    fn sync_editor_capture(&self) -> std::io::Result<()> {
        Err(std::io::Error::other("sync unavailable"))
    }
}

fn resolve_capture_path(path: &Path) -> ResolvedProjectPath {
    ProjectPaths::resolve_path(path).expect("capture path should resolve")
}

fn capture_test_root(case_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-frame-capture-{}-{case_name}",
        std::process::id()
    ))
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
fn editor_frame_png_encoder_roundtrips_rgba_pixels() {
    let root = capture_test_root("roundtrip");
    let path = root.join("editor-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    let rgba = [255, 0, 0, 255, 0, 255, 0, 128];
    let resolved_path = resolve_capture_path(&path);

    write_editor_frame_png(&resolved_path, 2, 1, &rgba).expect("frame capture PNG should encode");
    let decoded = image::open(&path)
        .expect("written editor capture PNG should decode")
        .to_rgba8();

    assert_eq!(decoded.dimensions(), (2, 1));
    assert_eq!(decoded.as_raw(), &rgba);
    assert_eq!(
        partial_capture_files(&root),
        Vec::<std::path::PathBuf>::new()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn editor_frame_png_encoder_rejects_mismatched_rgba_without_writing_evidence() {
    let root = capture_test_root("mismatched-rgba");
    let path = root.join("editor-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    let resolved_path = resolve_capture_path(&path);

    let error = write_editor_frame_png(&resolved_path, 2, 1, &[255, 0, 0, 255])
        .expect_err("truncated RGBA frame must not produce PNG evidence");

    assert!(error.to_string().contains("does not match 2x1 output"));
    assert!(!path.exists());
}

#[test]
fn editor_frame_png_encoder_cleans_staging_file_when_encoding_fails() {
    let root = capture_test_root("encode-failure");
    let path = root.join("editor-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let resolved_path = resolve_capture_path(&path);

    let error = write_editor_frame_png(&resolved_path, 0, 1, &[])
        .expect_err("zero-width PNG evidence must fail during encoding");

    assert!(error
        .to_string()
        .contains("failed to encode editor first-frame capture"));
    assert!(!path.exists());
    assert_eq!(
        partial_capture_files(&root),
        Vec::<std::path::PathBuf>::new()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn editor_frame_png_encoder_cleans_staging_file_when_commit_fails() {
    let root = capture_test_root("commit-failure");
    let path = root.join("editor-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&path).unwrap();
    let resolved_path = resolve_capture_path(&path);

    let error = write_editor_frame_png(&resolved_path, 1, 1, &[255, 0, 0, 255])
        .expect_err("a directory cannot be committed as PNG evidence");

    assert!(error
        .to_string()
        .contains("failed to commit editor first-frame capture"));
    assert!(path.is_dir(), "failed commit must preserve the destination");
    assert_eq!(
        partial_capture_files(&root),
        Vec::<std::path::PathBuf>::new()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn editor_frame_png_encoder_replaces_existing_evidence_only_after_success() {
    let root = capture_test_root("replace-existing");
    let path = root.join("editor-first-frame.png");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(&path, b"stale evidence").unwrap();
    let resolved_path = resolve_capture_path(&path);

    write_editor_frame_png(&resolved_path, 1, 1, &[1, 2, 3, 255])
        .expect("complete PNG should replace stale evidence");
    let decoded = image::open(&path).unwrap().to_rgba8();

    assert_eq!(decoded.dimensions(), (1, 1));
    assert_eq!(decoded.as_raw(), &[1, 2, 3, 255]);
    assert_eq!(
        partial_capture_files(&root),
        Vec::<std::path::PathBuf>::new()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn editor_frame_capture_flush_and_sync_failures_are_not_reported_as_success() {
    let path = resolve_capture_path(Path::new("editor-first-frame.png"));
    let flush_error = flush_editor_capture_writer(&mut FlushFailureWriter, &path)
        .expect_err("flush failure must block frame capture commit");
    let sync_error = sync_editor_capture_writer(&SyncFailureWriter, &path)
        .expect_err("sync failure must block frame capture commit");

    assert!(flush_error
        .to_string()
        .contains("flush editor first-frame capture"));
    assert!(flush_error.to_string().contains("flush unavailable"));
    assert!(sync_error
        .to_string()
        .contains("sync editor first-frame capture"));
    assert!(sync_error.to_string().contains("sync unavailable"));
}
