#[cfg(windows)]
use std::collections::BTreeSet;
use std::fs;
#[cfg(windows)]
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(windows)]
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use super::{
    wide_ascii_lowercase, wide_starts_with_ascii_case_insensitive,
    windows_os_str_equals_ascii_case_insensitive,
};
use super::{ProjectPaths, ResolvedProjectPath, ResolvedProjectPathIdentity};

static NEXT_TEMP_PROJECT: AtomicU64 = AtomicU64::new(1);
#[cfg(windows)]
const SAMPLE_PAIRS: usize = 21;
#[cfg(windows)]
const MANIFEST_NAME_CHECKS_PER_SAMPLE: usize = 131_072;

#[cfg(any(unix, windows))]
#[test]
fn from_root_resolves_an_existing_directory_alias_to_its_physical_identity() {
    let parent = unique_temp_root("project-paths-alias");
    let physical = parent.join("physical-project");
    fs::create_dir_all(&physical).unwrap();
    let alias = parent.join("project-alias");
    create_directory_link(&physical, &alias);

    let paths = ProjectPaths::from_root(&alias).unwrap();

    assert_eq!(
        paths.root(),
        ProjectPaths::resolve_existing_path(&physical).unwrap()
    );
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(any(unix, windows))]
#[test]
fn resolve_root_preserves_an_uncreated_tail_below_a_directory_alias() {
    let parent = unique_temp_root("project-paths-uncreated-tail");
    let physical = parent.join("physical-parent");
    fs::create_dir_all(&physical).unwrap();
    let alias = parent.join("parent-alias");
    create_directory_link(&physical, &alias);

    let resolved = ProjectPaths::resolve_root(alias.join("new-project")).unwrap();

    assert_eq!(
        resolved,
        ProjectPaths::resolve_existing_path(&physical)
            .unwrap()
            .join("new-project")
    );
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(any(unix, windows))]
#[test]
fn resolve_path_from_uses_the_resolved_base_identity_for_relative_paths() {
    let parent = unique_temp_root("project-paths-relative-base");
    let physical = parent.join("physical-product");
    fs::create_dir_all(&physical).unwrap();
    let alias = parent.join("product-alias");
    create_directory_link(&physical, &alias);

    let base = ProjectPaths::resolve_existing(&alias).unwrap();
    let resolved = ProjectPaths::resolve_path_from(&base, "plugins/runtime.dll").unwrap();

    assert_eq!(
        resolved.operation_path(),
        ProjectPaths::resolve_existing_path(&physical)
            .unwrap()
            .join("plugins/runtime.dll")
    );
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(any(unix, windows))]
#[test]
fn resolved_identity_resolves_an_uncreated_tail_below_a_directory_alias() {
    let parent = unique_temp_root("project-paths-resolved-identity");
    let physical = parent.join("physical-parent");
    fs::create_dir_all(&physical).unwrap();
    let alias = parent.join("parent-alias");
    create_directory_link(&physical, &alias);

    assert_eq!(
        ProjectPaths::resolve_identity(alias.join("assets/cube.obj.meta")).unwrap(),
        ProjectPaths::resolve_identity(physical.join("assets/cube.obj.meta")).unwrap()
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn resolved_identity_containment_uses_component_boundaries() {
    let root = unique_temp_root("project-paths-identity-containment");
    fs::create_dir_all(root.join("assets")).unwrap();
    let root_identity = ProjectPaths::resolve_identity(&root).unwrap();
    let child_identity =
        ProjectPaths::resolve_identity(root.join("assets/panel.zui.zmeta")).unwrap();
    let lexical_prefix_identity = ProjectPaths::resolve_identity(root.with_file_name(format!(
        "{}-backup",
        root.file_name().unwrap().to_string_lossy()
    )))
    .unwrap();

    assert!(child_identity.is_within(&root_identity));
    assert_eq!(
        child_identity.relative_to(&root_identity),
        Some(PathBuf::from("assets/panel.zui.zmeta"))
    );
    assert!(!lexical_prefix_identity.is_within(&root_identity));
    assert!(!root_identity.is_within(&child_identity));

    fs::remove_dir_all(root).unwrap();
}

#[cfg(any(unix, windows))]
#[test]
fn resolve_identity_rejects_a_broken_symlink_in_the_uncreated_tail() {
    let root = unique_temp_root("project-paths-broken-link");
    fs::create_dir_all(&root).unwrap();
    let broken = root.join("broken-alias");
    create_directory_link(&root.join("missing-target"), &broken);

    let error = ProjectPaths::resolve_identity(broken.join("asset.zmeta"))
        .expect_err("an existing alias with no physical target must fail closed");

    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn resolved_identity_orders_uncreated_case_aliases_as_one_path() {
    let root = unique_temp_root("project-paths-uncreated-case-identity");
    fs::create_dir_all(root.join("assets")).unwrap();

    let first = ResolvedProjectPathIdentity::from(
        ProjectPaths::resolve_path(root.join("assets/Panel.zui")).unwrap(),
    );
    let second = ResolvedProjectPathIdentity::from(
        ProjectPaths::resolve_path(root.join("ASSETS/panel.zui")).unwrap(),
    );
    let mut identities = BTreeSet::new();
    assert!(identities.insert(first));
    assert!(!identities.insert(second));
    assert_eq!(identities.len(), 1);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn resolve_root_normalizes_uncreated_dot_segments_before_deriving_project_paths() {
    let root = unique_temp_root("project-paths-dot-segments");
    let requested = root
        .join("uncreated-parent")
        .join("..")
        .join(".")
        .join("project");

    let resolved = ProjectPaths::resolve_root(&requested).unwrap();

    let physical_parent = ProjectPaths::resolve_existing_path(root.parent().unwrap()).unwrap();
    assert_eq!(
        resolved,
        physical_parent
            .join(root.file_name().unwrap())
            .join("project")
    );
    assert!(
        !resolved.components().any(|component| matches!(
            component,
            std::path::Component::CurDir | std::path::Component::ParentDir
        )),
        "resolved project identity must not retain lexical dot segments: {}",
        resolved.display()
    );
}

#[cfg(any(unix, windows))]
#[test]
fn resolve_root_normalizes_an_uncreated_dotdot_tail_after_resolving_a_directory_alias() {
    let parent = unique_temp_root("project-paths-alias-dotdot-tail");
    let physical = parent.join("physical-parent");
    fs::create_dir_all(&physical).unwrap();
    let alias = parent.join("parent-alias");
    create_directory_link(&physical, &alias);

    let resolved = ProjectPaths::resolve_root(
        alias
            .join("uncreated-parent")
            .join("..")
            .join("new-project"),
    )
    .unwrap();

    assert_eq!(
        resolved,
        ProjectPaths::resolve_existing_path(&physical)
            .unwrap()
            .join("new-project")
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn resolve_existing_path_rejects_an_uncreated_tail() {
    let parent = unique_temp_root("project-paths-existing");
    fs::create_dir_all(&parent).unwrap();

    let error = ProjectPaths::resolve_existing_path(parent.join("missing-project"))
        .expect_err("existing project paths must not preserve an uncreated tail");

    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(windows)]
#[test]
fn from_root_resolves_a_subst_drive_to_its_physical_identity() {
    let parent = unique_temp_root("project-paths-subst");
    let physical = parent.join("physical-project");
    fs::create_dir_all(&physical).unwrap();
    let mut subst = SubstDrive::mount(&physical);

    let paths = ProjectPaths::from_root(subst.path()).unwrap();

    assert_eq!(
        paths.root(),
        ProjectPaths::resolve_existing_path(&physical).unwrap()
    );
    drop(paths);
    subst.unmount();
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(windows)]
#[test]
fn resolve_root_rejects_a_drive_relative_project_path() {
    let error = ProjectPaths::resolve_root(r"C:ambiguous-project-root").unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}

#[cfg(windows)]
#[test]
fn top_level_project_resolvers_reject_root_relative_paths() {
    for path in [
        Path::new(r"\ambiguous-project-root"),
        Path::new("/ambiguous-project-root"),
    ] {
        for resolution in [
            ProjectPaths::resolve_path(path).map(|_| ()),
            ProjectPaths::resolve_root(path).map(|_| ()),
            ProjectPaths::resolve_existing(path).map(|_| ()),
        ] {
            let error = resolution.expect_err(
                "Windows root-relative project paths must not depend on the current drive",
            );
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        }
    }
}

#[cfg(windows)]
#[test]
fn resolve_path_from_rejects_rooted_and_drive_relative_paths() {
    let base = ProjectPaths::resolve_existing(std::env::temp_dir()).unwrap();

    for path in [
        Path::new(r"C:ambiguous-runtime-library.dll"),
        Path::new(r"\ambiguous-runtime-library.dll"),
        Path::new("/ambiguous-runtime-library.dll"),
    ] {
        let error = ProjectPaths::resolve_path_from(&base, path)
            .expect_err("relative path resolution must reject ambiguous Windows path forms");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }
}

#[cfg(windows)]
#[test]
fn drive_relative_project_errors_display_verbatim_roots_without_prefixes() {
    let error = ProjectPaths::resolve_root(r"\\?\C:ambiguous-project-root").unwrap_err();

    assert_eq!(
        error.to_string(),
        "Windows project paths must be drive-rooted, not drive-relative: C:ambiguous-project-root"
    );
}

#[cfg(windows)]
#[test]
fn normalize_windows_final_path_strips_supported_verbatim_prefixes() {
    assert_eq!(
        ProjectPaths::display_path(PathBuf::from(r"\\?\C:\projects\mvp")),
        PathBuf::from(r"C:\projects\mvp")
    );
    assert_eq!(
        ProjectPaths::display_path(PathBuf::from(r"\\?\UNC\server\share\projects\mvp")),
        PathBuf::from(r"\\server\share\projects\mvp")
    );
    assert_eq!(
        ProjectPaths::display_path(PathBuf::from(r"\\?\unc\server\share\projects\mvp")),
        PathBuf::from(r"\\server\share\projects\mvp")
    );
    assert_eq!(
        ProjectPaths::display_path(PathBuf::from(r"\\?\Volume{guid}\projects\mvp")),
        PathBuf::from(r"\\?\Volume{guid}\projects\mvp")
    );
}

#[test]
fn resolved_project_path_keeps_operation_and_display_views_separate() {
    #[cfg(windows)]
    let operation_path = PathBuf::from(r"\\?\C:\projects\mvp\assets\scenes\main.scene.toml");
    #[cfg(not(windows))]
    let operation_path = PathBuf::from("/projects/mvp/assets/scenes/main.scene.toml");

    let resolved = ResolvedProjectPath::from_operational_path(operation_path.clone());

    assert_eq!(resolved.operation_path(), operation_path);
    #[cfg(windows)]
    assert_eq!(
        resolved.display_path(),
        PathBuf::from(r"C:\projects\mvp\assets\scenes\main.scene.toml")
    );
    #[cfg(not(windows))]
    assert_eq!(resolved.display_path(), operation_path);
    #[cfg(windows)]
    assert_eq!(
        resolved.to_string(),
        r"C:\projects\mvp\assets\scenes\main.scene.toml"
    );
    #[cfg(not(windows))]
    assert_eq!(
        resolved.to_string(),
        "/projects/mvp/assets/scenes/main.scene.toml"
    );
}

#[test]
fn project_manifest_path_identification_is_owned_by_the_resolver() {
    assert!(ProjectPaths::is_project_manifest_path(Path::new(
        "zircon-project.toml"
    )));
    assert!(!ProjectPaths::is_project_manifest_path(Path::new(
        "zircon-project.backup.toml"
    )));

    #[cfg(windows)]
    assert!(ProjectPaths::is_project_manifest_path(Path::new(
        "ZIRCON-PROJECT.TOML"
    )));
}

#[cfg(windows)]
#[test]
fn project_manifest_name_comparison_preserves_ascii_case_and_non_ascii_exactness() {
    use std::ffi::OsStr;

    assert!(windows_os_str_equals_ascii_case_insensitive(
        OsStr::new("zircon-project.toml"),
        "zircon-project.toml"
    ));
    assert!(windows_os_str_equals_ascii_case_insensitive(
        OsStr::new("ZIRCON-PROJECT.TOML"),
        "zircon-project.toml"
    ));
    assert!(!windows_os_str_equals_ascii_case_insensitive(
        OsStr::new("zircon-project.toml.bak"),
        "zircon-project.toml"
    ));
    assert!(!windows_os_str_equals_ascii_case_insensitive(
        OsStr::new("zircon-proj\u{00e9}ct.toml"),
        "zircon-project.toml"
    ));
    assert!(windows_os_str_equals_ascii_case_insensitive(
        OsStr::new("zircon-proj\u{00e9}ct.toml"),
        "zircon-proj\u{00e9}ct.toml"
    ));
}

#[cfg(windows)]
#[test]
#[ignore = "release-only performance contract"]
fn benchmark_project_manifest_name_zero_allocation_comparison() {
    use std::ffi::OsStr;

    let value = OsStr::new("ZIRCON-PROJECT.TOML");
    let expected = "zircon-project.toml";
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_manifest_name_checks(
                legacy_manifest_name_equals,
                value,
                expected,
            ));
            optimized_raw.push(measure_manifest_name_checks(
                windows_os_str_equals_ascii_case_insensitive,
                value,
                expected,
            ));
        } else {
            optimized_raw.push(measure_manifest_name_checks(
                windows_os_str_equals_ascii_case_insensitive,
                value,
                expected,
            ));
            legacy_raw.push(measure_manifest_name_checks(
                legacy_manifest_name_equals,
                value,
                expected,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "zero-allocation project manifest name comparison must improve P95 by at least 50%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    println!(
            "PERF_RESULT task=plugins07_zero_allocation_project_manifest_name sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank checks_per_sample={MANIFEST_NAME_CHECKS_PER_SAMPLE} legacy_allocations_per_check=2 optimized_allocations_per_check=0 threshold_percent=50 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
}

#[cfg(windows)]
fn legacy_manifest_name_equals(value: &std::ffi::OsStr, expected: &str) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let value = black_box(value.encode_wide().collect::<Vec<_>>());
    let expected = black_box(expected.encode_utf16().collect::<Vec<_>>());
    value.len() == expected.len()
        && value.iter().zip(expected).all(|(actual, expected)| {
            actual == &expected
                || matches!(
                    (wide_ascii_lowercase(*actual), wide_ascii_lowercase(expected)),
                    (Some(actual), Some(expected)) if actual == expected
                )
        })
}

#[cfg(windows)]
fn measure_manifest_name_checks(
    predicate: fn(&std::ffi::OsStr, &str) -> bool,
    value: &std::ffi::OsStr,
    expected: &str,
) -> u64 {
    let started = Instant::now();
    let mut matches = 0;
    for _ in 0..MANIFEST_NAME_CHECKS_PER_SAMPLE {
        matches += usize::from(predicate(black_box(value), black_box(expected)));
    }
    black_box(matches);
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(windows)]
fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[cfg(windows)]
fn raw_samples(samples: &[u64]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn project_paths_derive_from_a_resolved_root_without_changing_its_operation_identity() {
    #[cfg(windows)]
    let operation_path = PathBuf::from(r"\\?\C:\projects\mvp");
    #[cfg(not(windows))]
    let operation_path = PathBuf::from("/projects/mvp");

    let resolved = ResolvedProjectPath::from_operational_path(operation_path.clone());
    let paths = ProjectPaths::from_resolved_root(&resolved);

    assert_eq!(paths.root(), operation_path);
}

#[test]
fn resolved_project_path_derives_sibling_views_together() {
    #[cfg(windows)]
    let operation_path = PathBuf::from(r"\\?\C:\projects\mvp\evidence\editor.png");
    #[cfg(not(windows))]
    let operation_path = PathBuf::from("/projects/mvp/evidence/editor.png");

    let resolved = ResolvedProjectPath::from_operational_path(operation_path);
    let staging = resolved.with_file_name("editor.png.partial-1");

    #[cfg(windows)]
    assert_eq!(
        staging.operation_path(),
        PathBuf::from(r"\\?\C:\projects\mvp\evidence\editor.png.partial-1")
    );
    #[cfg(windows)]
    assert_eq!(
        staging.display_path(),
        PathBuf::from(r"C:\projects\mvp\evidence\editor.png.partial-1")
    );
    #[cfg(not(windows))]
    assert_eq!(
        staging.operation_path(),
        PathBuf::from("/projects/mvp/evidence/editor.png.partial-1")
    );
    #[cfg(not(windows))]
    assert_eq!(
        staging.display_path(),
        PathBuf::from("/projects/mvp/evidence/editor.png.partial-1")
    );
}

#[test]
fn resolved_project_path_derives_parent_views_together_without_resolving_again() {
    #[cfg(windows)]
    let operation_path = PathBuf::from(r"\\?\C:\projects\mvp\zircon-project.toml");
    #[cfg(not(windows))]
    let operation_path = PathBuf::from("/projects/mvp/zircon-project.toml");

    let resolved = ResolvedProjectPath::from_operational_path(operation_path);
    let parent = resolved
        .parent()
        .expect("project manifest should have a parent directory");

    #[cfg(windows)]
    assert_eq!(
        parent.operation_path(),
        PathBuf::from(r"\\?\C:\projects\mvp")
    );
    #[cfg(windows)]
    assert_eq!(parent.display_path(), PathBuf::from(r"C:\projects\mvp"));
    #[cfg(not(windows))]
    assert_eq!(parent.operation_path(), PathBuf::from("/projects/mvp"));
    #[cfg(not(windows))]
    assert_eq!(parent.display_path(), PathBuf::from("/projects/mvp"));
}

#[test]
fn resolved_project_path_formats_diagnostics_through_its_display_view() {
    #[cfg(windows)]
    let operation_path = PathBuf::from(r"\\?\C:\projects\mvp");
    #[cfg(not(windows))]
    let operation_path = PathBuf::from("/projects/mvp");

    let resolved = ResolvedProjectPath::from_operational_path(operation_path);
    let diagnostic = resolved.display_diagnostic(format!(
        "project manifest is missing: {}\\zircon-project.toml",
        resolved.operation_path().display()
    ));

    #[cfg(windows)]
    assert_eq!(
        diagnostic,
        r"project manifest is missing: C:\projects\mvp\zircon-project.toml"
    );
    #[cfg(not(windows))]
    assert_eq!(
        diagnostic,
        "project manifest is missing: /projects/mvp\\zircon-project.toml"
    );
}

#[cfg(windows)]
#[test]
fn wide_prefix_comparison_folds_ascii_utf16_units_only() {
    assert!(wide_starts_with_ascii_case_insensitive(
        &[
            b'\\' as u16,
            b'\\' as u16,
            b'?' as u16,
            b'\\' as u16,
            b'u' as u16,
            b'n' as u16,
            b'c' as u16,
            b'\\' as u16,
            b's' as u16,
        ],
        &[
            b'\\' as u16,
            b'\\' as u16,
            b'?' as u16,
            b'\\' as u16,
            b'U' as u16,
            b'N' as u16,
            b'C' as u16,
            b'\\' as u16,
        ],
    ));
    assert!(!wide_starts_with_ascii_case_insensitive(
        &[0x00e9],
        &[0x00c9],
    ));
}

fn unique_temp_root(label: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let sequence = NEXT_TEMP_PROJECT.fetch_add(1, Ordering::Relaxed);
    test_output_root().join(format!(
        "zircon_project_paths_{label}_{timestamp}_{sequence}"
    ))
}

fn test_output_root() -> PathBuf {
    std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("resolve current workspace for project-path test output")
                .join("target")
        })
        .join("zircon-test-output")
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create project-path alias fixture");
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for project-path alias fixture");
    assert!(
        output.status.success(),
        "create project-path junction fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(windows)]
struct SubstDrive {
    drive: String,
    root: PathBuf,
    mounted: bool,
}

#[cfg(windows)]
impl SubstDrive {
    fn mount(target: &Path) -> Self {
        for letter in b'D'..=b'Z' {
            let drive = format!("{}:", char::from(letter));
            let root = PathBuf::from(format!("{drive}\\"));
            if root.exists() {
                continue;
            }
            let output = std::process::Command::new("subst")
                .arg(&drive)
                .arg(target)
                .output()
                .expect("start SUBST for project-path fixture");
            if output.status.success() {
                return Self {
                    drive,
                    root,
                    mounted: true,
                };
            }
        }
        panic!("reserve a free SUBST drive for project-path fixture");
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn unmount(&mut self) {
        let output = std::process::Command::new("subst")
            .arg(&self.drive)
            .arg("/D")
            .output()
            .expect("start SUBST fixture cleanup");
        assert!(
            output.status.success(),
            "remove SUBST fixture {} failed: {}",
            self.drive,
            String::from_utf8_lossy(&output.stderr)
        );
        self.mounted = false;
    }
}

#[cfg(windows)]
impl Drop for SubstDrive {
    fn drop(&mut self) {
        if self.mounted {
            let _ = std::process::Command::new("subst")
                .arg(&self.drive)
                .arg("/D")
                .output();
        }
    }
}
