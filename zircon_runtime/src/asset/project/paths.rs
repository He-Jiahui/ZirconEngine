use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::RelPath;

/// Canonical file name for a Zircon project manifest.
pub const PROJECT_MANIFEST_FILE: &str = "zircon-project.toml";

/// A project filesystem path after its physical identity has been resolved.
///
/// The operation path is the only path suitable for filesystem reads and writes. The display
/// path is an explicitly lossy view for diagnostics and platform APIs that do not accept Windows
/// verbatim paths. Keeping both views together prevents callers from re-resolving aliases or
/// stripping Windows prefixes themselves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedProjectPath {
    operation_path: PathBuf,
    display_path: PathBuf,
}

impl ResolvedProjectPath {
    fn from_operational_path(operation_path: PathBuf) -> Self {
        let display_path = ProjectPaths::display_path(&operation_path);
        Self {
            operation_path,
            display_path,
        }
    }

    /// Physical path for filesystem operations. Do not convert it to a display path before I/O.
    pub fn operation_path(&self) -> &Path {
        &self.operation_path
    }

    /// Human-readable path for diagnostics and external platform APIs.
    pub fn display_path(&self) -> &Path {
        &self.display_path
    }

    pub fn into_operation_path(self) -> PathBuf {
        self.operation_path
    }

    /// Formats an operational diagnostic through this path's display view.
    ///
    /// Type-erased errors can retain the physical Windows path used for I/O. Keep the conversion
    /// here so product entry points do not each learn Windows verbatim-path normalization.
    pub fn display_diagnostic(&self, diagnostic: impl std::fmt::Display) -> String {
        diagnostic.to_string().replace(
            self.operation_path.to_string_lossy().as_ref(),
            self.display_path.to_string_lossy().as_ref(),
        )
    }

    /// Derives a sibling path without allowing operation and display views to drift apart.
    pub fn with_file_name(&self, file_name: impl AsRef<std::ffi::OsStr>) -> Self {
        let file_name = file_name.as_ref();
        Self {
            operation_path: self.operation_path.with_file_name(file_name),
            display_path: self.display_path.with_file_name(file_name),
        }
    }

    /// Derives the parent directory without re-resolving the physical identity.
    ///
    /// This is for boundaries that accept either a project directory or an already resolved
    /// project manifest. Both views move together so callers never need platform-specific
    /// prefix handling when normalizing the input shape.
    pub fn parent(&self) -> Option<Self> {
        Some(Self {
            operation_path: self.operation_path.parent()?.to_path_buf(),
            display_path: self.display_path.parent()?.to_path_buf(),
        })
    }
}

