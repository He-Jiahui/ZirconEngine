use crate::command_payload::{decode_utf8_id_f64_pair, encode_utf8_id_f64_pair};
use crate::{
    require_finite, validate_command_payload, ProtocolError, MARKET_BUY_COMMAND_ID,
    MARKET_CANCEL_COMMAND_ID, MARKET_LIST_COMMAND_ID, MARKET_SEARCH_COMMAND_ID,
};

const MARKET_LISTING_ID_BYTES: usize = 8;
const MARKET_SEARCH_LENGTH_PREFIX_BYTES: usize = 4;
const MARKET_SEARCH_STRING_COUNT: usize = 4;
const MARKET_SEARCH_MAX_UTF8_BYTES: usize = 256;
const MARKET_SEARCH_PAGE_BYTES: usize = 8;
const MARKET_SEARCH_STRING_CONTEXTS: [&str; MARKET_SEARCH_STRING_COUNT] = [
    "MarketSearchCommandPayload.search",
    "MarketSearchCommandPayload.item_type",
    "MarketSearchCommandPayload.subtype",
    "MarketSearchCommandPayload.rarity",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketAction {
    Buy,
    Cancel,
}

impl MarketAction {
    const fn command_id(self) -> u16 {
        match self {
            Self::Buy => MARKET_BUY_COMMAND_ID,
            Self::Cancel => MARKET_CANCEL_COMMAND_ID,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MarketListingIdPayload {
    pub listing_id: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MarketListCommandPayload {
    pub item_id: String,
    pub count: f64,
    pub price: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MarketSearchCommandPayload {
    pub search: String,
    pub item_type: String,
    pub subtype: String,
    pub rarity: String,
    pub page: f64,
}

impl MarketSearchCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let values = [
            (self.search.as_str(), MARKET_SEARCH_STRING_CONTEXTS[0]),
            (self.item_type.as_str(), MARKET_SEARCH_STRING_CONTEXTS[1]),
            (self.subtype.as_str(), MARKET_SEARCH_STRING_CONTEXTS[2]),
            (self.rarity.as_str(), MARKET_SEARCH_STRING_CONTEXTS[3]),
        ];
        let mut bytes = Vec::with_capacity(
            MARKET_SEARCH_PAGE_BYTES
                + values
                    .iter()
                    .map(|(value, _)| MARKET_SEARCH_LENGTH_PREFIX_BYTES + value.len())
                    .sum::<usize>(),
        );
        for (value, context) in values {
            write_market_search_string(value, context, &mut bytes)?;
        }
        bytes.extend_from_slice(
            &canonical_finite_f64("MarketSearchCommandPayload.page", self.page)?.to_le_bytes(),
        );
        validate_command_payload(MARKET_SEARCH_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(MARKET_SEARCH_COMMAND_ID, bytes)?;
        let (search, consumed) =
            read_market_search_string(bytes, 0, MARKET_SEARCH_STRING_CONTEXTS[0])?;
        let (item_type, consumed) =
            read_market_search_string(bytes, consumed, MARKET_SEARCH_STRING_CONTEXTS[1])?;
        let (subtype, consumed) =
            read_market_search_string(bytes, consumed, MARKET_SEARCH_STRING_CONTEXTS[2])?;
        let (rarity, consumed) =
            read_market_search_string(bytes, consumed, MARKET_SEARCH_STRING_CONTEXTS[3])?;
        let (page, consumed) = read_market_search_page(bytes, consumed)?;
        reject_market_search_trailing(bytes, consumed)?;
        Ok(Self {
            search: search.to_owned(),
            item_type: item_type.to_owned(),
            subtype: subtype.to_owned(),
            rarity: rarity.to_owned(),
            page,
        })
    }
}

impl MarketListCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        encode_utf8_id_f64_pair(
            MARKET_LIST_COMMAND_ID,
            &self.item_id,
            self.count,
            self.price,
            "MarketListCommandPayload.count",
            "MarketListCommandPayload.price",
        )
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let (item_id, count, price) = decode_utf8_id_f64_pair(
            MARKET_LIST_COMMAND_ID,
            bytes,
            "MarketListCommandPayload.count",
            "MarketListCommandPayload.price",
        )?;
        Ok(Self {
            item_id,
            count,
            price,
        })
    }
}

impl MarketListingIdPayload {
    pub fn encode(
        self,
        action: MarketAction,
    ) -> Result<[u8; MARKET_LISTING_ID_BYTES], ProtocolError> {
        let listing_id =
            canonical_finite_f64("MarketListingIdPayload.listing_id", self.listing_id)?;
        let bytes = listing_id.to_le_bytes();
        validate_command_payload(action.command_id(), &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8], action: MarketAction) -> Result<Self, ProtocolError> {
        validate_command_payload(action.command_id(), bytes)?;
        Ok(Self {
            listing_id: read_finite_f64(bytes, "MarketListingIdPayload.listing_id")?,
        })
    }
}

pub(crate) fn validate_market_listing_id_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, "MarketListingIdPayload.listing_id")?;
    Ok(())
}

pub(crate) fn validate_market_search_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let mut consumed = 0;
    for context in MARKET_SEARCH_STRING_CONTEXTS {
        consumed = read_market_search_string(bytes, consumed, context)?.1;
    }
    let (_, consumed) = read_market_search_page(bytes, consumed)?;
    reject_market_search_trailing(bytes, consumed)
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(bytes: &[u8], context: &'static str) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[..MARKET_LISTING_ID_BYTES]
            .try_into()
            .expect("validated market-listing payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}

fn write_market_search_string(
    value: &str,
    context: &'static str,
    bytes: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    if value.len() > MARKET_SEARCH_MAX_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: value.len(),
            maximum: MARKET_SEARCH_MAX_UTF8_BYTES,
        });
    }
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .expect("bounded market-search string length fits u32")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn read_market_search_string<'a>(
    bytes: &'a [u8],
    offset: usize,
    context: &'static str,
) -> Result<(&'a str, usize), ProtocolError> {
    let length = u32::from_le_bytes(
        take_market_search(bytes, offset, MARKET_SEARCH_LENGTH_PREFIX_BYTES, context)?
            .try_into()
            .expect("market-search string length prefix has four bytes"),
    ) as usize;
    if length > MARKET_SEARCH_MAX_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: length,
            maximum: MARKET_SEARCH_MAX_UTF8_BYTES,
        });
    }
    let value_offset = offset
        .checked_add(MARKET_SEARCH_LENGTH_PREFIX_BYTES)
        .ok_or(ProtocolError::TruncatedPayload {
            context,
            needed: MARKET_SEARCH_LENGTH_PREFIX_BYTES,
            remaining: bytes.len().saturating_sub(offset),
        })?;
    let value = take_market_search(bytes, value_offset, length, context)?;
    let value = std::str::from_utf8(value).map_err(|_| ProtocolError::InvalidUtf8 { context })?;
    Ok((value, value_offset + length))
}

fn read_market_search_page(bytes: &[u8], offset: usize) -> Result<(f64, usize), ProtocolError> {
    let context = "MarketSearchCommandPayload.page";
    let value = f64::from_le_bytes(
        take_market_search(bytes, offset, MARKET_SEARCH_PAGE_BYTES, context)?
            .try_into()
            .expect("market-search page has eight bytes"),
    );
    Ok((
        canonical_finite_f64(context, value)?,
        offset + MARKET_SEARCH_PAGE_BYTES,
    ))
}

fn take_market_search<'a>(
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

fn reject_market_search_trailing(bytes: &[u8], consumed: usize) -> Result<(), ProtocolError> {
    let remaining = bytes.len().saturating_sub(consumed);
    if remaining == 0 {
        Ok(())
    } else {
        Err(ProtocolError::TrailingPayload { remaining })
    }
}
