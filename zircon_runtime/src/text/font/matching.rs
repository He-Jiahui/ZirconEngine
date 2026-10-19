//! 字体查询中的家族身份与候选作用域；资产 owner 的私有家族不能被同名系统字体悄然替换。
//! 本模块只生成比较键、整理候选顺序与计算选面距离；字体登记、owner 隔离和字符覆盖由数据库消费端完成。

use std::collections::{HashMap, HashSet};

use crate::text::{FontFamilyName, FontStretch, FontStyle, FontWeight};

/// 区分家族比较键用途的哈希前缀，不承载 owner、数据库代际或字体源身份。
const FONT_FAMILY_IDENTITY_HASH_DOMAIN: &[u8] = b"zircon-font-family-identity-v1";

/// trim 与 ASCII 字节小写后的家族名哈希前 16 字节，供去重、索引和缓存键共用。
/// 这是有碰撞可能的比较键；不表示完整 Unicode 等价，也不持有家族文本或字体资源。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FontFamilyIdentity([u8; 16]);

/// OwnerLocalOnly 限定资产私有查询；OwnerThenGlobal 允许显式回退到全局家族。
/// 标签本身不携带 owner 或执行查询；调用者须同时传入当前请求的 owner。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FontFamilyCandidateScope {
    /// 只在当前请求的 owner 字体面中查找该家族。
    OwnerLocalOnly,
    /// owner 下没有该家族候选时允许补充全局候选；此判定先于字符覆盖过滤。
    OwnerThenGlobal,
}

/// 按首次出现顺序保存的家族值与查询作用域；只拥有值，不借用数据库或字体资源。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScopedFontFamilyCandidate {
    /// 首次保留的输入家族值；比较键规范化不会改写这里的拼写或空白。
    pub(super) family: FontFamilyName,
    /// 重复候选合并后的查询范围标签，由 owner 查询消费端解释。
    pub(super) scope: FontFamilyCandidateScope,
}

impl FontFamilyIdentity {
    /// 借入本键的全部 16 字节，供 `family_candidate_cache_key` 继续哈希。
    /// 返回引用与 `&self` 同期；调用方需保留本键，返回值不借用原始家族名。
    pub(super) const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// 为家族名生成去重与索引共用的固定长度比较键。
///
/// # 参数与返回
/// `family` 只在调用期间借用；先 `trim()`，再仅折叠 ASCII 字母，返回域内哈希前 16 字节。
/// 不执行 Unicode 规范化或完整大小写折叠，也不改写输入；结果不保留文本借用。
pub(super) fn font_family_identity(family: &str) -> FontFamilyIdentity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FONT_FAMILY_IDENTITY_HASH_DOMAIN);
    let family = family.trim();
    // 按 UTF-8 字节流折叠 ASCII；非 ASCII 字节保持原值。
    for byte in family.bytes() {
        hasher.update(&[byte.to_ascii_lowercase()]);
    }
    // 截取摘要作比较键，调用方仍负责 owner 范围和实际字体候选的有效性。
    let mut identity = [0_u8; 16];
    identity.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    FontFamilyIdentity(identity)
}

/// 消耗家族序列，跳过空名，并按比较键保留首次出现的家族值与顺序。
///
/// # 参数与返回
/// `families` 按值移入迭代器；结果拥有保留的原值，返回后不借用迭代器。
/// 容量只按 `size_hint()` 的下界预留，不保证结果长度、精确容量或零次扩容。
pub(super) fn dedupe_families(
    families: impl IntoIterator<Item = FontFamilyName>,
) -> Vec<FontFamilyName> {
    let families = families.into_iter();
    // 输入下界尚未扣除空名和重复项，不能当作最终家族数。
    let (capacity, _) = families.size_hint();
    let mut identities = HashSet::with_capacity(capacity);
    let mut result = Vec::with_capacity(capacity);
    for family in families {
        if family.is_empty() {
            continue;
        }
        // 只比较固定键；输出继续使用首次保留的家族原值。
        if !identities.insert(font_family_identity(family.as_str())) {
            continue;
        }
        result.push(family);
    }
    result
}

/// 保留首次出现的家族优先级；后续同名候选若允许全局查找，则放宽原候选的作用域。
///
/// `families` 移入 `(家族, 作用域)` 对；空名被跳过，结果拥有首个家族原值。
/// 同键后续项只可升级为 `OwnerThenGlobal`，不降级，也不重排首项。
/// 作用域由 `candidate_families` 与 `font_chain_metric_families` 构建，实际 owner 查询由数据库执行。
pub(super) fn dedupe_scoped_families(
    families: impl IntoIterator<Item = (FontFamilyName, FontFamilyCandidateScope)>,
) -> Vec<ScopedFontFamilyCandidate> {
    let families = families.into_iter();
    let (capacity, _) = families.size_hint();
    let mut index_by_identity = HashMap::<FontFamilyIdentity, usize>::with_capacity(capacity);
    let mut result = Vec::<ScopedFontFamilyCandidate>::with_capacity(capacity);
    for (family, scope) in families {
        if family.is_empty() {
            continue;
        }
        let identity = font_family_identity(family.as_str());
        // 映射保存首项位置；后续同键只合并作用域，不替换名称或移动位置。
        if let Some(index) = index_by_identity.get(&identity).copied() {
            if scope == FontFamilyCandidateScope::OwnerThenGlobal {
                result[index].scope = scope;
            }
            continue;
        }
        // 首次出现才登记位置并移入结果，保持查询家族优先级。
        index_by_identity.insert(identity, result.len());
        result.push(ScopedFontFamilyCandidate { family, scope });
    }
    result
}

/// 返回 `candidate` 与 `requested` 字重数值的绝对差；按值读取，不限幅或校验字体面。
pub(super) fn weight_distance(candidate: FontWeight, requested: FontWeight) -> u16 {
    candidate.0.abs_diff(requested.0)
}

/// 返回 `candidate` 与 `requested` 宽度偏好的绝对差；只比较 u16 数值，不执行布局缩放。
pub(super) fn stretch_distance(candidate: FontStretch, requested: FontStretch) -> u16 {
    candidate.0.abs_diff(requested.0)
}

/// 比较 `candidate` 与 `requested` 的样式类别，供选面分数的第三项使用。
/// 同类（包括任意 Oblique 角度）为 0，Italic/Oblique 互配为 1，其余为 2。
/// 不比较 Oblique 载荷，不进行角度校验，也不创建合成字体样式。
pub(super) fn style_distance(candidate: FontStyle, requested: FontStyle) -> u8 {
    match (candidate, requested) {
        (FontStyle::Normal, FontStyle::Normal) => 0,
        (FontStyle::Italic, FontStyle::Italic) => 0,
        (FontStyle::Oblique(_), FontStyle::Oblique(_)) => 0,
        (FontStyle::Italic, FontStyle::Oblique(_)) | (FontStyle::Oblique(_), FontStyle::Italic) => {
            1
        }
        _ => 2,
    }
}

#[cfg(test)]
#[path = "matching/tests/capacity_tests.rs"]
mod capacity_tests;
