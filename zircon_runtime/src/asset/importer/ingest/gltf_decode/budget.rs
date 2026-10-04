use crate::asset::AssetImportError;

use super::gltf_parse_error;

pub(super) struct DecodedBudget {
    remaining: u64,
    limit: u64,
}

impl DecodedBudget {
    pub(super) fn new(limit: u64) -> Self {
        Self {
            remaining: limit,
            limit,
        }
    }

    pub(super) fn remaining(&self) -> u64 {
        self.remaining
    }

    pub(super) fn check(&self, bytes: u64, kind: &str) -> Result<(), AssetImportError> {
        if bytes > self.remaining {
            return Err(gltf_parse_error(format!(
                "gltf decoded {kind} budget exceeds the {}-byte cumulative limit",
                self.limit
            )));
        }
        Ok(())
    }

    pub(super) fn charge(&mut self, bytes: u64, kind: &str) -> Result<(), AssetImportError> {
        self.check(bytes, kind)?;
        self.remaining -= bytes;
        Ok(())
    }
}
