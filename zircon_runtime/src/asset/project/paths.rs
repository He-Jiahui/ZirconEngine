use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::RelPath;

/// Canonical project paths. All regenerable state lives below `.zircon`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectPaths {
    root: PathBuf,
    manifest: PathBuf,
    derived_root: PathBuf,
    cache_root: PathBuf,
    asset_artifact_root: PathBuf,
    registry_root: PathBuf,
    autosave_root: PathBuf,
    play_root: PathBuf,
    thumbnails_root: PathBuf,
}

impl ProjectPaths {
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = Self::resolve_root(root)?;
        let derived_root = root.join(".zircon");
        let cache_root = derived_root.join("cache");
        Ok(Self {
            manifest: root.join("zircon-project.toml"),
            asset_artifact_root: cache_root.join("assets"),
            registry_root: derived_root.join("registry"),
            autosave_root: derived_root.join("autosave"),
            play_root: derived_root.join("play"),
            thumbnails_root: derived_root.join("thumbnails"),
            cache_root,
            derived_root,
            root,
        })
    }

    /// Resolves a project path to one physical identity before project-owned paths are derived.
    ///
    /// Existing aliases, including Windows junctions, SUBST drives, and symlinks, are resolved by
    /// the filesystem. For a new project target, the deepest existing ancestor is resolved and
    /// the uncreated tail is preserved so creation remains rooted in the same physical location.
    /// Windows drive-relative paths such as `C:project` are rejected because their per-drive
    /// working directory is not a stable project identity.
    pub fn resolve_root(root: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
        let absolute = absolute_project_path(root.as_ref())?;
        let Some((existing, unresolved_tail)) =
            split_at_deepest_existing_project_ancestor(&absolute)
        else {
            return Ok(absolute);
        };

        let mut resolved = canonicalize_physical_path(&existing)?;
        for name in unresolved_tail {
            append_uncreated_project_component(&mut resolved, &name);
        }
        Ok(resolved)
    }

    /// Resolves an existing file or directory to its physical identity.
    ///
    /// Unlike [`Self::resolve_root`], this rejects an uncreated path tail. Use it for existing
    /// asset roots and other paths whose identity must already be materialized on disk.
    pub fn resolve_existing_path(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
        let absolute = absolute_project_path(path.as_ref())?;
        canonicalize_physical_path(&absolute)
    }

    /// Compares two unresolved user paths without probing or flattening uncreated components.
    ///
    /// This is appropriate for validating two declared output targets before either is created.
    /// Existing-file identity remains the caller's responsibility, because lexical equality
    /// cannot establish hard-link or reparse-point identity.
    pub fn same_lexical_path(
        left: impl AsRef<Path>,
        right: impl AsRef<Path>,
    ) -> Result<bool, std::io::Error> {
        let left = absolute_project_path(left.as_ref())?;
        let right = absolute_project_path(right.as_ref())?;
        #[cfg(windows)]
        {
            let left = normalize_windows_final_path(left);
            let right = normalize_windows_final_path(right);
            Ok(windows_paths_equal_ignore_case(&left, &right))
        }
        #[cfg(not(windows))]
        {
            Ok(left == right)
        }
    }

    /// Returns a diagnostic-safe representation of a path without changing its operational form.
    ///
    /// On Windows, supported verbatim DOS and UNC prefixes are removed for logs and external
    /// tooling. Callers must keep the original resolved path for filesystem operations, because
    /// some valid Windows paths require verbatim semantics.
    pub fn display_path(path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        #[cfg(windows)]
        {
            normalize_windows_final_path(path.to_path_buf())
        }
        #[cfg(not(windows))]
        {
            path.to_path_buf()
        }
    }

    pub fn ensure_layout(&self, asset_roots: &[RelPath]) -> Result<(), std::io::Error> {
        self.ensure_derived_layout()?;
        self.ensure_asset_roots(asset_roots)
    }

    pub fn ensure_derived_layout(&self) -> Result<(), std::io::Error> {
        for root in [
            &self.cache_root,
            &self.asset_artifact_root,
            &self.registry_root,
            &self.autosave_root,
            &self.play_root,
            &self.thumbnails_root,
        ] {
            fs::create_dir_all(root)?;
        }
        Ok(())
    }

    pub fn ensure_asset_roots(&self, roots: &[RelPath]) -> Result<(), std::io::Error> {
        for root in roots {
            fs::create_dir_all(root.join_to(&self.root))?;
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest
    }

    pub fn asset_root(&self, relative: &RelPath) -> PathBuf {
        relative.join_to(&self.root)
    }

    pub fn derived_root(&self) -> &Path {
        &self.derived_root
    }

    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    pub fn asset_artifact_root(&self) -> &Path {
        &self.asset_artifact_root
    }

    pub fn registry_root(&self) -> &Path {
        &self.registry_root
    }

    pub fn autosave_root(&self) -> &Path {
        &self.autosave_root
    }

    pub fn play_root(&self) -> &Path {
        &self.play_root
    }

    pub fn thumbnails_root(&self) -> &Path {
        &self.thumbnails_root
    }
}

