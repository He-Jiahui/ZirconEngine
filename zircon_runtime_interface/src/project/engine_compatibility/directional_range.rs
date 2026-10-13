use std::cmp::Ordering;

use semver::{Comparator, Op, Version, VersionReq};

use super::ProjectEngineCompatibilityDisposition;

#[derive(Clone, Debug)]
/// 稳定版本区间的一端；`inclusive` 决定端点版本本身是否属于区间。
struct VersionBound {
    version: Version,
    inclusive: bool,
}

#[derive(Clone, Debug, Default)]
/// 由 comparator 交集得到的稳定版本范围；缺少端点表示该方向无界。
struct VersionRange {
    minimum: Option<VersionBound>,
    maximum: Option<VersionBound>,
}

/// 版本比较区间的交集与方向归类操作。
impl VersionRange {
    /// 同时满足两个比较器时取更严格的下界和上界。
    fn intersect(self, other: Self) -> Self {
        Self {
            minimum: stricter_minimum(self.minimum, other.minimum),
            maximum: stricter_maximum(self.maximum, other.maximum),
        }
    }

    /// 判断端点顺序是否矛盾，或相等端点是否因任一端排除而没有成员。
    fn is_empty(&self) -> bool {
        let (Some(minimum), Some(maximum)) = (&self.minimum, &self.maximum) else {
            return false;
        };
        match minimum.version.cmp(&maximum.version) {
            Ordering::Less => false,
            Ordering::Greater => true,
            Ordering::Equal => !(minimum.inclusive && maximum.inclusive),
        }
    }

    /// 只有运行版本整体落在下界外或上界外时才给出可证明的更新方向。
    fn directional_disposition(&self, running: &Version) -> ProjectEngineCompatibilityDisposition {
        // 空区间不可能由单向更新修复，保留为一般不兼容。
        if self.is_empty() {
            return ProjectEngineCompatibilityDisposition::Incompatible;
        }
        // 低于下界（或等于排除的下界）可证明项目要求更高版本。
        if self.minimum.as_ref().is_some_and(|minimum| {
            running < &minimum.version || (running == &minimum.version && !minimum.inclusive)
        }) {
            return ProjectEngineCompatibilityDisposition::ProjectRequiresNewerEngine;
        }
        // 高于上界（或等于排除的上界）可证明项目要求更旧版本。
        if self.maximum.as_ref().is_some_and(|maximum| {
            running > &maximum.version || (running == &maximum.version && !maximum.inclusive)
        }) {
            return ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine;
        }
        // semver 已拒绝当前版本，但区间边界不能证明升级或降级方向。
        ProjectEngineCompatibilityDisposition::Incompatible
    }
}

/// Returns a direction only when the entire requirement can be reduced to a stable-version
/// interval. Prerelease ranges intentionally stay indeterminate because Cargo's prerelease
/// admission rules cannot be represented by ordinary inclusive bounds.
/// 仅当整个要求可化为稳定版本区间时报告升级或降级方向；预发布规则无法用普通区间准确表示。
///
/// # Parameters
/// - `requirement`：已由 `semver::VersionReq` 解析的比较器；这些条件按逻辑与求交集。
/// - `running`：已被原始 semver 匹配检查判为不匹配的引擎版本；预发布版本会保留为不确定结果。
///
/// # Returns
/// 返回完整区间能证明的更新方向；预发布、无法归约或空区间均为一般不兼容。
///
/// # 约束
/// 此辅助函数只处理“已不匹配”的要求，不会重新判定兼容性；端点递增溢出会停止方向推导。
///
/// # 参考
/// 上游由 `assessment::assess_project_engine_compatibility` 在 `VersionReq::matches` 为假时调用；
/// `zircon_runtime_interface/src/project/tests/directional_range.rs` 覆盖比较器端点，公开行为由
/// `crate::project::tests::engine_compatibility` 覆盖。
pub(super) fn classify_incompatible_requirement(
    requirement: &VersionReq,
    running: &Version,
) -> ProjectEngineCompatibilityDisposition {
    // semver 对预发布版本有额外准入规则，简单闭区间不能完整表达，因此不猜测方向。
    if !running.pre.is_empty()
        || requirement
            .comparators
            .iter()
            .any(|comparator| !comparator.pre.is_empty())
    {
        return ProjectEngineCompatibilityDisposition::Incompatible;
    }

    // 比较器逐一收窄交集；任一比较器不能表示为稳定区间（含边界溢出）就放弃方向判定。
    let Some(range) = requirement
        .comparators
        .iter()
        .map(comparator_range)
        .try_fold(VersionRange::default(), |current, next| {
            next.map(|next| current.intersect(next))
        })
    else {
        return ProjectEngineCompatibilityDisposition::Incompatible;
    };
    range.directional_disposition(running)
}

