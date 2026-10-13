//! sink 测试入口把格式、文件候选、批处理、背压、生命周期和性能契约组合到同一私有模块。
#[path = "backpressure.rs"]
mod backpressure;
#[path = "batching.rs"]
mod batching;
#[path = "durability.rs"]
mod durability;
#[path = "fixtures.rs"]
mod fixtures;
#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "performance/mod.rs"]
mod performance;

use std::path::PathBuf;

use super::{
    append_diagnostic_log_line, open_first_available_log_file, sanitize_channel_name,
    LogDirectoryCandidate,
};
use crate::diagnostic_log::DiagnosticLogLevel;

#[test]
fn channel_names_are_safe_file_stems() {
    assert_eq!(sanitize_channel_name("editor".to_string()), "editor");
    assert_eq!(
        sanitize_channel_name("runtime/player".to_string()),
        "runtime_player"
    );
    assert_eq!(sanitize_channel_name(String::new()), "runtime");
}

#[test]
fn diagnostic_log_lines_include_level_scope_and_escape_newlines() {
    let mut buffer = b"[2026-05-04-16-30-00] ".to_vec();
    append_diagnostic_log_line(
        &mut buffer,
        DiagnosticLogLevel::Warn,
        "runtime_asset_path",
        "first\nsecond",
    );
    let line = String::from_utf8(buffer).expect("diagnostic log line is UTF-8");

    assert_eq!(
        line,
        "[2026-05-04-16-30-00] [warn] [runtime_asset_path] first\\nsecond\n"
    );
}

#[test]
fn disabled_file_sink_skips_directory_candidates() {
    let mut notes = Vec::new();
    let (path, file) = open_first_available_log_file(
        "runtime",
        vec![LogDirectoryCandidate {
            source: "test",
            path: PathBuf::from("should-not-be-created"),
        }],
        false,
        &mut notes,
    );

    assert!(path.is_none());
    assert!(file.is_none());
    assert_eq!(notes.len(), 1);
    assert!(notes[0].message.contains("file-backed log sink disabled"));
}

#[test]
fn diagnostic_log_lines_preserve_literal_backslashes_and_unicode() {
    let mut buffer = b"[2026-05-04-16-30-00] ".to_vec();
    append_diagnostic_log_line(
        &mut buffer,
        DiagnosticLogLevel::Debug,
        "作用域",
        "literal \\n and C:\\assets\\界面\n第二行 🧭",
    );

    assert_eq!(
        String::from_utf8(buffer).expect("diagnostic log line is UTF-8"),
        "[2026-05-04-16-30-00] [debug] [作用域] literal \\n and C:\\assets\\界面\\n第二行 🧭\n"
    );
}

#[cfg(target_os = "windows")]
struct LogArtifactExpected {
    bytes: Vec<u8>,
    byte_len: u64,
    blake3: blake3::Hash,
    io_failed: bool,
}

#[cfg(target_os = "windows")]
impl LogArtifactExpected {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            byte_len: u64::try_from(bytes.len()).expect("fixture expected length fits u64"),
            blake3: blake3::hash(bytes),
            io_failed: false,
        }
    }

    fn record_input(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        if self.io_failed {
            return Err(std::io::Error::other("fixture output already failed"));
        }
        let added = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("fixture expected length overflow"))?;
        self.byte_len = self
            .byte_len
            .checked_add(added)
            .ok_or_else(|| std::io::Error::other("fixture expected length overflow"))?;
        self.bytes.extend_from_slice(bytes);
        self.blake3 = blake3::hash(&self.bytes);
        Ok(())
    }
}

#[cfg(target_os = "windows")]
struct LogArtifactFilePlan {
    path: PathBuf,
    creation_identity: u64,
    expected: std::sync::Arc<std::sync::Mutex<LogArtifactExpected>>,
    sealed_expected: Option<(u64, blake3::Hash)>,
}

