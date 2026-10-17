use std::path::Path;

/// A UTF-8, slash-separated path inside an engine asset root.
///
/// Parsing preserves the spelling and rejects roots, drive/stream colons, backslashes,
/// empty segments, dot segments and control characters before any root is inspected.
/// This lexical contract does not replace filesystem identity or secure-open checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetRelativePath<'a>(&'a Path);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AssetRelativePathError {
    #[error("asset-relative path is empty; use the explicit root value for a root query")]
    Empty,
    #[error("asset-relative path contains a root")]
    Rooted,
    #[error("asset-relative path contains a backslash")]
    Backslash,
    #[error("asset-relative path contains a drive or stream colon")]
    Colon,
    #[error("asset-relative path contains an empty segment")]
    EmptySegment,
    #[error("asset-relative path contains a current-directory segment")]
    CurrentDirectory,
    #[error("asset-relative path contains a parent-directory segment")]
    ParentDirectory,
    #[error("asset-relative path contains a control character")]
    ControlCharacter,
    #[error("asset-relative path is not UTF-8")]
    NonUtf8,
    #[error("legacy asset path must begin with the relative assets/ prefix")]
    MissingAssetsPrefix,
}

impl<'a> AssetRelativePath<'a> {
    pub fn parse<P: AsRef<Path> + ?Sized>(path: &'a P) -> Result<Self, AssetRelativePathError> {
        let path = path.as_ref();
        let value = path.to_str().ok_or(AssetRelativePathError::NonUtf8)?;
        if value.is_empty() {
            return Err(AssetRelativePathError::Empty);
        }
        if value.starts_with('/') {
            return Err(AssetRelativePathError::Rooted);
        }
        if value.contains('\\') {
            return Err(AssetRelativePathError::Backslash);
        }
        if value.contains(':') {
            return Err(AssetRelativePathError::Colon);
        }
        for segment in value.split('/') {
            match segment {
                "" => return Err(AssetRelativePathError::EmptySegment),
                "." => return Err(AssetRelativePathError::CurrentDirectory),
                ".." => return Err(AssetRelativePathError::ParentDirectory),
                _ => {}
            }
            if segment.chars().any(char::is_control) {
                return Err(AssetRelativePathError::ControlCharacter);
            }
        }
        Ok(Self(path))
    }

    /// Explicit compatibility for exactly one rootless assets/ prefix.
    pub fn from_assets_prefixed<P: AsRef<Path> + ?Sized>(
        path: &'a P,
    ) -> Result<Self, AssetRelativePathError> {
        let value = path
            .as_ref()
            .to_str()
            .ok_or(AssetRelativePathError::NonUtf8)?;
        let relative = value
            .strip_prefix("assets/")
            .ok_or(AssetRelativePathError::MissingAssetsPrefix)?;
        Self::parse(relative)
    }

    /// A deliberate root query; strict parsing never accepts an empty path.
    pub fn root() -> Self {
        Self(Path::new(""))
    }

    pub fn as_path(self) -> &'a Path {
        self.0
    }
}
