//! Runtime 世界变化和订阅失效的传输 DTO。
//!
//! WorldFact 描述可观察的世界变化，不包含 Editor 视图状态。Runtime 订阅表把事实映射到
//! WatchToken；Runtime 会话把待发事实与失效 token 合并为 InvalidationBatch，Editor 同步
//! pump 再将 token 映射回本地视图。
//!
//! # 使用时机与生命周期
//!
//! 订阅表在 flush 时给批次赋予世界代数；Runtime 会话暂存这些批次并按输出预算分页返回。
//! 批次中的 token 应在签发它们的会话上下文中解释，facts 可供 Editor 的事实消费者更新
//! 运行时投影。
//!
//! # 参考
//!
//! Runtime 事实入队见 zircon_runtime/src/scene/inspection/subscription.rs，会话批次封装见
//! zircon_runtime/src/dynamic_api/session/world_sync.rs；Editor 投影见
//! zircon_editor/src/core/sync/pump.rs。

use serde::{Deserialize, Serialize};

use crate::resource::ResourceId;

use super::{EntityId, WatchToken};

/// Stable summary of the runtime-owned dynamic-scene reload apply report.
///
/// 该摘要随 AssetReloadApplied 事实传给 Editor，只保留应用、失败、过期和仍待处理项数，
/// 不携带每个资产的诊断详情。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetReloadFrameApplyReportDto {
    /// 本帧成功应用的动态场景资源数。
    pub applied: u64,
    /// 本帧应用失败的资源数。
    pub failed: u64,
    /// 本帧判定为陈旧而未应用的资源数。
    pub stale: u64,
    /// 仍需后续帧处理的资源数。
    pub pending_count: u64,
}

/// 事实只描述世界变化，订阅表负责映射到 dirty token，Editor 再映射到视图。
/// Runtime facts that can invalidate editor projections without carrying editor state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorldFact {
    /// 一个实体已进入 Runtime 世界。
    Spawned(EntityId),
    /// 一个实体已从 Runtime 世界移除。
    Despawned(EntityId),
    /// 一个实体的父实体发生变化；None 表示新层级下无父实体。
    Reparented {
        /// 发生重挂接的实体。
        entity: EntityId,
        /// 记录事实时该实体的新父实体。
        new_parent: Option<EntityId>,
    },
    /// 指定场景资源已载入 Runtime 世界。
    SceneLoaded {
        /// 已载入场景的资源身份。
        scene: ResourceId,
    },
    /// 指定场景资源已从 Runtime 世界卸载。
    SceneUnloaded {
        /// 已卸载场景的资源身份。
        scene: ResourceId,
    },
    /// Runtime 世界已被替换，并进入给定的替换代数。
    WorldReplaced {
        /// 替换后的世界代数。
        replacement_epoch: u64,
    },
    /// 一帧动态场景资产重载应用活动的汇总。
    AssetReloadApplied(AssetReloadFrameApplyReportDto),
}

/// 一帧内将同代事实和失效 token 批量传输；消费者以代数和 token 维护投影。
/// One frame's monotonic generation plus the subscriptions and facts it invalidated.
///
/// Runtime 订阅表按世界代数产生批次；会话保留这些批次并按输出预算分页。分页保留每个
/// 批次的代数，单个批次的 token 或事实也可能分布在相邻页中；成功提交输出后才移除已发送项。
/// dirty 是失效订阅身份，facts 是 Runtime 记录的变化事实。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvalidationBatch {
    /// 生成或封装此批次时对应的 Runtime 世界代数。
    pub generation: u64,
    /// 此批次使其投影失效的订阅 token；Runtime 订阅表输出时严格递增且无重复。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dirty: Vec<WatchToken>,
    /// 批次所报告的世界变化事实。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub facts: Vec<WorldFact>,
}

impl InvalidationBatch {
    /// 检查 dirty 中的 token 是否严格递增，因此没有重复项。
    ///
    /// # 返回值
    ///
    /// 空列表、单项列表或严格递增列表返回 true；包含重复项或逆序项返回 false。
    ///
    /// # 使用时机与生命周期
    ///
    /// Runtime 订阅表使用有序索引直接生成规范列表。Editor pump 可据此选择免分配投影快路；
    /// 来自传输层的非规范列表仍需交由诊断慢路处理，不能因本检查失败而丢弃。
    /// Returns true when runtime dirty tokens are strictly increasing and therefore unique.
    ///
    /// Runtime subscription tables emit this canonical form directly from their ordered dirty
    /// index. Consumers may use it as a no-allocation projection fast path; malformed transport
    /// input remains valid wire data and must stay observable through the diagnostic slow path.
    pub fn has_canonical_dirty_tokens(&self) -> bool {
        self.dirty.windows(2).all(|tokens| tokens[0] < tokens[1])
    }
}