#[cfg(target_os = "windows")]
struct LogArtifactOutput {
    file: std::fs::File,
    expected: std::sync::Arc<std::sync::Mutex<LogArtifactExpected>>,
}

#[cfg(target_os = "windows")]
impl std::io::Write for LogArtifactOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;

        let mut expected = self
            .expected
            .lock()
            .map_err(|_| std::io::Error::other("fixture expected-input plan poisoned"))?;
        // Dynamic timestamp bytes come from the formatter's real Write input, before File I/O.
        expected.record_input(bytes)?;
        match self.file.write_all(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(error) => {
                expected.io_failed = true;
                Err(error)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;

        let result = self.file.flush();
        if result.is_err() {
            self.expected
                .lock()
                .map_err(|_| std::io::Error::other("fixture expected-input plan poisoned"))?
                .io_failed = true;
        }
        result
    }
}

#[cfg(target_os = "windows")]
impl super::worker::DurableOutput for LogArtifactOutput {
    fn sync_data(&mut self) -> std::io::Result<()> {
        let result = self.file.sync_data();
        if result.is_err() {
            self.expected
                .lock()
                .map_err(|_| std::io::Error::other("fixture expected-input plan poisoned"))?
                .io_failed = true;
        }
        result
    }
}

#[cfg(target_os = "windows")]
struct LogArtifactFixture {
    root: PathBuf,
    physical_root: PathBuf,
    directories: Vec<(PathBuf, u64)>,
    files: Vec<LogArtifactFilePlan>,
    workers_joined: bool,
    sealed: bool,
    cleaned: bool,
    removed_files: usize,
    removed_directories: usize,
    removed_bytes: u64,
}

#[cfg(target_os = "windows")]
impl LogArtifactFixture {
    fn checked_identity(path: &std::path::Path, directory: bool) -> std::io::Result<u64> {
        use std::os::windows::fs::MetadataExt;

        let metadata = std::fs::symlink_metadata(path)?;
        if metadata.file_attributes() & 0x400 != 0
            || if directory {
                !metadata.is_dir()
            } else {
                !metadata.is_file()
            }
        {
            return Err(std::io::Error::other(
                "fixture refuses reparse points or changed path types",
            ));
        }
        Ok(metadata.creation_time())
    }

    fn checked_ancestors(path: &std::path::Path) -> std::io::Result<()> {
        for ancestor in path.ancestors() {
            match Self::checked_identity(ancestor, true) {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn new(label: &str) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};

        static FIXTURE_NONCE: AtomicU64 = AtomicU64::new(0);
        let approved = PathBuf::from("E:/cargo-targets/zircon-local");
        let container =
            approved.join("reports/runtime-editor-optimize/runtime44-file-generation-isolation-v1");
        assert!(approved.is_absolute() && container.starts_with(&approved));
        Self::checked_ancestors(&approved).expect("physical E root has no reparse ancestors");
        let physical_approved = std::fs::canonicalize(&approved).expect("approved E root exists");
        Self::checked_ancestors(&container).expect("fixture container has no reparse ancestors");
        std::fs::create_dir_all(&container).expect("prepare the registered fixture container");
        Self::checked_ancestors(&container)
            .expect("created fixture container has no reparse ancestors");
        let physical_container = std::fs::canonicalize(&container).unwrap();
        let expected_container = physical_approved
            .join("reports/runtime-editor-optimize/runtime44-file-generation-isolation-v1");
        assert_eq!(physical_container, expected_container);

        for _ in 0..32 {
            let nonce = FIXTURE_NONCE
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                    value.checked_add(1)
                })
                .expect("fixture nonce does not wrap");
            let root = container.join(format!("{label}-{}-{nonce:016x}", std::process::id()));
            match std::fs::create_dir(&root) {
                Ok(()) => {
                    let identity = Self::checked_identity(&root, true).unwrap();
                    let physical_root = std::fs::canonicalize(&root).unwrap();
                    assert_eq!(physical_root.parent(), Some(physical_container.as_path()));
                    return Self {
                        directories: vec![(root.clone(), identity)],
                        root,
                        physical_root,
                        files: Vec::new(),
                        workers_joined: true,
                        sealed: false,
                        cleaned: false,
                        removed_files: 0,
                        removed_directories: 0,
                        removed_bytes: 0,
                    };
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("exclusive fixture directory failed: {error}"),
            }
        }
        panic!("exclusive fixture directory collision limit reached");
    }

