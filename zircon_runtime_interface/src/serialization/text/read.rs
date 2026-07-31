use serde::Deserialize;
use serde_json::value::RawValue;

use crate::serialization::PayloadHeader;

/// A text envelope borrows its payload so current-version readers can decode T directly.
pub(in crate::serialization) struct BorrowedTextEnvelope<'a> {
    pub(in crate::serialization) header: PayloadHeader,
    pub(in crate::serialization) payload: &'a RawValue,
}

pub(in crate::serialization) enum TextInput<'a> {
    Envelope(BorrowedTextEnvelope<'a>),
    Legacy,
}

pub(in crate::serialization) enum TextReadError {
    Malformed(serde_json::Error),
    InvalidEnvelope(serde_json::Error),
}

#[derive(Deserialize)]
struct EnvelopeProbe<'a> {
    #[serde(rename = "$zircon", default, borrow)]
    envelope: Option<&'a RawValue>,
}

#[derive(Deserialize)]
struct EnvelopeHeaderProbe<'a> {
    #[serde(default, borrow)]
    header: Option<&'a RawValue>,
}

#[derive(Deserialize)]
struct HeaderSignatureProbe<'a> {
    #[serde(default, borrow)]
    schema_id: Option<&'a RawValue>,
    #[serde(default, borrow)]
    schema_version: Option<&'a RawValue>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTextDocument<'a> {
    #[serde(rename = "$zircon", borrow)]
    envelope: &'a RawValue,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTextEnvelope<'a> {
    header: PayloadHeader,
    #[serde(borrow)]
    payload: &'a RawValue,
}

/// Identifies the reserved envelope without materializing a legacy payload Value.
pub(in crate::serialization) fn inspect_text(bytes: &[u8]) -> Result<TextInput<'_>, TextReadError> {
    if first_non_whitespace(bytes) != Some(b'{') {
        return Ok(TextInput::Legacy);
    }

    let probe =
        serde_json::from_slice::<EnvelopeProbe<'_>>(bytes).map_err(TextReadError::Malformed)?;
    let Some(envelope) = probe.envelope else {
        return Ok(TextInput::Legacy);
    };
    if !claims_version_header(envelope).map_err(TextReadError::InvalidEnvelope)? {
        return Ok(TextInput::Legacy);
    }
    let document = serde_json::from_slice::<RawTextDocument<'_>>(bytes)
        .map_err(TextReadError::InvalidEnvelope)?;
    let envelope = serde_json::from_str::<RawTextEnvelope<'_>>(document.envelope.get())
        .map_err(TextReadError::InvalidEnvelope)?;
    Ok(TextInput::Envelope(BorrowedTextEnvelope {
        header: envelope.header,
        payload: envelope.payload,
    }))
}

fn claims_version_header(envelope: &RawValue) -> Result<bool, serde_json::Error> {
    if first_non_whitespace(envelope.get().as_bytes()) != Some(b'{') {
        return Ok(false);
    }
    let probe = serde_json::from_str::<EnvelopeHeaderProbe<'_>>(envelope.get())?;
    let Some(header) = probe.header else {
        return Ok(false);
    };
    if first_non_whitespace(header.get().as_bytes()) != Some(b'{') {
        return Ok(false);
    }
    let signature = serde_json::from_str::<HeaderSignatureProbe<'_>>(header.get())?;
    Ok(signature.schema_id.is_some() && signature.schema_version.is_some())
}

fn first_non_whitespace(bytes: &[u8]) -> Option<u8> {
    bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}