/// 把 semver comparator 的操作符与部分版本字段换成等价稳定版本端点。
/// 无法精确表示时返回 `None`，让上层采用一般不兼容结果。
fn comparator_range(comparator: &Comparator) -> Option<VersionRange> {
    let base = comparator_version(comparator);
    match comparator.op {
        // 完整精确版本只包含单个版本；缺少 patch 或通配符则展开为前缀区间。
        Op::Exact if comparator.patch.is_some() => Some(exact_range(base)),
        Op::Exact | Op::Wildcard => prefix_range(&base, comparator.minor.is_some()),
        // 部分版本的严格大于移到下一个前缀边界；完整版本保留排他的原端点。
        Op::Greater => Some(VersionRange {
            minimum: Some(VersionBound {
                version: greater_lower_bound(&base, comparator.minor, comparator.patch)?,
                inclusive: comparator.patch.is_none(),
            }),
            maximum: None,
        }),
        // 大于等于直接采用 comparator 表示的最小版本作为闭下界。
        Op::GreaterEq => Some(VersionRange {
            minimum: Some(VersionBound {
                version: base,
                inclusive: true,
            }),
            maximum: None,
        }),
        // 小于比较器给出排他的上界，不需要计算相邻版本。
        Op::Less => Some(VersionRange {
            minimum: None,
            maximum: Some(VersionBound {
                version: base,
                inclusive: false,
            }),
        }),
        // 部分版本的小于等于扩展到下一前缀的排他上界；完整版本自身闭合。
        Op::LessEq => Some(VersionRange {
            minimum: None,
            maximum: Some(VersionBound {
                version: less_equal_upper_bound(&base, comparator.minor, comparator.patch)?,
                inclusive: comparator.minor.is_some() && comparator.patch.is_some(),
            }),
        }),
        // 波浪范围保留下界，并按已声明精度把上界限制到下一 minor 或 major。
        Op::Tilde => Some(VersionRange {
            minimum: Some(VersionBound {
                version: base.clone(),
                inclusive: true,
            }),
            maximum: Some(VersionBound {
                version: if comparator.minor.is_some() {
                    next_minor(&base)?
                } else {
                    next_major(&base)?
                },
                inclusive: false,
            }),
        }),
        // 脱字符范围的上界还取决于零主版本下的已声明精度。
        Op::Caret => Some(caret_range(&base, comparator.minor, comparator.patch)?),
        _ => None,
    }
}

/// 从 comparator 的可选版本部件构造比较基值；缺少的 minor 或 patch 按 semver 表示补零。
fn comparator_version(comparator: &Comparator) -> Version {
    let mut version = Version::new(
        comparator.major,
        comparator.minor.unwrap_or_default(),
        comparator.patch.unwrap_or_default(),
    );
    version.pre = comparator.pre.clone();
    version
}

/// 把完整精确版本表示为同值闭合的上下界。
fn exact_range(version: Version) -> VersionRange {
    VersionRange {
        minimum: Some(VersionBound {
            version: version.clone(),
            inclusive: true,
        }),
        maximum: Some(VersionBound {
            version,
            inclusive: true,
        }),
    }
}

/// 把部分精确/通配比较转换为含前缀起点、排除下一同级前缀的范围。
fn prefix_range(base: &Version, has_minor: bool) -> Option<VersionRange> {
    Some(VersionRange {
        minimum: Some(VersionBound {
            version: base.clone(),
            inclusive: true,
        }),
        maximum: Some(VersionBound {
            version: if has_minor {
                next_minor(base)?
            } else {
                next_major(base)?
            },
            inclusive: false,
        }),
    })
}

