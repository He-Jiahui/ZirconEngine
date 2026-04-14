use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::path::{Component, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceScheme {
    Res,
    Library,
    Builtin,
    Memory,
}

impl ResourceScheme {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "res" => Some(Self::Res),
            "lib" => Some(Self::Library),
            "builtin" => Some(Self::Builtin),
            "mem" => Some(Self::Memory),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Res => "res",
            Self::Library => "lib",
            Self::Builtin => "builtin",
            Self::Memory => "mem",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceLocator {
    scheme: ResourceScheme,
    path: String,
    label: Option<String>,
}

impl ResourceLocator {
    pub fn parse(value: &str) -> Result<Self, ResourceLocatorError> {
        let Some((scheme, remainder)) = value.split_once("://") else {
            return Err(ResourceLocatorError::MissingScheme(value.to_string()));
        };
        let Some(scheme) = ResourceScheme::parse(scheme) else {
            return Err(ResourceLocatorError::UnsupportedScheme(scheme.to_string()));
        };
        let (path, label) = split_label(remainder)?;
        Self::new(scheme, path, label)
    }

    pub fn new(
        scheme: ResourceScheme,
        path: impl Into<String>,
        label: Option<String>,
    ) -> Result<Self, ResourceLocatorError> {
        let raw_path = path.into();
        let normalized_path = normalize_resource_path(&raw_path)?;
        let normalized_label = match label {
            Some(value) if value.is_empty() => return Err(ResourceLocatorError::EmptyLabel),
            Some(value) => Some(value),
            None => None,
        };
        Ok(Self {
            scheme,
            path: normalized_path,
            label: normalized_label,
        })
    }

    pub fn scheme(&self) -> ResourceScheme {
        self.scheme
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

impl Display for ResourceLocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.scheme.as_str(), self.path)?;
        if let Some(label) = &self.label {
            write!(f, "#{label}")?;
        }
        Ok(())
    }
}

impl Serialize for ResourceLocator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceLocator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceLocatorError {
    MissingScheme(String),
    UnsupportedScheme(String),
    EmptyPath,
    EscapeAttempt(String),
    EmptyLabel,
}

impl Display for ResourceLocatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingScheme(value) => write!(f, "resource locator is missing scheme: {value}"),
            Self::UnsupportedScheme(value) => write!(f, "unsupported resource scheme: {value}"),
            Self::EmptyPath => write!(f, "resource locator path cannot be empty"),
            Self::EscapeAttempt(value) => write!(f, "resource locator escapes root: {value}"),
            Self::EmptyLabel => write!(f, "resource locator label cannot be empty"),
        }
    }
}

impl std::error::Error for ResourceLocatorError {}

fn split_label(value: &str) -> Result<(String, Option<String>), ResourceLocatorError> {
    match value.split_once('#') {
        Some((_path, label)) if label.is_empty() => Err(ResourceLocatorError::EmptyLabel),
        Some((path, label)) => Ok((path.to_string(), Some(label.to_string()))),
        None => Ok((value.to_string(), None)),
    }
}

fn normalize_resource_path(path: &str) -> Result<String, ResourceLocatorError> {
    let original = path.replace('\\', "/");
    let mut normalized = Vec::new();

    for component in Path::new(&original).components() {
        match component {
            Component::Normal(segment) => normalized.push(segment.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized.pop().is_none() {
                    return Err(ResourceLocatorError::EscapeAttempt(original));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(ResourceLocatorError::EscapeAttempt(original));
            }
        }
    }

    if normalized.is_empty() {
        return Err(ResourceLocatorError::EmptyPath);
    }

    Ok(normalized.join("/"))
}
