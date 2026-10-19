//! 为结构守卫定位运行时源码与仓库文档；读取失败会直接使消费它的测试失败。
pub(super) fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    assert_contains_all_exact(label, source, required);
}

pub(super) fn assert_contains_all_exact(label: &str, source: &str, required: &[&str]) {
    let missing: Vec<_> = required
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(
        missing.is_empty(),
        "{label} missing required anchors: {missing:?}"
    );
}

pub(super) fn runtime_src_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .join("src")
            .join(relative)
    } else {
        std::path::PathBuf::from("zircon_runtime")
            .join("src")
            .join(relative)
    }
}

pub(super) fn repo_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .parent()
            .expect("zircon_runtime manifest should live under repository root")
            .join(relative)
    } else {
        std::path::PathBuf::from(relative)
    }
}

pub(crate) fn priority_plan_doc_current_owner_archive_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        [
            "docs/plans/zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-inventory.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-02.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-03.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-04.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-05.md",
        ]
        .into_iter()
        .map(read_output_archive)
        .collect::<Vec<_>>()
        .join("\n")
    })
}

// Only these index02 identities are accepted by the shared cached entry point.
struct FixedHistoryDocument {
    relative: &'static str,
    bytes: u64,
    sha256: &'static str,
    source: std::sync::OnceLock<(std::path::PathBuf, String)>,
}

impl FixedHistoryDocument {
    const fn new(relative: &'static str, bytes: u64, sha256: &'static str) -> Self {
        Self {
            relative,
            bytes,
            sha256,
            source: std::sync::OnceLock::new(),
        }
    }
}

