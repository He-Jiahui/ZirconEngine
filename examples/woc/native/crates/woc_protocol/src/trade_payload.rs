use crate::{
    require_finite, validate_command_payload, ProtocolError, TRADE_OFFER_COMMAND_ID,
    TRADE_REQUEST_COMMAND_ID,
};

const NUMBER_BYTES: usize = 8;
const TRADE_OFFER_COUNT_BYTES: usize = 1;
const TRADE_OFFER_LENGTH_PREFIX_BYTES: usize = 4;
const TRADE_OFFER_MAX_ITEMS: usize = 6;
const TRADE_OFFER_MAX_ITEM_ID_UTF8_BYTES: usize = 256;
const TRADE_OFFER_ITEM_CONTEXT: &str = "TradeOfferCommandPayload.items";
const TRADE_OFFER_ITEM_ID_CONTEXT: &str = "TradeOfferItem.item_id";
const TRADE_OFFER_COUNT_CONTEXT: &str = "TradeOfferItem.count";
const TRADE_OFFER_COPPER_CONTEXT: &str = "TradeOfferCommandPayload.copper";

#[derive(Clone, Debug, PartialEq)]
pub struct TradeRequestCommandPayload {
    pub target_id: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TradeOfferItem {
    pub item_id: String,
    pub count: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TradeOfferCommandPayload {
    pub items: Vec<TradeOfferItem>,
    pub copper: f64,
}

impl TradeOfferCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        validate_trade_offer_item_count(self.items.len())?;
        let mut bytes = Vec::with_capacity(
            TRADE_OFFER_COUNT_BYTES
                + NUMBER_BYTES
                + self
                    .items
                    .iter()
                    .map(|item| TRADE_OFFER_LENGTH_PREFIX_BYTES + item.item_id.len() + NUMBER_BYTES)
                    .sum::<usize>(),
        );
        bytes.push(u8::try_from(self.items.len()).expect("bounded trade-offer item count fits u8"));
        for item in &self.items {
            write_trade_offer_item(item, &mut bytes)?;
        }
        bytes.extend_from_slice(
            &canonical_finite_f64(TRADE_OFFER_COPPER_CONTEXT, self.copper)?.to_le_bytes(),
        );
        validate_command_payload(TRADE_OFFER_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(TRADE_OFFER_COMMAND_ID, bytes)?;
        let count = bytes[0] as usize;
        let mut items = Vec::with_capacity(count);
        let mut consumed = TRADE_OFFER_COUNT_BYTES;
        for _ in 0..count {
            let (item, next) = read_trade_offer_item(bytes, consumed)?;
            items.push(item);
            consumed = next;
        }
        let (copper, consumed) =
            read_trade_offer_number(bytes, consumed, TRADE_OFFER_COPPER_CONTEXT)?;
        reject_trade_offer_trailing(bytes, consumed)?;
        Ok(Self { items, copper })
    }
}

impl TradeRequestCommandPayload {
    pub fn encode(self) -> Result<[u8; NUMBER_BYTES], ProtocolError> {
        let target_id =
            canonical_finite_f64("TradeRequestCommandPayload.target_id", self.target_id)?;
        let bytes = target_id.to_le_bytes();
        validate_command_payload(TRADE_REQUEST_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(TRADE_REQUEST_COMMAND_ID, bytes)?;
        Ok(Self {
            target_id: read_finite_f64(bytes, "TradeRequestCommandPayload.target_id")?,
        })
    }
}

pub(crate) fn validate_trade_request_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, "TradeRequestCommandPayload.target_id")?;
    Ok(())
}

