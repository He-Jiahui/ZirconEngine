use super::AccountError;
use openidconnect::url::Url;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountConfig {
    pub issuer: String,
    pub client_id: String,
    pub service_url: String,
    pub callback_port: u16,
    pub allow_loopback_http: bool,
    /// Optional path for the non-secret pending-operation journal.
    #[serde(default)]
    pub operation_journal_path: Option<PathBuf>,
}

impl AccountConfig {
    pub fn load(path: &Path) -> Result<Self, AccountError> {
        let bytes = crate::file_io::read_bounded_regular(path, 65536)
            .map_err(|_| AccountError::Configuration)?;
        let config: Self =
            serde_json::from_slice(&bytes).map_err(|_| AccountError::Configuration)?;
        config.endpoint(&config.issuer)?;
        config.endpoint(&config.service_url)?;
        if config.callback_port < 1024
            || config.client_id.is_empty()
            || config.client_id.len() > 256
        {
            return Err(AccountError::Configuration);
        }
        if config.operation_journal_path.is_some() {
            config.journal_path()?;
        }
        Ok(config)
    }

    pub fn endpoint(&self, value: &str) -> Result<Url, AccountError> {
        let url = Url::parse(value).map_err(|_| AccountError::Configuration)?;
        let local = url
            .host_str()
            .and_then(|host| host.parse::<std::net::IpAddr>().ok())
            .is_some_and(|ip| ip.is_loopback());
        if !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.query().is_some()
            || !(url.scheme() == "https"
                || (self.allow_loopback_http && local && url.scheme() == "http"))
        {
            return Err(AccountError::Configuration);
        }
        Ok(url)
    }

    pub fn redirect(&self) -> String {
        format!("http://127.0.0.1:{}/callback", self.callback_port)
    }

    pub fn journal_path(&self) -> Result<PathBuf, AccountError> {
        if let Some(path) = self.operation_journal_path.as_ref() {
            if !path.is_absolute()
                || path.file_name().is_none()
                || path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
            {
                return Err(AccountError::Configuration);
            }
            return Ok(path.clone());
        }
        if let Some(base) = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .filter(|base| base.is_absolute())
        {
            return Ok(base
                .join("ZirconHub")
                .join("Account")
                .join("operations.dat"));
        }
        Err(AccountError::OperationStore)
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