fn absolute_project_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    #[cfg(windows)]
    if is_windows_drive_relative(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Windows project paths must be drive-rooted, not drive-relative: {}",
                path.display()
            ),
        ));
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

/// Appends an uncreated path component after the existing ancestor has been resolved.
///
/// Dot segments must be applied only after resolving the existing prefix. In particular,
/// resolving `junction/..` lexically before that step would incorrectly use the junction's
/// textual parent instead of the physical target's parent.
fn append_uncreated_project_component(root: &mut PathBuf, component: &std::ffi::OsStr) {
    if component == std::ffi::OsStr::new(".") {
        return;
    }
    if component == std::ffi::OsStr::new("..") {
        let _ = root.pop();
        return;
    }
    root.push(component);
}

/// Splits an absolute path after its deepest existing prefix without flattening links.
///
/// `Path::components` safely removes `.` segments but intentionally retains `..`. Scanning the
/// candidates before canonicalization therefore preserves the filesystem semantics of an
/// existing junction or symlink, while returning the remaining lexical tail for creation paths.
fn split_at_deepest_existing_project_ancestor(
    path: &Path,
) -> Option<(PathBuf, Vec<std::ffi::OsString>)> {
    let mut candidate = PathBuf::new();
    let mut deepest_existing = None;
    let mut unresolved_tail = Vec::new();

    for component in path.components() {
        candidate.push(component.as_os_str());
        if candidate.exists() {
            deepest_existing = Some(candidate.clone());
            unresolved_tail.clear();
        } else {
            unresolved_tail.push(component.as_os_str().to_os_string());
        }
    }

    deepest_existing.map(|existing| (existing, unresolved_tail))
}

fn canonicalize_physical_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    fs::canonicalize(path)
}

#[cfg(windows)]
fn is_windows_drive_relative(path: &Path) -> bool {
    matches!(
        path.components().next(),
        Some(std::path::Component::Prefix(prefix))
            if matches!(
                prefix.kind(),
                std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)
            )
    ) && !path.has_root()
}

#[cfg(windows)]
fn normalize_windows_final_path(path: PathBuf) -> PathBuf {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    const VERBATIM_PREFIX: &[u16] = &[b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16];
    const VERBATIM_UNC_PREFIX: &[u16] = &[
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
        b'U' as u16,
        b'N' as u16,
        b'C' as u16,
        b'\\' as u16,
    ];

    let wide = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if wide_starts_with_ascii_case_insensitive(&wide, VERBATIM_UNC_PREFIX) {
        let mut normalized = vec![b'\\' as u16, b'\\' as u16];
        normalized.extend_from_slice(&wide[VERBATIM_UNC_PREFIX.len()..]);
        return PathBuf::from(std::ffi::OsString::from_wide(&normalized));
    }
    if wide.starts_with(VERBATIM_PREFIX) {
        let suffix = &wide[VERBATIM_PREFIX.len()..];
        if suffix.len() >= 3
            && ((b'A' as u16..=b'Z' as u16).contains(&suffix[0])
                || (b'a' as u16..=b'z' as u16).contains(&suffix[0]))
            && suffix[1] == b':' as u16
            && suffix[2] == b'\\' as u16
        {
            return PathBuf::from(std::ffi::OsString::from_wide(suffix));
        }
    }
    path
}

#[cfg(windows)]
fn wide_starts_with_ascii_case_insensitive(path: &[u16], prefix: &[u16]) -> bool {
    path.get(..prefix.len()).is_some_and(|head| {
        head.iter().zip(prefix).all(|(actual, expected)| {
            actual == expected
                || (actual.is_ascii_alphabetic()
                    && expected.is_ascii_alphabetic()
                    && actual.to_ascii_lowercase() == expected.to_ascii_lowercase())
        })
    })
}

#[cfg(windows)]
fn windows_paths_equal_ignore_case(left: &Path, right: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    const CSTR_EQUAL: i32 = 2;
    let left = left.as_os_str().encode_wide().collect::<Vec<_>>();
    let right = right.as_os_str().encode_wide().collect::<Vec<_>>();
    let Ok(left_length) = i32::try_from(left.len()) else {
        return false;
    };
    let Ok(right_length) = i32::try_from(right.len()) else {
        return false;
    };

    unsafe {
        CompareStringOrdinal(left.as_ptr(), left_length, right.as_ptr(), right_length, 1)
            == CSTR_EQUAL
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn CompareStringOrdinal(
        left: *const u16,
        left_length: i32,
        right: *const u16,
        right_length: i32,
        ignore_case: i32,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::ProjectPaths;

    static NEXT_TEMP_PROJECT: AtomicU64 = AtomicU64::new(1);

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

    fn unique_temp_root(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_TEMP_PROJECT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "zircon_project_paths_{label}_{timestamp}_{sequence}"
        ))
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
}