    fn directory(&mut self, name: &str) -> PathBuf {
        assert!(!self.sealed && !name.is_empty());
        assert!(name
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || value == b'-'));
        let path = self.root.join(name);
        std::fs::create_dir(&path).expect("only create a new owned directory");
        let identity = Self::checked_identity(&path, true).unwrap();
        self.directories.push((path.clone(), identity));
        path
    }

    fn remember_file(&mut self, path: PathBuf, expected_bytes: &[u8]) {
        assert!(!self.sealed);
        assert!(self
            .directories
            .iter()
            .any(|(directory, _)| Some(directory.as_path()) == path.parent()));
        assert!(
            !self.files.iter().any(|plan| plan.path == path),
            "exclusive creation cannot adopt a previously owned path"
        );
        let creation_identity = Self::checked_identity(&path, false).unwrap();
        self.files.push(LogArtifactFilePlan {
            path,
            creation_identity,
            expected: std::sync::Arc::new(std::sync::Mutex::new(LogArtifactExpected::new(
                expected_bytes,
            ))),
            sealed_expected: None,
        });
    }

    fn output(&self, path: &std::path::Path, file: std::fs::File) -> LogArtifactOutput {
        assert!(!self.sealed && !self.workers_joined);
        let plan = self.files.iter().find(|plan| plan.path == path).unwrap();
        LogArtifactOutput {
            file,
            expected: std::sync::Arc::clone(&plan.expected),
        }
    }

    fn expected_bytes(&self, path: &std::path::Path) -> Vec<u8> {
        let plan = self.files.iter().find(|plan| plan.path == path).unwrap();
        let expected = plan.expected.lock().unwrap();
        assert!(!expected.io_failed);
        expected.bytes.clone()
    }

    fn seed(&mut self, path: PathBuf, bytes: &[u8]) {
        use std::io::Write;

        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        // Known seed input and its expected length/hash are registered before any write.
        self.remember_file(path, bytes);
        file.write_all(bytes).unwrap();
        file.flush().unwrap();
    }

    fn checked_root(&self) -> std::io::Result<()> {
        Self::checked_ancestors(&self.root)?;
        if std::fs::canonicalize(&self.root)? != self.physical_root {
            return Err(std::io::Error::other("fixture physical owner changed"));
        }
        let (_, identity) = self.directories.first().ok_or_else(|| {
            std::io::Error::other("fixture root creation identity is not manifested")
        })?;
        if Self::checked_identity(&self.root, true)? != *identity {
            return Err(std::io::Error::other(
                "fixture root creation identity changed",
            ));
        }
        Ok(())
    }

    fn checked_directory(&self, path: &std::path::Path, identity: u64) -> std::io::Result<()> {
        Self::checked_ancestors(path)?;
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| std::io::Error::other("fixture directory is outside owned root"))?;
        if Self::checked_identity(path, true)? != identity
            || std::fs::canonicalize(path)? != self.physical_root.join(relative)
        {
            return Err(std::io::Error::other(
                "fixture directory creation or physical identity changed",
            ));
        }
        Ok(())
    }

    fn checked_file(&self, plan: &LogArtifactFilePlan) -> std::io::Result<(u64, blake3::Hash)> {
        self.checked_root()?;
        Self::checked_ancestors(plan.path.parent().unwrap())?;
        let relative = plan
            .path
            .strip_prefix(&self.root)
            .map_err(|_| std::io::Error::other("fixture file is outside owned root"))?;
        let expected = plan
            .expected
            .lock()
            .map_err(|_| std::io::Error::other("fixture expected-input plan poisoned"))?;
        let fingerprint = (expected.byte_len, expected.blake3);
        if expected.io_failed
            || u64::try_from(expected.bytes.len()).ok() != Some(expected.byte_len)
            || blake3::hash(&expected.bytes) != expected.blake3
            || plan
                .sealed_expected
                .is_some_and(|sealed| sealed != fingerprint)
            || (self.sealed && plan.sealed_expected.is_none())
        {
            return Err(std::io::Error::other(
                "fixture expected-input plan failed or changed after sealing",
            ));
        }
        if Self::checked_identity(&plan.path, false)? != plan.creation_identity
            || std::fs::canonicalize(&plan.path)? != self.physical_root.join(relative)
            || std::fs::symlink_metadata(&plan.path)?.len() != expected.byte_len
        {
            return Err(std::io::Error::other(
                "fixture file creation, physical identity or expected length changed",
            ));
        }
        let actual = std::fs::read(&plan.path)?;
        if u64::try_from(actual.len()).ok() != Some(expected.byte_len)
            || blake3::hash(&actual) != expected.blake3
            || Self::checked_identity(&plan.path, false)? != plan.creation_identity
        {
            return Err(std::io::Error::other(
                "fixture file content differs from expected byteLen/BLAKE3; retain evidence",
            ));
        }
        Ok(fingerprint)
    }

    fn validate_manifest(&self) -> std::io::Result<()> {
        use std::collections::HashSet;

        self.checked_root()?;
        for (path, identity) in &self.directories {
            self.checked_directory(path, *identity)?;
        }
        for plan in &self.files {
            self.checked_file(plan)?;
        }
        for (directory, _) in &self.directories {
            let expected: HashSet<_> = self
                .files
                .iter()
                .map(|plan| &plan.path)
                .chain(self.directories.iter().map(|(path, _)| path))
                .filter(|path| path.parent() == Some(directory.as_path()))
                .cloned()
                .collect();
            let actual: HashSet<_> = std::fs::read_dir(directory)?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<std::io::Result<_>>()?;
            if actual != expected {
                return Err(std::io::Error::other(
                    "fixture directory entries differ from full owned manifest; retain evidence",
                ));
            }
        }
        Ok(())
    }

    fn seal(&mut self) -> std::io::Result<()> {
        if std::thread::panicking() || !self.workers_joined || self.sealed {
            return Err(std::io::Error::other(
                "fixture cannot seal while panicking, unjoined or already sealed",
            ));
        }
        // Callers reach here only after all join, content, FIFO, UTF-8 and metrics assertions.
        self.validate_manifest()?;
        for plan in &mut self.files {
            let expected = plan
                .expected
                .lock()
                .map_err(|_| std::io::Error::other("fixture expected-input plan poisoned"))?;
            plan.sealed_expected = Some((expected.byte_len, expected.blake3));
        }
        self.sealed = true;
        Ok(())
    }

    fn cleanup(&mut self) -> std::io::Result<()> {
        if std::thread::panicking() || !self.workers_joined || !self.sealed {
            return Err(std::io::Error::other(
                "fixture panicking, unjoined or unsealed; retain evidence",
            ));
        }
        // Check every planned file and directory before the first deletion.
        self.validate_manifest()?;
        while let Some(plan) = self.files.last() {
            let (byte_len, _) = self.checked_file(plan)?;
            std::fs::remove_file(&plan.path)?;
            self.files.pop();
            self.removed_files += 1;
            self.removed_bytes += byte_len;
        }
        while let Some((path, identity)) = self.directories.last() {
            self.checked_directory(path, *identity)?;
            if std::fs::read_dir(path)?.next().transpose()?.is_some() {
                return Err(std::io::Error::other(
                    "fixture directory is no longer empty; retain remaining evidence",
                ));
            }
            std::fs::remove_dir(path)?;
            self.directories.pop();
            self.removed_directories += 1;
        }
        self.cleaned = true;
        eprintln!(
            "runtime44_file_collision_fixture_cleanup files_removed={} directories_removed={} released_bytes={}",
            self.removed_files, self.removed_directories, self.removed_bytes
        );
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl Drop for LogArtifactFixture {
    fn drop(&mut self) {
        if !self.cleaned {
            // Failure and unwind paths preserve evidence; only explicit sealed cleanup deletes.
            eprintln!(
                "runtime44_file_collision_fixture_retained root={} files={} directories={} panicking={} workers_joined={} sealed={} files_removed={} directories_removed={} released_bytes={}",
                self.root.display(), self.files.len(), self.directories.len(),
                std::thread::panicking(), self.workers_joined, self.sealed,
                self.removed_files, self.removed_directories, self.removed_bytes
            );
            for plan in &self.files {
                match plan.expected.lock() {
                    Ok(expected) => eprintln!(
                        "runtime44_file_collision_fixture_plan path={} creation_identity={} expected_byte_len={} expected_blake3={} io_failed={} sealed={}",
                        plan.path.display(), plan.creation_identity, expected.byte_len,
                        expected.blake3, expected.io_failed, plan.sealed_expected.is_some()
                    ),
                    Err(_) => eprintln!(
                        "runtime44_file_collision_fixture_plan path={} creation_identity={} expected_plan_poisoned=true",
                        plan.path.display(), plan.creation_identity
                    ),
                }
            }
            for (path, identity) in &self.directories {
                eprintln!(
                    "runtime44_file_collision_fixture_directory path={} creation_identity={identity}",
                    path.display()
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
#[test]
fn same_timestamp_generations_use_distinct_files_and_join_workers() {
    use std::collections::HashSet;
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    let mut fixture = LogArtifactFixture::new("generation-isolation");
    let directory = fixture.directory("2026-10-12-12-34-56");
    let mut opened = Vec::new();
    for _ in 0..2 {
        let mut notes = Vec::new();
        let (path, file) = open_first_available_log_file(
            "runtime",
            vec![LogDirectoryCandidate {
                source: "same-timestamp",
                path: directory.clone(),
            }],
            true,
            &mut notes,
        );
        let path = path.expect("new generation selects its actual path");
        fixture.remember_file(path.clone(), b"");
        opened.push((path, file.expect("new generation owns a real File")));
    }

    let start = Arc::new((Mutex::new(false), Condvar::new()));
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let directory = directory.clone();
            let start = Arc::clone(&start);
            thread::Builder::new().spawn(move || {
                let (ready, changed) = &*start;
                let mut ready = ready.lock().unwrap();
                while !*ready {
                    ready = changed.wait(ready).unwrap();
                }
                drop(ready);
                let mut notes = Vec::new();
                open_first_available_log_file(
                    "runtime",
                    vec![LogDirectoryCandidate {
                        source: "concurrent-generation",
                        path: directory,
                    }],
                    true,
                    &mut notes,
                )
            })
        })
        .collect();
    {
        let (ready, changed) = &*start;
        *ready.lock().unwrap() = true;
        changed.notify_all();
    }
    // Release every successfully spawned opener even if another spawn failed, then join all.
    let results: Vec<_> = threads
        .into_iter()
        .map(|spawned| spawned.map(|thread| thread.join()))
        .collect();
    for result in results {
        let (path, file) = result
            .expect("concurrent opener spawns")
            .expect("all concurrent openers terminate");
        let path = path.expect("concurrent generation selects an actual path");
        fixture.remember_file(path.clone(), b"");
        opened.push((path, file.expect("concurrent generation owns a real File")));
    }
    let paths: Vec<_> = opened.iter().map(|(path, _)| path.clone()).collect();
    assert_eq!(
        paths.iter().collect::<HashSet<_>>().len(),
        6,
        "fresh generations cannot reopen one artifact"
    );
    assert_eq!(paths[0], directory.join("runtime.log"));
    assert!(paths
        .iter()
        .all(|path| path.extension().is_some_and(|value| value == "log")));

    fixture.workers_joined = false;
    let owners: Vec<_> = opened
        .into_iter()
        .map(|(path, file)| {
            super::worker::SinkRuntime::start(
                Some(Box::new(fixture.output(&path, file))),
                false,
                crate::diagnostic_log::DiagnosticLogSinkSettings::default(),
            )
        })
        .collect();
    let outcomes: Vec<_> = owners
        .iter()
        .enumerate()
        .map(|(index, owner)| {
            owner.as_ref().map(|sink| {
                let first = sink.enqueue(
                    DiagnosticLogLevel::Log,
                    "artifact_collision",
                    &format!("generation-{index}-first 🧭"),
                );
                let second = sink.enqueue(
                    DiagnosticLogLevel::Log,
                    "artifact_collision",
                    &format!("generation-{index}-second 界面"),
                );
                let flushed = sink.flush(Duration::from_secs(5));
                let joined = sink.shutdown_for_library_unload(Duration::from_secs(5));
                (
                    first,
                    second,
                    flushed,
                    joined,
                    sink.outputs_succeeded(),
                    sink.snapshot(),
                )
            })
        })
        .collect();
    fixture.workers_joined = outcomes
        .iter()
        .all(|outcome| outcome.as_ref().is_ok_and(|value| value.3));
    assert!(
        fixture.workers_joined,
        "every actual worker must finish and be joined before fixture cleanup"
    );
    for (index, outcome) in outcomes.into_iter().enumerate() {
        let (first, second, flushed, joined, outputs_succeeded, snapshot) = outcome.unwrap();
        assert!(first && second && flushed && joined && outputs_succeeded);
        assert!(snapshot.closed);
        assert_eq!(snapshot.written_records, 2);
        assert_eq!(snapshot.output_errors, 0);
        let expected = fixture.expected_bytes(&paths[index]);
        let text = std::fs::read_to_string(&paths[index]).unwrap();
        assert_eq!(text.as_bytes(), expected.as_slice());
        assert_eq!(snapshot.written_bytes, expected.len() as u64);
        assert_eq!(text.lines().count(), 2);
        let first = format!("generation-{index}-first 🧭");
        let second = format!("generation-{index}-second 界面");
        assert!(text.find(&first).unwrap() < text.find(&second).unwrap());
        for other in 0..paths.len() {
            if other != index {
                assert!(!text.contains(&format!("generation-{other}-")));
            }
        }
    }
    fixture
        .seal()
        .expect("seal expected-input plans only after every join and content assertion");
    fixture
        .cleanup()
        .expect("remove only sealed, joined and hash-matched owned fixtures");
}

#[cfg(target_os = "windows")]
#[test]
fn file_collision_errors_preserve_existing_artifacts_and_directory_fallback() {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    let mut fixture = LogArtifactFixture::new("collision-fallback");
    let blocked = fixture.root.join("blocked");
    fixture.seed(blocked.clone(), b"directory candidate is an owned file");
    let collisions = fixture.directory("collisions");
    let fallback = fixture.directory("fallback");
    let sentinel = collisions.join("runtime.log");
    fixture.seed(sentinel.clone(), b"existing generation remains byte exact");
    let collision_paths: Vec<_> = (0..32)
        .map(|nonce| collisions.join(format!("runtime-{}-{nonce:016x}.log", std::process::id())))
        .collect();
    for path in &collision_paths {
        fixture.seed(path.clone(), b"existing collision remains byte exact");
    }
    let nonce = AtomicU64::new(0);
    let mut notes = Vec::new();
    let (path, file) = super::open_first_available_log_file_with_nonce(
        "runtime",
        vec![
            LogDirectoryCandidate {
                source: "noncollision-directory-error",
                path: blocked,
            },
            LogDirectoryCandidate {
                source: "collision-cap",
                path: collisions.clone(),
            },
            LogDirectoryCandidate {
                source: "fallback",
                path: fallback.clone(),
            },
        ],
        true,
        &mut notes,
        &nonce,
    );
    let path = path.unwrap();
    fixture.remember_file(path.clone(), b"fresh fallback bytes");
    assert_eq!(path, fallback.join("runtime.log"));
    let mut file = file.unwrap();
    file.write_all(b"fresh fallback bytes").unwrap();
    file.flush().unwrap();
    drop(file);
    assert_eq!(nonce.load(Ordering::Relaxed), 32);
    assert!(notes
        .iter()
        .any(|note| note.message.contains("log directory candidate failed")));
    assert!(notes
        .iter()
        .any(|note| note.message.contains("collision retry limit (32) reached")));
    assert_eq!(
        std::fs::read(&sentinel).unwrap(),
        b"existing generation remains byte exact"
    );
    for path in &collision_paths {
        assert_eq!(
            std::fs::read(path).unwrap(),
            b"existing collision remains byte exact"
        );
    }
    assert_eq!(std::fs::read(&path).unwrap(), b"fresh fallback bytes");

    let capped_nonce = AtomicU64::new(0);
    let (path, file) = super::open_first_available_log_file_with_nonce(
        "runtime",
        vec![LogDirectoryCandidate {
            source: "all-collided",
            path: collisions,
        }],
        true,
        &mut notes,
        &capped_nonce,
    );
    assert!(path.is_none() && file.is_none());
    assert_eq!(capped_nonce.load(Ordering::Relaxed), 32);
    assert!(notes
        .last()
        .unwrap()
        .message
        .contains("no file-backed log sink available"));

    let exhausted = fixture.directory("exhausted");
    let exhausted_sentinel = exhausted.join("runtime.log");
    fixture.seed(
        exhausted_sentinel.clone(),
        b"exhausted nonce cannot reopen this file",
    );
    let exhaustion_fallback = fixture.directory("exhaustion-fallback");
    let exhausted_nonce = AtomicU64::new(u64::MAX);
    let (path, file) = super::open_first_available_log_file_with_nonce(
        "runtime",
        vec![
            LogDirectoryCandidate {
                source: "nonce-exhaustion",
                path: exhausted,
            },
            LogDirectoryCandidate {
                source: "after-exhaustion",
                path: exhaustion_fallback.clone(),
            },
        ],
        true,
        &mut notes,
        &exhausted_nonce,
    );
    let path = path.unwrap();
    fixture.remember_file(path.clone(), b"");
    drop(file.unwrap());
    assert_eq!(path, exhaustion_fallback.join("runtime.log"));
    assert_eq!(exhausted_nonce.load(Ordering::Relaxed), u64::MAX);
    assert!(notes
        .iter()
        .any(|note| note.message.contains("diagnostic log file nonce exhausted")));
    assert_eq!(
        std::fs::read(&exhausted_sentinel).unwrap(),
        b"exhausted nonce cannot reopen this file"
    );

    let long_directory = fixture.directory("long-channel");
    let long_channel = "x".repeat(80);
    fixture.seed(
        long_directory.join(format!("{long_channel}.log")),
        b"old long channel",
    );
    let (path, file) = super::open_first_available_log_file_with_nonce(
        &long_channel,
        vec![LogDirectoryCandidate {
            source: "bounded-suffix",
            path: long_directory,
        }],
        true,
        &mut notes,
        &AtomicU64::new(0),
    );
    let path = path.unwrap();
    fixture.remember_file(path.clone(), b"");
    drop(file.unwrap());
    assert!(path.file_name().unwrap().to_str().unwrap().len() <= 64);
    assert!(path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with(&"x".repeat(32)));
    fixture
        .seal()
        .expect("seal known seed and output byte plans after every fallback assertion");
    fixture
        .cleanup()
        .expect("clean only the fully checked identity, byteLen and BLAKE3 manifest");
}
