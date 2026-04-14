use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::path::{Component, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetUriScheme {
    Res,
    Library,
}

impl AssetUriScheme {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "res" => Some(Self::Res),
            "lib" => Some(Self::Library),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Res => "res",
            Self::Library => "lib",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssetUri {
    scheme: AssetUriScheme,
    path: String,
}

impl AssetUri {
    pub fn parse(value: &str) -> Result<Self, AssetUriError> {
        let Some((scheme, raw_path)) = value.split_once("://") else {
            return Err(AssetUriError::MissingScheme(value.to_string()));
        };
        let Some(scheme) = AssetUriScheme::parse(scheme) else {
            return Err(AssetUriError::UnsupportedScheme(scheme.to_string()));
        };
        let path = normalize_asset_path(raw_path)?;
        Ok(Self { scheme, path })
    }

    pub fn scheme(&self) -> AssetUriScheme {
        self.scheme
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Display for AssetUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.scheme.as_str(), self.path)
    }
}

impl Serialize for AssetUri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AssetUri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetUriError {
    MissingScheme(String),
    UnsupportedScheme(String),
    EmptyPath,
    EscapeAttempt(String),
}

impl Display for AssetUriError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingScheme(value) => write!(f, "asset uri is missing scheme: {value}"),
            Self::UnsupportedScheme(value) => write!(f, "unsupported asset uri scheme: {value}"),
            Self::EmptyPath => write!(f, "asset uri path cannot be empty"),
            Self::EscapeAttempt(value) => write!(f, "asset uri escapes project root: {value}"),
        }
    }
}

impl std::error::Error for AssetUriError {}

fn normalize_asset_path(path: &str) -> Result<String, AssetUriError> {
    let path = path.replace('\\', "/");
    let mut normalized = Vec::new();

    for component in Path::new(&path).components() {
        match component {
            Component::Normal(segment) => normalized.push(segment.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized.pop().is_none() {
                    return Err(AssetUriError::EscapeAttempt(path));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AssetUriError::EscapeAttempt(path));
            }
        }
    }

    if normalized.is_empty() {
        return Err(AssetUriError::EmptyPath);
    }

    Ok(normalized.join("/"))
}