pub(crate) fn validate_trade_offer_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let count = bytes[0] as usize;
    validate_trade_offer_item_count(count)?;
    let mut consumed = TRADE_OFFER_COUNT_BYTES;
    for _ in 0..count {
        consumed = read_trade_offer_item(bytes, consumed)?.1;
    }
    let (_, consumed) = read_trade_offer_number(bytes, consumed, TRADE_OFFER_COPPER_CONTEXT)?;
    reject_trade_offer_trailing(bytes, consumed)
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(bytes: &[u8], context: &'static str) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[..NUMBER_BYTES]
            .try_into()
            .expect("validated trade payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}

fn validate_trade_offer_item_count(count: usize) -> Result<(), ProtocolError> {
    if count > TRADE_OFFER_MAX_ITEMS {
        return Err(ProtocolError::CollectionTooLarge {
            context: TRADE_OFFER_ITEM_CONTEXT,
            actual: count,
            maximum: TRADE_OFFER_MAX_ITEMS,
        });
    }
    Ok(())
}

fn write_trade_offer_item(item: &TradeOfferItem, bytes: &mut Vec<u8>) -> Result<(), ProtocolError> {
    if item.item_id.len() > TRADE_OFFER_MAX_ITEM_ID_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context: TRADE_OFFER_ITEM_ID_CONTEXT,
            actual: item.item_id.len(),
            maximum: TRADE_OFFER_MAX_ITEM_ID_UTF8_BYTES,
        });
    }
    bytes.extend_from_slice(
        &u32::try_from(item.item_id.len())
            .expect("bounded trade-offer item id fits u32")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(item.item_id.as_bytes());
    bytes.extend_from_slice(
        &canonical_finite_f64(TRADE_OFFER_COUNT_CONTEXT, item.count)?.to_le_bytes(),
    );
    Ok(())
}

fn read_trade_offer_item(
    bytes: &[u8],
    offset: usize,
) -> Result<(TradeOfferItem, usize), ProtocolError> {
    let length = u32::from_le_bytes(
        take_trade_offer(
            bytes,
            offset,
            TRADE_OFFER_LENGTH_PREFIX_BYTES,
            TRADE_OFFER_ITEM_ID_CONTEXT,
        )?
        .try_into()
        .expect("trade-offer item id length prefix has four bytes"),
    ) as usize;
    if length > TRADE_OFFER_MAX_ITEM_ID_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context: TRADE_OFFER_ITEM_ID_CONTEXT,
            actual: length,
            maximum: TRADE_OFFER_MAX_ITEM_ID_UTF8_BYTES,
        });
    }
    let item_offset = offset.checked_add(TRADE_OFFER_LENGTH_PREFIX_BYTES).ok_or(
        ProtocolError::TruncatedPayload {
            context: TRADE_OFFER_ITEM_ID_CONTEXT,
            needed: TRADE_OFFER_LENGTH_PREFIX_BYTES,
            remaining: bytes.len().saturating_sub(offset),
        },
    )?;
    let item_id = take_trade_offer(bytes, item_offset, length, TRADE_OFFER_ITEM_ID_CONTEXT)?;
    let item_id = std::str::from_utf8(item_id).map_err(|_| ProtocolError::InvalidUtf8 {
        context: TRADE_OFFER_ITEM_ID_CONTEXT,
    })?;
    let (count, consumed) =
        read_trade_offer_number(bytes, item_offset + length, TRADE_OFFER_COUNT_CONTEXT)?;
    Ok((
        TradeOfferItem {
            item_id: item_id.to_owned(),
            count,
        },
        consumed,
    ))
}

fn read_trade_offer_number(
    bytes: &[u8],
    offset: usize,
    context: &'static str,
) -> Result<(f64, usize), ProtocolError> {
    let value = f64::from_le_bytes(
        take_trade_offer(bytes, offset, NUMBER_BYTES, context)?
            .try_into()
            .expect("trade-offer numeric field has eight bytes"),
    );
    Ok((canonical_finite_f64(context, value)?, offset + NUMBER_BYTES))
}

fn take_trade_offer<'a>(
    bytes: &'a [u8],
    offset: usize,
    length: usize,
    context: &'static str,
) -> Result<&'a [u8], ProtocolError> {
    let end = offset
        .checked_add(length)
        .ok_or(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining: bytes.len().saturating_sub(offset),
        })?;
    bytes
        .get(offset..end)
        .ok_or(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining: bytes.len().saturating_sub(offset),
        })
}

fn reject_trade_offer_trailing(bytes: &[u8], consumed: usize) -> Result<(), ProtocolError> {
    let remaining = bytes.len().saturating_sub(consumed);
    if remaining == 0 {
        Ok(())
    } else {
        Err(ProtocolError::TrailingPayload { remaining })
    }
}
