use super::AccountError;
use serde::{Deserialize, Serialize};

const CREDENTIAL_BLOB_BYTES: usize = 5 * 512;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedSession {
    pub issuer: String,
    pub subject: String,
    pub nonce: String,
    pub refresh_token: String,
    #[serde(default)]
    pub revoke_only: bool,
}

pub trait CredentialStore: Send + Sync {
    fn load(&self) -> Result<Option<SavedSession>, AccountError>;
    fn save(
        &self,
        session: &SavedSession,
        expected: Option<&SavedSession>,
    ) -> Result<(), AccountError>;
    fn clear(&self, expected: Option<&SavedSession>) -> Result<(), AccountError>;
    fn begin_logout(&self) -> Result<Option<SavedSession>, AccountError>;
    fn pending_revocation(&self) -> Result<Option<SavedSession>, AccountError>;
    fn finish_revocation(&self, saved: &SavedSession) -> Result<(), AccountError>;
}

pub struct WindowsCredentials {
    entry: keyring::Entry,
}

impl WindowsCredentials {
    pub fn new(issuer: &str, client: &str) -> Result<Self, AccountError> {
        if !cfg!(windows) {
            return Err(AccountError::CredentialStore);
        }
        let key =
            serde_json::to_string(&(issuer, client)).map_err(|_| AccountError::Configuration)?;
        Ok(Self {
            entry: keyring::Entry::new("ZirconEngine.Hub.OIDC", &key)
                .map_err(|_| AccountError::CredentialStore)?,
        })
    }

    fn read(&self) -> Result<Option<SavedSession>, AccountError> {
        match self.entry.get_secret() {
            Ok(bytes) => decode(&bytes).map(Some),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(AccountError::CredentialStore),
        }
    }

    fn write(&self, session: &SavedSession) -> Result<(), AccountError> {
        // Store UTF-8 secret bytes. set_password expands to UTF-16 and halves usable capacity.
        let bytes = encode(session)?;
        self.entry
            .set_secret(&bytes)
            .map_err(|_| AccountError::CredentialStore)
    }

    fn delete(&self) -> Result<(), AccountError> {
        match self.entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(AccountError::CredentialStore),
        }
    }
}

impl CredentialStore for WindowsCredentials {
    fn load(&self) -> Result<Option<SavedSession>, AccountError> {
        let _guard = super::operations::credential_lock()?;
        Ok(self.read()?.filter(|session| !session.revoke_only))
    }
    fn save(
        &self,
        session: &SavedSession,
        expected: Option<&SavedSession>,
    ) -> Result<(), AccountError> {
        let _guard = super::operations::credential_lock()?;
        let current = self.read()?;
        if current.as_ref().is_some_and(|saved| saved.revoke_only) {
            return Err(AccountError::RevocationPending);
        }
        if expected.is_some_and(|expected| {
            !current.as_ref().is_some_and(|current| {
                current.refresh_token == expected.refresh_token
                    && current.issuer == expected.issuer
                    && current.subject == expected.subject
            })
        }) {
            return Err(AccountError::Cancelled);
        }
        self.write(session)
    }
    fn clear(&self, expected: Option<&SavedSession>) -> Result<(), AccountError> {
        let _guard = super::operations::credential_lock()?;
        let current = self.read()?;
        if current.as_ref().is_some_and(|saved| saved.revoke_only) {
            return Ok(());
        }
        if expected.is_some_and(|expected| {
            !current.as_ref().is_some_and(|current| {
                current.refresh_token == expected.refresh_token
                    && current.issuer == expected.issuer
                    && current.subject == expected.subject
            })
        }) {
            return Ok(());
        }
        self.delete()
    }
    fn begin_logout(&self) -> Result<Option<SavedSession>, AccountError> {
        let _guard = super::operations::credential_lock()?;
        let Some(mut saved) = self.read()? else {
            return Ok(None);
        };
        if !saved.revoke_only {
            saved.revoke_only = true;
            saved.nonce.clear();
            // Replacing false with true cannot enlarge a current credential blob.
            // A crash now leaves a retry-only token, never a refreshable local session.
            self.write(&saved)?;
        }
        Ok(Some(saved))
    }
    fn pending_revocation(&self) -> Result<Option<SavedSession>, AccountError> {
        let _guard = super::operations::credential_lock()?;
        Ok(self.read()?.filter(|saved| saved.revoke_only))
    }
    fn finish_revocation(&self, revoked: &SavedSession) -> Result<(), AccountError> {
        let _guard = super::operations::credential_lock()?;
        if self.read()?.is_some_and(|saved| {
            saved.revoke_only
                && saved.refresh_token == revoked.refresh_token
                && saved.issuer == revoked.issuer
                && saved.subject == revoked.subject
        }) {
            self.delete()?;
        }
        Ok(())
    }
}

fn encode(session: &SavedSession) -> Result<Vec<u8>, AccountError> {
    let bytes = serde_json::to_vec(session).map_err(|_| AccountError::CredentialStore)?;
    if bytes.len() > CREDENTIAL_BLOB_BYTES {
        return Err(AccountError::CredentialStore);
    }
    Ok(bytes)
}

fn decode(bytes: &[u8]) -> Result<SavedSession, AccountError> {
    if bytes.len() > CREDENTIAL_BLOB_BYTES {
        return Err(AccountError::CredentialStore);
    }
    if let Ok(saved) = serde_json::from_slice(bytes) {
        return Ok(saved);
    }
    // Read the existing set_password UTF-16 representation once; subsequent writes use UTF-8.
    if bytes.len() % 2 != 0 {
        return Err(AccountError::CredentialStore);
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let text = String::from_utf16(&units).map_err(|_| AccountError::CredentialStore)?;
    serde_json::from_str(&text).map_err(|_| AccountError::CredentialStore)
}

#[cfg(test)]
#[path = "tests/credential.rs"]
mod tests;
