#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// revision 标识源字节及可选解码格式；hash_words 再绑定 face/mip 布局，供 IBL artifact 请求共享。
pub(super) struct EnvironmentIblSourceIdentity {
    revision: u64,
    hash_words: [u32; 4],
}

impl EnvironmentIblSourceIdentity {
    pub(super) const fn revision(self) -> u64 {
        self.revision
    }

    pub(super) const fn hash_words(self) -> [u32; 4] {
        self.hash_words
    }
}

pub(super) fn derive_source_identity(
    bytes: &[u8],
    face_size: u32,
    mip_count: u32,
) -> EnvironmentIblSourceIdentity {
    derive_source_identity_with_optional_format(bytes, None, face_size, mip_count)
}

// 普通图像入口使用已解析的解码格式区分同一字节流的解释方式；
// DDS/KTX/.zcube 容器入口由自身格式字节提供区分，保留原有无格式键布局。
pub(super) fn derive_source_identity_with_format(
    bytes: &[u8],
    format_identity: u32,
    face_size: u32,
    mip_count: u32,
) -> EnvironmentIblSourceIdentity {
    derive_source_identity_with_optional_format(bytes, Some(format_identity), face_size, mip_count)
}

fn derive_source_identity_with_optional_format(
    bytes: &[u8],
    format_identity: Option<u32>,
    face_size: u32,
    mip_count: u32,
) -> EnvironmentIblSourceIdentity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    if let Some(format_identity) = format_identity {
        hasher.update(&format_identity.to_le_bytes());
    }
    let revision = digest_revision(hasher.finalize());
    hasher.update(&face_size.to_le_bytes());
    hasher.update(&mip_count.to_le_bytes());
    let hash_words = digest_hash_words(hasher.finalize());
    EnvironmentIblSourceIdentity {
        revision,
        hash_words,
    }
}

fn digest_revision(digest: blake3::Hash) -> u64 {
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
    .max(1)
}

fn digest_hash_words(digest: blake3::Hash) -> [u32; 4] {
    let bytes = digest.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}

#[cfg(test)]
#[path = "tests/source_identity.rs"]
mod tests;
