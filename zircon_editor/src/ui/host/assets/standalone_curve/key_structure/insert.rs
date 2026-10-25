use std::collections::BTreeSet;

use zircon_runtime::asset::DataAsset;

use super::super::{
    StandaloneCurveChannelId, StandaloneCurveCodec, StandaloneCurveError, StandaloneCurveKey,
    StandaloneCurveKind,
};

impl StandaloneCurveCodec {
    /// Prepares one whole-source candidate for a batch of fully authored new keys.
    ///
    /// The caller supplies actual stable identities, modes and slopes; no handle, default,
    /// tangent or sample is invented. Original and final sources must satisfy the existing
    /// surface contract. Semantic admission and one original Document/history/CAS transaction
    /// remain with the host, which must retain exact before/after DataAssets as its inverse.
    pub fn prepare_keys_insert(
        asset: &DataAsset,
        expected: StandaloneCurveKind,
        keys: &[(StandaloneCurveChannelId, StandaloneCurveKey)],
    ) -> Result<Option<DataAsset>, StandaloneCurveError> {
        let mut source = Self::decode(asset, expected)?;
        source.validate_surface()?;
        let mut seen = BTreeSet::new();
        for (channel_id, key) in keys {
            if !seen.insert((*channel_id, key.id.as_str())) {
                return Err(StandaloneCurveError::invalid(
                    "insert",
                    "duplicate key identity in the insertion batch",
                ));
            }
            let channel = source
                .channels
                .iter_mut()
                .find(|channel| channel.id == *channel_id)
                .ok_or_else(|| {
                    StandaloneCurveError::invalid("insert", "channel is absent from the source")
                })?;
            if channel.keys.iter().any(|current| current.id == key.id) {
                return Err(StandaloneCurveError::invalid(
                    "insert",
                    "new key identity already exists in the source",
                ));
            }
            channel.keys.push(key.clone());
        }
        // UI/filter order is unrelated to authored input order. total_cmp is safe even for
        // invalid incoming numbers; the complete finite/strict-order gate below rejects them.
        for channel in &mut source.channels {
            channel
                .keys
                .sort_by(|left, right| left.input.total_cmp(&right.input));
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
