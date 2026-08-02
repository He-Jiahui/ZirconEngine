use crate::{
    require_finite, validate_command_payload, ProtocolError, MAIL_DELETE_COMMAND_ID,
    MAIL_READ_COMMAND_ID, MAIL_SEND_COMMAND_ID, MAIL_TAKE_COMMAND_ID,
};

const MAIL_ID_BYTES: usize = 8;
pub const MAIL_SEND_MAX_PAYLOAD_BYTES: usize = 16 * 1024;
pub const MAIL_SEND_MAX_ATTACHMENTS: usize = 3;
const MAIL_SEND_BASE_BYTES: usize = 4 + 4 + 4 + 8 + 1;
const U32_BYTES: usize = 4;

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

#[derive(Clone, Debug, PartialEq)]
pub struct MailSendAttachment {
    pub item_id: String,
    pub count: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MailSendCommandPayload {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub copper: f64,
    pub items: Vec<MailSendAttachment>,
}

impl MailSendCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let copper = canonical_finite_f64("MailSendCommandPayload.copper", self.copper)?;
        if self.items.len() > MAIL_SEND_MAX_ATTACHMENTS {
            return Err(ProtocolError::CollectionTooLarge {
                context: "MailSendCommandPayload.items",
                actual: self.items.len(),
                maximum: MAIL_SEND_MAX_ATTACHMENTS,
            });
        }
        let encoded_length = encoded_mail_send_length(self)?;
        let mut bytes = Vec::with_capacity(encoded_length);
        push_utf8(&mut bytes, &self.to)?;
        push_utf8(&mut bytes, &self.subject)?;
        push_utf8(&mut bytes, &self.body)?;
        bytes.extend_from_slice(&copper.to_le_bytes());
        bytes.push(self.items.len() as u8);
        for item in &self.items {
            push_utf8(&mut bytes, &item.item_id)?;
            bytes.extend_from_slice(
                &canonical_finite_f64("MailSendAttachment.count", item.count)?.to_le_bytes(),
            );
        }
        debug_assert_eq!(bytes.len(), encoded_length);
        validate_command_payload(MAIL_SEND_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(MAIL_SEND_COMMAND_ID, bytes)?;
        decode_mail_send_payload(bytes)
    }
}

pub(crate) fn validate_mail_send_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    decode_mail_send_payload(bytes).map(|_| ())
}

fn encoded_mail_send_length(payload: &MailSendCommandPayload) -> Result<usize, ProtocolError> {
    let mut length = MAIL_SEND_BASE_BYTES;
    for value in [&payload.to, &payload.subject, &payload.body] {
        length = checked_mail_send_length(length, value.len().saturating_add(U32_BYTES))?;
    }
    for item in &payload.items {
        let _ = canonical_finite_f64("MailSendAttachment.count", item.count)?;
        length = checked_mail_send_length(
            length,
            item.item_id
                .len()
                .saturating_add(U32_BYTES)
                .saturating_add(MAIL_ID_BYTES),
        )?;
    }
    Ok(length)
}

fn checked_mail_send_length(current: usize, additional: usize) -> Result<usize, ProtocolError> {
    let actual = current.saturating_add(additional);
    if actual > MAIL_SEND_MAX_PAYLOAD_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context: "MailSendCommandPayload.payload",
            actual,
            maximum: MAIL_SEND_MAX_PAYLOAD_BYTES,
        });
    }
    Ok(actual)
}

fn push_utf8(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProtocolError> {
    let length = u32::try_from(value.len()).map_err(|_| ProtocolError::CollectionTooLarge {
        context: "MailSendCommandPayload.text",
        actual: value.len(),
        maximum: u32::MAX as usize,
    })?;
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn decode_mail_send_payload(bytes: &[u8]) -> Result<MailSendCommandPayload, ProtocolError> {
    let mut offset = 0;
    let to = read_utf8(bytes, &mut offset, "MailSendCommandPayload.to")?;
    let subject = read_utf8(bytes, &mut offset, "MailSendCommandPayload.subject")?;
    let body = read_utf8(bytes, &mut offset, "MailSendCommandPayload.body")?;
    let copper = read_finite(bytes, &mut offset, "MailSendCommandPayload.copper")?;
    let item_count = usize::from(read_byte(
        bytes,
        &mut offset,
        "MailSendCommandPayload.items",
    )?);
    if item_count > MAIL_SEND_MAX_ATTACHMENTS {
        return Err(ProtocolError::CollectionTooLarge {
            context: "MailSendCommandPayload.items",
            actual: item_count,
            maximum: MAIL_SEND_MAX_ATTACHMENTS,
        });
    }
    let mut items = Vec::with_capacity(item_count);
    for _ in 0..item_count {
        items.push(MailSendAttachment {
            item_id: read_utf8(bytes, &mut offset, "MailSendAttachment.item_id")?,
            count: read_finite(bytes, &mut offset, "MailSendAttachment.count")?,
        });
    }
    if offset != bytes.len() {
        return Err(ProtocolError::TrailingPayload {
            remaining: bytes.len() - offset,
        });
    }
    Ok(MailSendCommandPayload {
        to,
        subject,
        body,
        copper,
        items,
    })
}

fn read_utf8(
    bytes: &[u8],
    offset: &mut usize,
    context: &'static str,
) -> Result<String, ProtocolError> {
    let length = usize::try_from(u32::from_le_bytes(
        take(bytes, offset, U32_BYTES, context)?
            .try_into()
            .expect("mail-send text length is four bytes"),
    ))
    .expect("u32 fits usize on supported platforms");
    let value = take(bytes, offset, length, context)?;
    String::from_utf8(value.to_vec()).map_err(|_| ProtocolError::InvalidUtf8 { context })
}

fn read_finite(
    bytes: &[u8],
    offset: &mut usize,
    context: &'static str,
) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        take(bytes, offset, MAIL_ID_BYTES, context)?
            .try_into()
            .expect("mail-send number is eight bytes"),
    );
    canonical_finite_f64(context, value)
}

fn read_byte(bytes: &[u8], offset: &mut usize, context: &'static str) -> Result<u8, ProtocolError> {
    Ok(take(bytes, offset, 1, context)?[0])
}

fn take<'a>(
    bytes: &'a [u8],
    offset: &mut usize,
    length: usize,
    context: &'static str,
) -> Result<&'a [u8], ProtocolError> {
    let remaining = bytes.len().saturating_sub(*offset);
    if remaining < length {
        return Err(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining,
        });
    }
    let start = *offset;
    *offset += length;
    Ok(&bytes[start..*offset])
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
