#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IblSourceCubemapBundlePayloadStamp {
    encoded_len: u64,
    digest: [u8; blake3::OUT_LEN],
}

impl IblSourceCubemapBundlePayloadStamp {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            encoded_len: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            digest: *blake3::hash(bytes).as_bytes(),
        }
    }

    pub(crate) const fn from_parts(encoded_len: u64, digest: [u8; blake3::OUT_LEN]) -> Self {
        Self {
            encoded_len,
            digest,
        }
    }

    pub(crate) const fn encoded_len(self) -> u64 {
        self.encoded_len
    }

    pub(crate) const fn digest(self) -> [u8; blake3::OUT_LEN] {
        self.digest
    }

    pub(crate) fn matches_bytes(self, bytes: &[u8]) -> bool {
        u64::try_from(bytes.len()).ok() == Some(self.encoded_len)
            && blake3::hash(bytes).as_bytes() == &self.digest
    }
}