static FIXED_HISTORY_DOCUMENTS: [FixedHistoryDocument; 30] = [
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md", 7312, "4b0bb9e0d2a03b17a55ead4bafe405fa360a592e62132bda8389d5e9c3e4bd47"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-inventory.md", 212136, "94efa6039b38cccc01b733ee3b4f4e49e76abaca66f219ccfd4a7731eeab41d7"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-02.md", 38560, "df6ea7ccbdf8127d389de984f71a58ba9e2e0037aef6b808ff4f0306498ece16"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-03.md", 14197, "514d36b2e2e3a833086c56dae896c334cb0d169218b6c8fe0d2980aa9b0a6fac"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-04.md", 22229, "1ffe3b5ad4932fc0d377df18690bd33ebc065e85bd10a631fbb614dea54f2e16"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-05.md", 40555, "abf268edbd1196c091a9bb68263e99ea3ff7438b6ef93f5289d68b3cf6910c08"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md", 136846, "f5106b752d3826e197d6e13aa95bca11b382dbf6efbb409822bb296ae24e1c7b"),
    FixedHistoryDocument::new("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md", 2049600, "d0731c18edcd3427d0a422aee748cf4e4802fe98c961abd5be83bd562a28e14b"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/13/2026-07-09-script-binding-and-reflection-output-records.md", 20414, "0645d5c5588f2e0fd0ebeecac121d8fe7d0e7ecd2220899bc11a77d3519a2b6f"),
    FixedHistoryDocument::new("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md", 4004187, "dfb46d8bd8c70523b184fd88699b5cec4d7bbae050f077e3db91c5dcb9814eb0"),
    FixedHistoryDocument::new("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md", 4290750, "7aa2d64892f05cfb7045b9f23df4b453e7d800c629ee4463051fdc8f9cfc3990"),
    FixedHistoryDocument::new("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md", 1642499, "c535d29fb4810580d7d361bc50d7ed29b5fa2dfcb8151a7fd5e92644e3b5997c"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md", 506947, "158a06dbb62f45278c68015e7f6fff6ed02c1f23f3326d334f3ddfc53410b228"),
    FixedHistoryDocument::new("docs/plans/zircon_plugins/12/2026-07-09-plugin-dx-and-structure-framework-output-records.md", 63746, "7c6858a5a9a9c88cc49d158cb6c7d8d9aa645a25d03bd1f26bb264fa584feb27"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md", 49040, "2dfeeb83daf87bfab5da5b4f00db7cdffb192cadedf85846ee07a6d6cb41360f"),
    FixedHistoryDocument::new("docs/plans/_archive/zircon_runtime/runtime/05/2026-07-09-scene-editor-boundary-closeout-output-records.md", 636701, "b6a57ef93e81a22bec882caf3ffd717b0fbb7e2f865227df68fa2e941143eb78"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/01/2026-07-09-tech-stack-and-dependency-governance-output-records.md", 39866, "9e9a19eb815442275984c10c05281b1e04bb50cd2eaaca9147deb8ffc7840b37"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/02/2026-07-09-core-spine-and-root-surface-output-records.md", 52995, "82f581fe32905b9ed5fa566583446726e080644d9982c8f07caa332712ad6b86"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/04/2026-07-09-asset-pipeline-alignment-output-records.md", 40147, "233c487873251be1a037b5a9a03676359270771f49bc4343e93293fe786098d2"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md", 42919, "73d6e8c4cab58e0f3e0ef27dfbfd2dd11e2f39099963ea09991a1c5a5046228b"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-index-output-records.md", 1626, "8710c13a8d7c2b6ad78b559a1aa7175af9fcf1509597b31f42bc6eaa91f4b640"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/08/2026-07-09-ecs-kernel-data-alignment-output-records.md", 43424, "afc8d6d07e65bf2bc9d6e7909469070f6199caedad7c64b05babf0be0bf875be"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/09/2026-07-09-ui-subsystem-architecture-output-records.md", 33828, "7e2b375c3289647586d26288cf2f05a972322fe8631182561914d2ce0b408bc0"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/11/2026-07-09-job-system-task-model-output-records.md", 52442, "758c465dfdb64844e16b418a87ca0008100c4bebb907e31368ca588902e92297"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/14/2026-07-09-runtime-module-family-closeout-output-records.md", 26820, "b34b37c70d79af1c7cbc2178fef9394874c5cebf90259a51f89fc27c5aaa4d46"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/12/2026-07-17-m5-input-event-bounds-current-source-closeout.md", 9157, "4c9924b96701c8cfe6954a0cf9475ca67e898f6f0ba06e3776b36b7f3fd4cf4b"),
    FixedHistoryDocument::new("versions/reconciliation-515/6b7c6e18f36ee98dafb0f0d44e06a4bf67db37edefbb5af501aeef63c1833fbf/docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md", 39953, "6b7c6e18f36ee98dafb0f0d44e06a4bf67db37edefbb5af501aeef63c1833fbf"),
    FixedHistoryDocument::new("versions/reconciliation-515/eba15ac42775b6ad479b3ccd5d90869900041fdf944d9b46cee303e21f8b7107/docs/plans/zircon_runtime/runtime/index.md", 66778, "eba15ac42775b6ad479b3ccd5d90869900041fdf944d9b46cee303e21f8b7107"),
    FixedHistoryDocument::new("docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md", 11495, "157c1f73e2264d57c6f03f8575d699c89bb078d03cfd462bf6ddc52646568214"),
    FixedHistoryDocument::new("versions/reconciliation-515/460103b9c9e99bf507b713237c0a9db858d312a1ba3d809dd8593ed65bbda728/docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md", 47203, "460103b9c9e99bf507b713237c0a9db858d312a1ba3d809dd8593ed65bbda728"),
];

// Each expansion owns its cache; arguments are immutable current literals or verified history.
#[macro_export]
macro_rules! runtime_test_history_concat {
    ($($part:expr),+ $(,)?) => {{
        static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let parts = [$($part),+];
        SOURCE.get_or_init(|| parts.concat()).as_str()
    }};
}

/// Frozen historical evidence, never a fallback for current source or acceptance.
pub(crate) fn fixed_history_archive_source(relative: &str) -> &'static str {
    let document = FIXED_HISTORY_DOCUMENTS
        .iter()
        .find(|entry| entry.relative == relative)
        .unwrap_or_else(|| panic!("unregistered fixed historical document: {relative}"));
    let configured =
        std::env::var_os("ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT").map(std::path::PathBuf::from);
    let root = checked_history_root(configured.as_deref())
        .unwrap_or_else(|error| panic!("fixed historical archive {relative}: {error}"));
    let (cached_root, source) = document.source.get_or_init(|| {
        let source =
            read_verified_history_at(Some(&root), relative, document.bytes, document.sha256)
                .unwrap_or_else(|error| panic!("fixed historical archive {relative}: {error}"));
        (root.clone(), source)
    });
    assert_eq!(
        cached_root, &root,
        "fixed historical archive root changed after cache initialization"
    );
    source
}

fn checked_history_root(root: Option<&std::path::Path>) -> Result<std::path::PathBuf, String> {
    let root = root.ok_or_else(|| "archive root not configured".to_string())?;
    if !root.is_absolute() {
        return Err("archive root must be absolute".to_string());
    }
    let root = root
        .canonicalize()
        .map_err(|error| format!("archive root unavailable: {error}"))?;
    if !root.is_dir() {
        return Err("archive root must be a directory".to_string());
    }
    Ok(root)
}

fn read_verified_history_at(
    root: Option<&std::path::Path>,
    relative: &str,
    expected_bytes: u64,
    expected_sha256: &str,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    if relative.is_empty()
        || relative.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.contains([':', '\\'])
        })
    {
        return Err("invalid archive relative path".to_string());
    }
    let root = checked_history_root(root)?;
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("archive file unavailable: {error}"))?;
    if !path.starts_with(&root) {
        return Err("archive file escapes root".to_string());
    }
    let file =
        std::fs::File::open(&path).map_err(|error| format!("archive read failed: {error}"))?;
    let mut payload = Vec::new();
    file.take(expected_bytes.saturating_add(1))
        .read_to_end(&mut payload)
        .map_err(|error| format!("archive read failed: {error}"))?;
    if payload.len() as u64 != expected_bytes {
        return Err(format!(
            "archive length mismatch: expected {expected_bytes}, got {}",
            payload.len()
        ));
    }
    let sha256 = format!("{:x}", Sha256::digest(&payload));
    if sha256 != expected_sha256 {
        return Err(format!(
            "archive SHA256 mismatch: expected {expected_sha256}, got {sha256}"
        ));
    }
    String::from_utf8(payload).map_err(|error| format!("archive UTF8 invalid: {error}"))
}

