use std::collections::BTreeSet;

use zircon_runtime::asset::DataAsset;

use super::super::{
    StandaloneCurveChannelId, StandaloneCurveCodec, StandaloneCurveError, StandaloneCurveKey,
    StandaloneCurveKind,
};

impl StandaloneCurveCodec {
    /// Prepares one deletion batch using channel identity and each full expected authored key.
    ///
    /// Missing, duplicate or stale targets reject the entire batch. Expected-key equality
    /// retains f32 bits but cannot establish the document revision or reject document ABA.
    /// Empty channels remain empty, without synthetic keys/defaults. The original host owns
    /// selection, focus, exact source inverse, one history/CAS commit, SaveCtx and cancellation.
    pub fn prepare_keys_remove(
        asset: &DataAsset,
        expected: StandaloneCurveKind,
        keys: &[(StandaloneCurveChannelId, StandaloneCurveKey)],
    ) -> Result<Option<DataAsset>, StandaloneCurveError> {
        let mut source = Self::decode(asset, expected)?;
        source.validate_surface()?;
        let mut seen = BTreeSet::new();
        for (channel_id, expected_key) in keys {
            if !seen.insert((*channel_id, expected_key.id.as_str())) {
                return Err(StandaloneCurveError::invalid(
                    "remove",
                    "duplicate key identity in the removal batch",
                ));
            }
            let stale = || StandaloneCurveError::StaleKey {
                channel: *channel_id,
                key_id: expected_key.id.clone(),
            };
            let channel = source
                .channels
                .iter_mut()
                .find(|channel| channel.id == *channel_id)
                .ok_or_else(stale)?;
            let index = channel
                .keys
                .iter()
                .position(|key| key.id == expected_key.id)
                .ok_or_else(stale)?;
            if channel.keys[index] != *expected_key {
                return Err(stale());
            }
            channel.keys.remove(index);
        }
        source.validate_surface()?;
        if keys.is_empty() {
            return Ok(None);
        }
        let mut candidate = asset.clone();
        let changed = Self::encode_into(&source, &mut candidate, expected)?;
        Ok(changed.then_some(candidate))
    }
}
