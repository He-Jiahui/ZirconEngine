use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldRuntimeExtensionError {
    diagnostic: String,
}

impl WorldRuntimeExtensionError {
    pub fn duplicate_registration(key: &str) -> Self {
        Self::new(format!("duplicate world runtime extension `{key}`"))
    }

    pub fn registration_failed(key: &str, diagnostic: impl fmt::Display) -> Self {
        Self::new(format!(
            "world runtime extension `{key}` failed: {diagnostic}"
        ))
    }

    pub fn new(diagnostic: impl Into<String>) -> Self {
        Self {
            diagnostic: diagnostic.into(),
        }
    }

    pub fn diagnostic(&self) -> &str {
        &self.diagnostic
    }
}

impl fmt::Display for WorldRuntimeExtensionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic)
    }
}

impl std::error::Error for WorldRuntimeExtensionError {}