fn read_output_archive(relative: &str) -> String {
    fixed_history_archive_source(relative).to_owned()
}

#[cfg(test)]
mod fixed_history_tests {
    use super::read_verified_history_at;
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("zircon-fixed-history-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&root).expect("create owned history fixture in task TEMP");
            Self(root)
        }

        fn write(&self, payload: &[u8]) -> String {
            let path = self.0.join("docs/history.md");
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, payload).unwrap();
            format!("{:x}", Sha256::digest(payload))
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).expect("remove owned history fixture");
        }
    }

    #[test]
    fn fixed_history_preserves_verified_bytes() {
        let fixture = Fixture::new();
        let payload = b"fixed history\r\noriginal bytes\n";
        let hash = fixture.write(payload);
        let source = read_verified_history_at(
            Some(&fixture.0),
            "docs/history.md",
            payload.len() as u64,
            &hash,
        )
        .unwrap();
        assert_eq!(source.as_bytes(), payload);
    }

    #[test]
    fn fixed_history_requires_explicit_root_and_never_falls_back_to_checkout() {
        let fixture = Fixture::new();
        let hash = fixture.write(b"history");
        assert!(read_verified_history_at(None, "docs/history.md", 7, &hash)
            .unwrap_err()
            .contains("root not configured"));
        assert!(read_verified_history_at(
            Some(std::path::Path::new("relative")),
            "docs/history.md",
            7,
            &hash
        )
        .unwrap_err()
        .contains("must be absolute"));
        assert!(
            read_verified_history_at(Some(&fixture.0), "missing.md", 7, &hash)
                .unwrap_err()
                .contains("file unavailable")
        );
        // This path exists in the real checkout, but the explicit fixture root lacks it.
        assert!(
            read_verified_history_at(Some(&fixture.0), "Cargo.toml", 7, &hash)
                .unwrap_err()
                .contains("file unavailable")
        );
    }

    #[test]
    fn fixed_history_rejects_length_hash_and_utf8_corruption() {
        let fixture = Fixture::new();
        let hash = fixture.write(b"history");
        assert!(
            read_verified_history_at(Some(&fixture.0), "docs/history.md", 6, &hash)
                .unwrap_err()
                .contains("length mismatch")
        );
        fixture.write(b"tampered");
        assert!(
            read_verified_history_at(Some(&fixture.0), "docs/history.md", 7, &hash)
                .unwrap_err()
                .contains("length mismatch")
        );
        fixture.write(b"changed");
        assert!(
            read_verified_history_at(Some(&fixture.0), "docs/history.md", 7, &hash)
                .unwrap_err()
                .contains("SHA256 mismatch")
        );
        let hash = fixture.write(&[0xff]);
        assert!(
            read_verified_history_at(Some(&fixture.0), "docs/history.md", 1, &hash)
                .unwrap_err()
                .contains("UTF8 invalid")
        );
    }

    #[test]
    fn fixed_history_rejects_relative_path_escape() {
        let fixture = Fixture::new();
        for relative in [
            "../history.md",
            "/history.md",
            "docs/../history.md",
            "C:/history.md",
            "C:history.md",
            "docs\\history.md",
            "docs//history.md",
        ] {
            assert!(
                read_verified_history_at(Some(&fixture.0), relative, 0, "")
                    .unwrap_err()
                    .contains("invalid archive relative path"),
                "{relative}"
            );
        }
    }
}
