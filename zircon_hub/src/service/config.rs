use super::error::ServiceError;
use openidconnect::url::Url;
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};

pub(super) fn read_bounded_regular(
    path: &std::path::Path,
    limit: usize,
) -> Result<Vec<u8>, ServiceError> {
    crate::file_io::read_bounded_regular(path, limit).map_err(|_| ServiceError::Configuration)
}

pub(super) fn read_bounded(path: &std::path::Path, limit: usize) -> Result<Vec<u8>, ServiceError> {
    read_bounded_regular(path, limit)
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    pub bind: SocketAddr,
    pub database: PathBuf,
    pub issuer: String,
    pub audience: String,
    pub introspection_client_id: String,
    pub introspection_secret_file: PathBuf,
    pub allow_loopback_http: bool,
    pub cloud: super::cloud::CloudConfig,
    #[serde(default)]
    pub catalog_policy_file: Option<PathBuf>,
}

impl ServiceConfig {
    pub fn load(path: &std::path::Path) -> Result<Self, ServiceError> {
        let bytes = read_bounded(path, 65536)?;
        let config: Self =
            toml::from_str(std::str::from_utf8(&bytes).map_err(|_| ServiceError::Configuration)?)
                .map_err(|_| ServiceError::Configuration)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ServiceError> {
        if !self.bind.ip().is_loopback()
            || self.audience.trim().is_empty()
            || self.introspection_client_id.trim().is_empty()
            || !self.database.is_absolute()
            || !self.introspection_secret_file.is_absolute()
            || self
                .catalog_policy_file
                .as_ref()
                .is_some_and(|path| !path.is_absolute())
        {
            return Err(ServiceError::Configuration);
        }
        self.validate_endpoint(&self.issuer)?;
        self.cloud.validate()?;
        Ok(())
    }

    pub fn validate_endpoint(&self, endpoint: &str) -> Result<Url, ServiceError> {
        let url = Url::parse(endpoint).map_err(|_| ServiceError::Configuration)?;
        let loopback = url
            .host_str()
            .and_then(|host| host.parse::<std::net::IpAddr>().ok())
            .is_some_and(|address| address.is_loopback());
        if !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || !(url.scheme() == "https"
                || (self.allow_loopback_http && loopback && url.scheme() == "http"))
        {
            return Err(ServiceError::Configuration);
        }
        Ok(url)
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
