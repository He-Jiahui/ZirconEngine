mod decode;
mod encode;
mod envelope;
mod value;
mod wire;

pub(super) use decode::{decode_binary_current, decode_binary_header, decode_binary_payload};
pub(super) use encode::encode_binary_payload;
#[cfg(test)]
pub(super) use encode::encode_binary_value;
pub(super) use value::DirectBinaryDecodeError;
#[cfg(test)]
pub(super) use value::{BinaryNode, BinaryValue};
#[cfg(test)]
pub(super) use wire::{MAX_BINARY_BODY_BYTES, MAX_BINARY_DEPTH, MAX_BINARY_STRING_BYTES};
