use super::{AccountError, InstallRequest, PackageClient};
use serde_json::Value;
use std::{ffi::OsString, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

const MAX_REPLY: usize = 65536;
const DEADLINE: Duration = Duration::from_secs(60);

pub(super) struct Session {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    #[cfg(windows)]
    _inputs: Vec<super::windows::VerifiedInput>,
    #[cfg(windows)]
    policy_index: Option<super::windows::VerifiedInput>,
}

impl Session {
    pub(super) async fn spawn(client: &PackageClient) -> Result<Self, AccountError> {
        #[cfg(not(windows))]
        {
            let _ = client;
            Err(AccountError::PackageUnavailable)
        }
        #[cfg(windows)]
        {
            let (root, owner) = super::windows::root_identity(&client.root)?;
            if owner != client.owner {
                return Err(AccountError::PackageUnavailable);
            }
            let mut inputs = vec![
                root,
                super::windows::verified_input(
                    &client.config.executable,
                    &client.config.executable_sha256,
                    128 * 1024 * 1024,
                )?,
            ];
            let policy_index = Some(super::windows::verified_input(
                &client.policy_index_path,
                &client.policy_index_digest,
                65536,
            )?);
            let mut command = Command::new(&client.config.executable);
            command
                .args(helper_policy_arguments(
                    &client.policy_index_path,
                    &client.policy_index_digest,
                ))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .creation_flags(0x08000000);
            if let Some(library) = &client.config.runtime_library {
                inputs.push(super::windows::verified_input(
                    &library.path,
                    &library.sha256,
                    128 * 1024 * 1024,
                )?);
                let mut paths = vec![library
                    .path
                    .parent()
                    .ok_or(AccountError::PackageUnavailable)?
                    .to_path_buf()];
                paths.extend(std::env::split_paths(
                    &std::env::var_os("PATH").unwrap_or_default(),
                ));
                command.env(
                    "PATH",
                    std::env::join_paths(paths).map_err(|_| AccountError::PackageUnavailable)?,
                );
            }
            let mut child = command
                .spawn()
                .map_err(|_| AccountError::PackageUnavailable)?;
            let input = child.stdin.take().ok_or(AccountError::PackageUnavailable)?;
            let output = BufReader::new(
                child
                    .stdout
                    .take()
                    .ok_or(AccountError::PackageUnavailable)?,
            );
            Ok(Self {
                child,
                input,
                output,
                _inputs: inputs,
                policy_index,
            })
        }
    }

    async fn write_header(&mut self, value: &Value) -> Result<(), AccountError> {
        let mut bytes = serde_json::to_vec(value).map_err(|_| AccountError::ServiceFailure)?;
        if bytes.len() >= MAX_REPLY {
            return Err(AccountError::ServiceFailure);
        }
        bytes.push(b'\n');
        self.input
            .write_all(&bytes)
            .await
            .map_err(|_| AccountError::PackageUnavailable)
    }

    async fn response(&mut self) -> Result<Value, AccountError> {
        read_response(&mut self.output).await
    }

    pub(super) async fn prepare(
        &mut self,
        request: &InstallRequest,
        bytes: Vec<u8>,
    ) -> Result<Value, AccountError> {
        tokio::time::timeout(DEADLINE, async {
            self.write_header(&serde_json::json!({"action":"install","request":request}))
                .await?;
            // An existing receipt can arrive before the helper consumes the archive.
            let (written, response) = tokio::join!(
                self.input.write_all(&bytes),
                read_response(&mut self.output)
            );
            let response = response?;
            #[cfg(windows)]
            drop(self.policy_index.take());
            let status = response.get("status").and_then(Value::as_str);
            if status == Some("committed") {
                return Ok(response);
            }
            if written.is_err()
                || status != Some("prepared")
                || response.get("operationId").and_then(Value::as_str)
                    != Some(request.operation_id.as_str())
            {
                return Err(AccountError::PackageUnavailable);
            }
            Ok(response)
        })
        .await
        .map_err(|_| AccountError::Timeout)?
    }

    pub(super) async fn authorize_commit(&mut self) -> Result<(), AccountError> {
        tokio::time::timeout(DEADLINE, self.input.write_all(b"commit\n"))
            .await
            .map_err(|_| AccountError::OutcomeUnknown)?
            .map_err(|_| AccountError::OutcomeUnknown)
    }

    pub(super) async fn committed(
        &mut self,
        commit_possible: &mut bool,
    ) -> Result<Value, AccountError> {
        let response = match tokio::time::timeout(DEADLINE, self.response()).await {
            Ok(Ok(response)) => response,
            Ok(Err(
                error @ (AccountError::PackageTrust
                | AccountError::PackagePolicyUnconfigured
                | AccountError::PackageTargetUnconfigured
                | AccountError::PackageCapacity
                | AccountError::PackageConflict
                | AccountError::ServiceFailure
                | AccountError::Busy),
            )) => {
                // These helper errors precede its durable transaction; storage/transport errors do not prove rejection.
                *commit_possible = false;
                return Err(error);
            }
            _ => return Err(AccountError::OutcomeUnknown),
        };
        if response.get("status").and_then(Value::as_str) != Some("committed") {
            return Err(AccountError::OutcomeUnknown);
        }
        Ok(response)
    }

    pub(super) async fn close(&mut self) {
        let _ = self.child.start_kill();
        let _ = tokio::time::timeout(Duration::from_secs(5), self.child.wait()).await;
    }
}

async fn read_response(
    input: &mut (impl tokio::io::AsyncBufRead + Unpin),
) -> Result<Value, AccountError> {
    let mut line = Vec::new();
    input
        .take((MAX_REPLY + 1) as u64)
        .read_until(b'\n', &mut line)
        .await
        .map_err(|_| AccountError::PackageUnavailable)?;
    if line.len() > MAX_REPLY || line.last() != Some(&b'\n') {
        return Err(AccountError::PackageUnavailable);
    }
    let value: Value =
        serde_json::from_slice(&line).map_err(|_| AccountError::PackageUnavailable)?;
    if value.get("status").and_then(Value::as_str) == Some("failed") {
        return Err(helper_error(value.get("error").and_then(Value::as_str)));
    }
    Ok(value)
}

#[cfg(test)]
#[path = "tests/process.rs"]
mod tests;

pub(super) async fn query(client: &PackageClient, request: Value) -> Result<Value, AccountError> {
    #[cfg(test)]
    super::record_package_helper_query();

    let mut session = Session::spawn(client).await?;
    let result = tokio::time::timeout(DEADLINE, async {
        session.write_header(&request).await?;
        let response = session.response().await?;
        if response.get("status").and_then(Value::as_str) != Some("ready") {
            return Err(AccountError::PackageUnavailable);
        }
        Ok(response)
    })
    .await
    .map_err(|_| AccountError::Timeout)
    .and_then(|result| result);
    session.close().await;
    result
}

fn helper_policy_arguments(path: &std::path::Path, digest: &str) -> [OsString; 2] {
    [path.as_os_str().to_owned(), OsString::from(digest)]
}

fn helper_error(error: Option<&str>) -> AccountError {
    match error {
        Some("package_trust_denied") => AccountError::PackageTrust,
        Some("package_policy_unconfigured") => AccountError::PackagePolicyUnconfigured,
        Some("package_target_unconfigured") => AccountError::PackageTargetUnconfigured,
        Some("package_input_invalid") => AccountError::ServiceFailure,
        Some("package_capacity_exceeded") => AccountError::PackageCapacity,
        Some("package_policy_conflict") => AccountError::PackageConflict,
        Some("package_operation_busy") => AccountError::Busy,
        Some("package_outcome_unknown") => AccountError::OutcomeUnknown,
        _ => AccountError::PackageUnavailable,
    }
}