/// 按 semver caret 规则选择上界；零主版本时边界取决于声明的 minor 与 patch 精度。
fn caret_range(base: &Version, minor: Option<u64>, patch: Option<u64>) -> Option<VersionRange> {
    let upper = match (base.major, minor, patch) {
        (_, None, _) => next_major(base)?,
        (1.., Some(_), _) => next_major(base)?,
        (0, Some(minor), _) if minor > 0 => next_minor(base)?,
        (0, Some(_), Some(_)) => next_patch(base)?,
        (0, Some(_), None) => next_minor(base)?,
    };
    Some(VersionRange {
        minimum: Some(VersionBound {
            version: base.clone(),
            inclusive: true,
        }),
        maximum: Some(VersionBound {
            version: upper,
            inclusive: false,
        }),
    })
}

/// 计算严格大于部分版本时的下界；完整版本沿用原值并由调用端设为排他。
fn greater_lower_bound(base: &Version, minor: Option<u64>, patch: Option<u64>) -> Option<Version> {
    match (minor, patch) {
        (None, _) => next_major(base),
        (Some(_), None) => next_minor(base),
        (Some(_), Some(_)) => Some(base.clone()),
    }
}

/// 计算小于等于部分版本的边界；完整版本沿用原版本，部分版本推进到下一前缀。
fn less_equal_upper_bound(
    base: &Version,
    minor: Option<u64>,
    patch: Option<u64>,
) -> Option<Version> {
    match (minor, patch) {
        (None, _) => next_major(base),
        (Some(_), None) => next_minor(base),
        (Some(_), Some(_)) => Some(base.clone()),
    }
}

/// 返回下一个 major 的零 minor/patch 版本；整数溢出表示无法建立方向边界。
fn next_major(version: &Version) -> Option<Version> {
    version
        .major
        .checked_add(1)
        .map(|major| Version::new(major, 0, 0))
}

/// 返回同一 major 的下一个 minor 零 patch 版本；溢出由上层作为未知方向处理。
fn next_minor(version: &Version) -> Option<Version> {
    version
        .minor
        .checked_add(1)
        .map(|minor| Version::new(version.major, minor, 0))
}

/// 返回同一 major/minor 的下一个 patch；溢出使依赖该边界的方向无法推导。
fn next_patch(version: &Version) -> Option<Version> {
    version
        .patch
        .checked_add(1)
        .map(|patch| Version::new(version.major, version.minor, patch))
}

/// 选择较大的下界；数值相等时只有双方都闭合，交集端点才闭合。
fn stricter_minimum(
    current: Option<VersionBound>,
    candidate: Option<VersionBound>,
) -> Option<VersionBound> {
    match (current, candidate) {
        (None, other) | (other, None) => other,
        (Some(current), Some(candidate)) => match current.version.cmp(&candidate.version) {
            Ordering::Less => Some(candidate),
            Ordering::Greater => Some(current),
            Ordering::Equal => Some(VersionBound {
                version: current.version,
                inclusive: current.inclusive && candidate.inclusive,
            }),
        },
    }
}

/// 选择较小的上界；数值相等时只有双方都闭合，交集端点才闭合。
fn stricter_maximum(
    current: Option<VersionBound>,
    candidate: Option<VersionBound>,
) -> Option<VersionBound> {
    match (current, candidate) {
        (None, other) | (other, None) => other,
        (Some(current), Some(candidate)) => match current.version.cmp(&candidate.version) {
            Ordering::Less => Some(current),
            Ordering::Greater => Some(candidate),
            Ordering::Equal => Some(VersionBound {
                version: current.version,
                inclusive: current.inclusive && candidate.inclusive,
            }),
        },
    }
}

// 方向边界用独立测试文件覆盖；此模块仅在单元测试构建中编译。
#[cfg(test)]
#[path = "../tests/directional_range.rs"]
mod tests;
