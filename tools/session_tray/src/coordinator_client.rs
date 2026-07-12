use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::recovery::RecoveryStatus;
use crate::runtime_descriptor::RuntimeDescriptor;
use crate::tray_state::SupervisionState;
use crate::TrayError;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blocker {
    pub kind: String,
    pub identity: String,
    pub status: String,
    pub blocking: bool,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisionHealth {
    pub repository_key: String,
    pub state: SupervisionState,
    pub explicit_stop: bool,
    pub maintenance_hold: bool,
    pub failure_count: u32,
    pub busy: bool,
    pub blockers: Vec<Blocker>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Health {
    pub status: String,
    pub repo_root: String,
    pub pid: u32,
    pub instance_id: String,
    pub process_creation_time: String,
    pub repository_key: String,
    pub schema_version: u32,
    pub control_api_versions: Vec<u32>,
    pub supervision_api_versions: Vec<u32>,
    pub supervision: SupervisionHealth,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRecord {
    pub action_id: String,
    pub kind: String,
    pub status: String,
    pub confirmation_phrase: Option<String>,
    #[serde(default)]
    pub impact: Option<Value>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    pub result: Option<Value>,
    pub error_code: Option<String>,
}

#[derive(Deserialize)]
struct Envelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<ControlError>,
}

#[derive(Deserialize)]
struct ControlError {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct ActionEnvelope {
    action: ActionRecord,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapTicket {
    pub bootstrap_path: String,
    pub expires_in_seconds: u32,
}

pub struct CoordinatorClient<'a> {
    descriptor: &'a RuntimeDescriptor,
    timeout: Duration,
}

impl<'a> CoordinatorClient<'a> {
    pub fn new(descriptor: &'a RuntimeDescriptor) -> Self {
        Self {
            descriptor,
            timeout: Duration::from_secs(3),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn health(&self) -> Result<Health, TrayError> {
        self.request("GET", "/health", None, false)
    }

    pub fn verify_health(&self) -> Result<Health, TrayError> {
        let health = self.health()?;
        if health.status != "ok"
            || health.pid != self.descriptor.pid
            || health.instance_id != self.descriptor.instance_id
            || health.process_creation_time != self.descriptor.process_creation_time
            || health.repository_key != self.descriptor.repository_key
            || health.supervision.repository_key != self.descriptor.repository_key
            || !health.control_api_versions.contains(&1)
            || !health.supervision_api_versions.contains(&1)
        {
            return Err(TrayError::IdentityMismatch(
                "authenticated health differs from runtime descriptor",
            ));
        }
        Ok(health)
    }

    pub fn issue_observer_ticket(&self) -> Result<BootstrapTicket, TrayError> {
        self.control_request(
            "POST",
            "/control/v1/bootstrap-tickets",
            Some(&json!({"actor": "zircon-session-tray", "role": "observer"})),
        )
    }

    pub fn preview_lifecycle(
        &self,
        kind: &str,
        timeout_seconds: u32,
    ) -> Result<ActionRecord, TrayError> {
        let response: ActionEnvelope = self.control_request(
            "POST",
            "/control/v1/actions/preview",
            Some(&json!({
                "kind": kind,
                "parameters": {"timeoutSeconds": timeout_seconds}
            })),
        )?;
        Ok(response.action)
    }

    pub fn confirm_action(
        &self,
        action_id: &str,
        phrase: &str,
        reason: &str,
    ) -> Result<ActionRecord, TrayError> {
        let path = format!("/control/v1/actions/{action_id}/confirm");
        let response: ActionEnvelope = self.control_request(
            "POST",
            &path,
            Some(&json!({"phrase": phrase, "reason": reason})),
        )?;
        Ok(response.action)
    }

    pub fn action(&self, action_id: &str) -> Result<ActionRecord, TrayError> {
        let path = format!("/control/v1/actions/{action_id}");
        let response: ActionEnvelope = self.control_request("GET", &path, None)?;
        Ok(response.action)
    }

    pub fn cancel_action(&self, action_id: &str, reason: &str) -> Result<ActionRecord, TrayError> {
        let path = format!("/control/v1/actions/{action_id}/cancel");
        let response: ActionEnvelope =
            self.control_request("POST", &path, Some(&json!({"reason": reason})))?;
        Ok(response.action)
    }

    pub fn record_recovery(&self, status: RecoveryStatus) -> Result<(), TrayError> {
        let _: Value = self.request(
            "POST",
            "/command",
            Some(&json!({
                "command": "supervision.recovery_record",
                "arguments": status,
            })),
            false,
        )?;
        Ok(())
    }

    pub fn acknowledge_force_stop(&self, action_id: &str) -> Result<(), TrayError> {
        let _: Value = self.request(
            "POST",
            "/command",
            Some(&json!({
                "command": "supervision.force_stop_ack",
                "arguments": {"actionId": action_id},
            })),
            false,
        )?;
        Ok(())
    }

    pub fn console_url(&self, ticket: &BootstrapTicket) -> String {
        format!(
            "http://127.0.0.1:{}{}",
            self.descriptor.port, ticket.bootstrap_path
        )
    }

    fn control_request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<T, TrayError> {
        let envelope: Envelope<T> = self.request(method, path, body, true)?;
        if envelope.ok {
            return envelope
                .data
                .ok_or_else(|| TrayError::Http("control response omitted data".into()));
        }
        let issue = envelope.error.unwrap_or(ControlError {
            code: "invalid_response".into(),
            message: "control response omitted error".into(),
        });
        Err(TrayError::Coordinator {
            code: issue.code,
            message: issue.message,
        })
    }

    fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        _control: bool,
    ) -> Result<T, TrayError> {
        let address = SocketAddr::from(([127, 0, 0, 1], self.descriptor.port));
        let mut stream = TcpStream::connect_timeout(&address, self.timeout)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        let payload = body.map(Value::to_string).unwrap_or_default();
        let request = format!(
            "{method} {path} HTTP/1.0\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.descriptor.port,
            self.descriptor.token.expose(),
            payload.len(),
            payload
        );
        stream.write_all(request.as_bytes())?;
        stream.flush()?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        let split = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or_else(|| TrayError::Http("HTTP response has no header boundary".into()))?;
        let headers = String::from_utf8_lossy(&response[..split]);
        let status = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<u16>().ok())
            .ok_or_else(|| TrayError::Http("HTTP response has no valid status".into()))?;
        let body = &response[split + 4..];
        if !(200..400).contains(&status) {
            if let Ok(envelope) = serde_json::from_slice::<Envelope<Value>>(body) {
                if let Some(issue) = envelope.error {
                    return Err(TrayError::Coordinator {
                        code: issue.code,
                        message: issue.message,
                    });
                }
            }
            return Err(TrayError::Http(format!(
                "coordinator returned HTTP {status}"
            )));
        }
        serde_json::from_slice(body).map_err(TrayError::Json)
    }
}
