use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::RelPath;

mod identity;
#[cfg(windows)]
mod windows;

pub use identity::ResolvedProjectPathIdentity;

#[cfg(windows)]
use windows::{
    is_windows_drive_relative, normalize_windows_final_path, wide_ascii_lowercase,
    wide_starts_with_ascii_case_insensitive, windows_os_str_equals_ascii_case_insensitive,
    windows_paths_equal_ignore_case,
};

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
            split_at_deepest_existing_project_ancestor(&absolute)?
        else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "project path has no accessible physical ancestor: {}",
                    Self::display_path(&absolute).display()
                ),
            ));
        };

        let mut resolved = canonicalize_physical_path(&existing)?;
        for name in unresolved_tail {
            append_uncreated_project_component(&mut resolved, &name);
        }
        Ok(ResolvedProjectPath::from_operational_path(resolved))
    }

    /// Resolves a path into the single ordered identity used by project coordination code.
    ///
    /// Resolution is deliberately fallible: admission, deduplication, and recovery validation
    /// must not replace an unresolved physical identity with a lexical or lossy string key.
    pub fn resolve_identity(
        path: impl AsRef<Path>,
    ) -> Result<ResolvedProjectPathIdentity, std::io::Error> {
        Self::resolve_path(path).map(ResolvedProjectPathIdentity::from)
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
) -> Result<Option<(PathBuf, Vec<std::ffi::OsString>)>, std::io::Error> {
    let mut candidate = PathBuf::new();
    let mut deepest_existing = None;
    let mut unresolved_tail = Vec::new();

    for component in path.components() {
        candidate.push(component.as_os_str());
        match fs::metadata(&candidate) {
            Ok(_) => {
                deepest_existing = Some(candidate.clone());
                unresolved_tail.clear();
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match fs::symlink_metadata(&candidate) {
                    Ok(_) => return Err(error),
                    Err(link_error) if link_error.kind() == std::io::ErrorKind::NotFound => {
                        unresolved_tail.push(component.as_os_str().to_os_string());
                    }
                    Err(link_error) => return Err(link_error),
                }
            }
            Err(error) => return Err(error),
        }
    }

    Ok(deepest_existing.map(|existing| (existing, unresolved_tail)))
}

fn canonicalize_physical_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    fs::canonicalize(path)
}

#[cfg(test)]
mod tests;