impl std::fmt::Display for ResolvedProjectPath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_path.display().fmt(formatter)
    }
}

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
    /// Returns whether a path names the project manifest at a process boundary.
    ///
    /// Windows filename comparison belongs to the resolver so callers can accept a project
    /// directory or manifest input without learning platform-specific case behavior.
    pub fn is_project_manifest_path(path: impl AsRef<Path>) -> bool {
        let Some(file_name) = path.as_ref().file_name() else {
            return false;
        };
        #[cfg(windows)]
        {
            return windows_os_str_equals_ascii_case_insensitive(file_name, PROJECT_MANIFEST_FILE);
        }
        #[cfg(not(windows))]
        {
            file_name == std::ffi::OsStr::new(PROJECT_MANIFEST_FILE)
        }
    }

    /// Returns whether an existing path is the project manifest file at an input boundary.
    ///
    /// A directory named `zircon-project.toml` remains a valid project-root input. Keeping this
    /// distinction with the filename rules prevents entry points from duplicating path-shape
    /// compatibility logic.
    pub fn is_project_manifest_file(path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        path.is_file() && Self::is_project_manifest_path(path)
    }

    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let resolved_root = Self::resolve_path(root)?;
        Ok(Self::from_resolved_root(&resolved_root))
    }

    /// Derives project-owned paths from an already resolved physical root.
    ///
    /// Callers that resolved an input once must retain that identity through project opening
    /// instead of converting it back into a raw path and resolving it again.
    pub fn from_resolved_root(root: &ResolvedProjectPath) -> Self {
        Self::from_operation_root(root.operation_path().to_path_buf())
    }

    fn from_operation_root(root: PathBuf) -> Self {
        let derived_root = root.join(".zircon");
        let cache_root = derived_root.join("cache");
        Self {
            manifest: root.join(PROJECT_MANIFEST_FILE),
            asset_artifact_root: cache_root.join("assets"),
            registry_root: derived_root.join("registry"),
            autosave_root: derived_root.join("autosave"),
            play_root: derived_root.join("play"),
            thumbnails_root: derived_root.join("thumbnails"),
            cache_root,
            derived_root,
            root,
        }
    }

    /// Resolves a project path to one physical identity before project-owned paths are derived.
    ///
    /// Existing aliases, including Windows junctions, SUBST drives, and symlinks, are resolved by
    /// the filesystem. For a new project target, the deepest existing ancestor is resolved and
    /// the uncreated tail is preserved so creation remains rooted in the same physical location.
    /// Windows drive-relative paths such as `C:project` and root-relative paths such as
    /// `\project` are rejected because they do not select a stable physical project identity.
    pub fn resolve_path(root: impl AsRef<Path>) -> Result<ResolvedProjectPath, std::io::Error> {
        let absolute = absolute_project_path(root.as_ref())?;
        let Some((existing, unresolved_tail)) =
            split_at_deepest_existing_project_ancestor(&absolute)
        else {
            return Ok(ResolvedProjectPath::from_operational_path(absolute));
        };

        let mut resolved = canonicalize_physical_path(&existing)?;
        for name in unresolved_tail {
            append_uncreated_project_component(&mut resolved, &name);
        }
        Ok(ResolvedProjectPath::from_operational_path(resolved))
    }

    /// Resolves a normal relative path from an already-resolved physical base.
    ///
    /// This retains the base identity selected by the caller instead of falling back to the
    /// process working directory. It is appropriate for product-owned paths such as a staged
    /// library beside an executable; project-owned asset paths should continue to use `RelPath`.
    pub fn resolve_path_from(
        base: &ResolvedProjectPath,
        relative: impl AsRef<Path>,
    ) -> Result<ResolvedProjectPath, std::io::Error> {
        let relative = relative.as_ref();
        if relative.is_absolute() || relative.has_root() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "relative path must not be rooted: {}",
                    Self::display_path(relative).display()
                ),
            ));
        }
        #[cfg(windows)]
        if is_windows_drive_relative(relative) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Windows relative paths must not be drive-relative: {}",
                    Self::display_path(relative).display()
                ),
            ));
        }
        Self::resolve_path(base.operation_path().join(relative))
    }

    /// Resolves a project path to the filesystem operation path retained by existing callers.
    pub fn resolve_root(root: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
        Self::resolve_path(root).map(ResolvedProjectPath::into_operation_path)
    }

    /// Resolves an existing file or directory to its physical identity.
    ///
    /// Unlike [`Self::resolve_root`], this rejects an uncreated path tail. Use it for existing
    /// asset roots and other paths whose identity must already be materialized on disk.
    pub fn resolve_existing(path: impl AsRef<Path>) -> Result<ResolvedProjectPath, std::io::Error> {
        let absolute = absolute_project_path(path.as_ref())?;
        canonicalize_physical_path(&absolute).map(ResolvedProjectPath::from_operational_path)
    }

    /// Resolves an existing project path to the filesystem operation path retained by existing
    /// callers. New consumers should use [`Self::resolve_existing`] to preserve both views.
    pub fn resolve_existing_path(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
        Self::resolve_existing(path).map(ResolvedProjectPath::into_operation_path)
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

    /// Returns a resolver-owned key for in-process synchronization of project files.
    ///
    /// This key is not an operation path and must never be used for I/O. It folds aliases and
    /// platform path equality into one hashable representation so callers do not reproduce
    /// Windows-specific separator or casing rules when coordinating writes.
    pub(crate) fn filesystem_identity_key(path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        let resolved = Self::resolve_path(path)
            .map(ResolvedProjectPath::into_operation_path)
            .unwrap_or_else(|_| absolute_project_path(path).unwrap_or_else(|_| path.to_path_buf()));

        #[cfg(windows)]
        {
            return PathBuf::from(resolved.to_string_lossy().replace('/', "\\").to_lowercase());
        }
        #[cfg(not(windows))]
        {
            resolved
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
                ProjectPaths::display_path(path).display()
            ),
        ));
    }
    #[cfg(windows)]
    if path.has_root() && !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Windows project paths must be drive-rooted, not root-relative: {}",
                ProjectPaths::display_path(path).display()
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
        if suffix.len() >= 2
            && ((b'A' as u16..=b'Z' as u16).contains(&suffix[0])
                || (b'a' as u16..=b'z' as u16).contains(&suffix[0]))
            && suffix[1] == b':' as u16
        {
            return PathBuf::from(std::ffi::OsString::from_wide(suffix));
        }
    }
    path
}

#[cfg(windows)]
fn wide_ascii_lowercase(value: u16) -> Option<u16> {
    const ASCII_UPPER_A: u16 = b'A' as u16;
    const ASCII_UPPER_Z: u16 = b'Z' as u16;
    const ASCII_LOWER_A: u16 = b'a' as u16;
    const ASCII_LOWER_Z: u16 = b'z' as u16;
    const ASCII_CASE_DELTA: u16 = ASCII_LOWER_A - ASCII_UPPER_A;

    if (ASCII_UPPER_A..=ASCII_UPPER_Z).contains(&value) {
        return Some(value + ASCII_CASE_DELTA);
    }
    if (ASCII_LOWER_A..=ASCII_LOWER_Z).contains(&value) {
        return Some(value);
    }
    None
}

#[cfg(windows)]
fn windows_os_str_equals_ascii_case_insensitive(value: &std::ffi::OsStr, expected: &str) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let value = value.encode_wide().collect::<Vec<_>>();
    let expected = expected.encode_utf16().collect::<Vec<_>>();
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
fn wide_starts_with_ascii_case_insensitive(path: &[u16], prefix: &[u16]) -> bool {
    path.get(..prefix.len()).is_some_and(|head| {
        head.iter().zip(prefix).all(|(actual, expected)| {
            actual == expected
                || matches!(
                    (wide_ascii_lowercase(*actual), wide_ascii_lowercase(*expected)),
                    (Some(actual), Some(expected)) if actual == expected
                )
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

    #[cfg(windows)]
    use super::wide_starts_with_ascii_case_insensitive;
    use super::{ProjectPaths, ResolvedProjectPath};

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
    fn filesystem_identity_key_resolves_an_uncreated_tail_below_a_directory_alias() {
        let parent = unique_temp_root("project-paths-identity-key");
        let physical = parent.join("physical-parent");
        fs::create_dir_all(&physical).unwrap();
        let alias = parent.join("parent-alias");
        create_directory_link(&physical, &alias);

        assert_eq!(
            ProjectPaths::filesystem_identity_key(alias.join("assets/cube.obj.meta")),
            ProjectPaths::filesystem_identity_key(physical.join("assets/cube.obj.meta"))
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
