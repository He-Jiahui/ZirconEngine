use std::error::Error;
use std::fmt;

const MAX_NAMESPACE_BYTES: usize = 128;
const MAX_KEY_BYTES: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreferenceKey {
    namespace: String,
    key: String,
}

impl PreferenceKey {
    pub fn new(
        namespace: impl Into<String>,
        key: impl Into<String>,
    ) -> Result<Self, PreferenceKeyError> {
        let namespace = namespace.into();
        let key = key.into();
        validate_component(&namespace, true)?;
        validate_component(&key, false)?;
        Ok(Self { namespace, key })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

fn validate_component(value: &str, namespace: bool) -> Result<(), PreferenceKeyError> {
    if value.is_empty() {
        return Err(PreferenceKeyError::new(if namespace {
            PreferenceKeyErrorKind::EmptyNamespace
        } else {
            PreferenceKeyErrorKind::EmptyKey
        }));
    }
    if value.contains('\0') {
        return Err(PreferenceKeyError::new(PreferenceKeyErrorKind::ContainsNul));
    }
    let limit = if namespace {
        MAX_NAMESPACE_BYTES
    } else {
        MAX_KEY_BYTES
    };
    if value.len() > limit {
        return Err(PreferenceKeyError::new(if namespace {
            PreferenceKeyErrorKind::NamespaceTooLong
        } else {
            PreferenceKeyErrorKind::KeyTooLong
        }));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreferenceKeyErrorKind {
    EmptyNamespace,
    EmptyKey,
    ContainsNul,
    NamespaceTooLong,
    KeyTooLong,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceKeyError {
    kind: PreferenceKeyErrorKind,
}

impl PreferenceKeyError {
    const fn new(kind: PreferenceKeyErrorKind) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> PreferenceKeyErrorKind {
        self.kind
    }
}

impl fmt::Display for PreferenceKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid preference key: {:?}", self.kind)
    }
}

impl Error for PreferenceKeyError {}
