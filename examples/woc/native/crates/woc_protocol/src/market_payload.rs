use crate::{
    require_finite, validate_command_payload, ProtocolError, MARKET_BUY_COMMAND_ID,
    MARKET_CANCEL_COMMAND_ID,
};

const MARKET_LISTING_ID_BYTES: usize = 8;

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
