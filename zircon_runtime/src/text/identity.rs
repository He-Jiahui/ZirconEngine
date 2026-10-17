use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Process-local hash used only to select cache buckets.
///
/// Cache owners must still compare their full key and, for source-backed entries, the exact source
/// bytes. This type deliberately has no serialization or byte-export contract.
///
/// # 使用约束
/// 仅保存当前进程内的候选桶身份；持久化内容请使用 [`StableContentDigest`]。
/// 相同哈希不替代缓存所有者对完整键及原文字节的比较。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct EphemeralCacheHash(u64);

impl EphemeralCacheHash {
    /// 借用 `value` 的 [`Hash`] 实现计算一次候选哈希，并返回独立的值。
    /// 调用方须继续保留用于碰撞判定的完整键；参考 [`EphemeralCacheHasher`]。
    pub(crate) fn from_hashable<T>(value: &T) -> Self
    where
        T: Hash + ?Sized,
    {
        let mut hasher = EphemeralCacheHasher::new();
        hasher.write(value);
        hasher.finish()
    }

    /// 接收调用方已计算的进程内哈希值；只包装 `value`，不重新散列或校验。
    pub(crate) const fn from_process_hash(value: u64) -> Self {
        Self(value)
    }
}

/// 按调用顺序汇集 [`Hash`] 输入；[`Self::finish`] 消费状态并返回候选哈希。
/// 不提供持久化编码或跨版本哈希结果约定；参考 [`EphemeralCacheHash`]。
pub(crate) struct EphemeralCacheHasher(DefaultHasher);

impl EphemeralCacheHasher {
    /// 创建尚未写入输入的状态，供一次候选哈希计算使用。
    pub(crate) fn new() -> Self {
        Self(DefaultHasher::new())
    }

    /// 依 `value` 的 [`Hash`] 实现更新当前状态；借用仅在本次写入期间使用。
    pub(crate) fn write<T>(&mut self, value: &T)
    where
        T: Hash + ?Sized,
    {
        value.hash(&mut self.0);
    }

    /// 消费当前状态，返回已写输入的候选身份；不验证输入间是否发生碰撞。
    pub(crate) fn finish(self) -> EphemeralCacheHash {
        EphemeralCacheHash(self.0.finish())
    }
}

/// Deterministic 32-byte identity stored at an artifact boundary.
///
/// # 字节边界
/// 保存调用方约定的 32 字节身份；[`Self::from_bytes`] 不验证内容来源。
/// [`Self::blake3`] 才会散列所给原始字节；默认值只是全零字节。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct StableContentDigest([u8; 32]);

impl StableContentDigest {
    /// 散列借用的原始 `bytes` 并返回拥有的 32 字节摘要；输入不会被规范化。
    pub(crate) fn blake3(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// 按原样接收拥有的 `bytes`；不重算摘要，也不验证其算法或来源。
    pub(crate) const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// 借用内部字节供边界编码使用；返回引用的有效期受 `self` 借用约束。
    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 消费包装值并返回拥有的原始字节，供调用方按自己的协议编码。
    pub(crate) const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// 复用原样包装转换；约束同 [`StableContentDigest::from_bytes`]。
impl From<[u8; 32]> for StableContentDigest {
    fn from(value: [u8; 32]) -> Self {
        Self::from_bytes(value)
    }
}

/// 复用字节取出转换；不更改字节顺序，参考 [`StableContentDigest::into_bytes`]。
impl From<StableContentDigest> for [u8; 32] {
    fn from(value: StableContentDigest) -> Self {
        value.into_bytes()
    }
}

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
