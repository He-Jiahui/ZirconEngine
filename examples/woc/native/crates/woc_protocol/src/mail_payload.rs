use crate::{
    require_finite, validate_command_payload, ProtocolError, MAIL_DELETE_COMMAND_ID,
    MAIL_READ_COMMAND_ID, MAIL_TAKE_COMMAND_ID,
};

const MAIL_ID_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailAction {
    Take,
    Delete,
    MarkRead,
}

impl MailAction {
    const fn command_id(self) -> u16 {
        match self {
            Self::Take => MAIL_TAKE_COMMAND_ID,
            Self::Delete => MAIL_DELETE_COMMAND_ID,
            Self::MarkRead => MAIL_READ_COMMAND_ID,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MailIdCommandPayload {
    pub mail_id: f64,
}

impl MailIdCommandPayload {
    pub fn encode(self, action: MailAction) -> Result<[u8; MAIL_ID_BYTES], ProtocolError> {
        let mail_id = canonical_finite_f64("MailIdCommandPayload.mail_id", self.mail_id)?;
        let bytes = mail_id.to_le_bytes();
        validate_command_payload(action.command_id(), &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8], action: MailAction) -> Result<Self, ProtocolError> {
        validate_command_payload(action.command_id(), bytes)?;
        Ok(Self {
            mail_id: read_finite_f64(bytes, "MailIdCommandPayload.mail_id")?,
        })
    }
}

pub(crate) fn validate_mail_id_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, "MailIdCommandPayload.mail_id")?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(bytes: &[u8], context: &'static str) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[..MAIL_ID_BYTES]
            .try_into()
            .expect("validated mail-id payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}
