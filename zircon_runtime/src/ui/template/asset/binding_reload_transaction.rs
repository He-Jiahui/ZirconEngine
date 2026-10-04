use thiserror::Error;
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::template::{
    UiCompiledBindingGeneration, UiCompiledBindingProgram,
};

/// 热重载发布回执，记录表面切换后的代际拒绝与组件状态迁移结果。
/// 静止状态针对已替换表面的旧绑定句柄；这份记录不承担独立线程或异步任务的等待屏障。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiBindingQuiescenceReceipt {
    pub tree_id: UiTreeId,
    pub old_generation: UiCompiledBindingGeneration,
    pub published_generation: UiCompiledBindingGeneration,
    pub retired_binding_count: usize,
    pub published_binding_count: usize,
    pub state_entries_migrated: usize,
    pub state_entries_reset: usize,
    pub old_generation_retired: bool,
    pub old_generation_quiescent: bool,
    pub stale_handles_rejected: bool,
}

// 将绑定合法性检查放在预备阶段，所有目标表面准备成功后才允许统一发布；本对象不持有或修改旧程序。
#[derive(Clone, Debug)]
pub(crate) struct UiBindingReloadTransaction {
    tree_id: UiTreeId,
    old_generation: UiCompiledBindingGeneration,
    published_generation: UiCompiledBindingGeneration,
    old_binding_count: usize,
    published_binding_count: usize,
    retires_old_generation: bool,
}

/// 在任何表面被替换前终止预备的契约错误，使宿主继续保留上一份可用状态。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingReloadPrepareError {
    #[error("existing binding program for {tree_id:?} is malformed")]
    MalformedPrevious { tree_id: UiTreeId },
    #[error("replacement binding program for {tree_id:?} is malformed")]
    MalformedReplacement { tree_id: UiTreeId },
    #[error(
        "binding reload for {tree_id:?} cannot replace compiled generation {old_generation:?} with an invalid generation"
    )]
    InvalidReplacementGeneration {
        tree_id: UiTreeId,
        old_generation: UiCompiledBindingGeneration,
    },
    #[error("binding reload for {tree_id:?} removed root asset identity {old_asset}")]
    MissingReplacementRootAsset {
        tree_id: UiTreeId,
        old_asset: String,
    },
    #[error(
        "binding reload for {tree_id:?} changed root asset identity from {old_asset} to {new_asset}"
    )]
    RootAssetMismatch {
        tree_id: UiTreeId,
        old_asset: String,
        new_asset: String,
    },
    #[error("binding reload for {tree_id:?} reused generation {generation:?} for different IR")]
    GenerationCollision {
        tree_id: UiTreeId,
        generation: UiCompiledBindingGeneration,
    },
}

impl UiBindingReloadTransaction {
    // 代际改变不允许顺便改变根资产身份；同一代际也不能代表不同内容，否则已发出的句柄会误指向新绑定。
    pub(crate) fn prepare(
        tree_id: UiTreeId,
        previous: &UiCompiledBindingProgram,
        replacement: &UiCompiledBindingProgram,
    ) -> Result<Self, UiBindingReloadPrepareError> {
        if !previous.is_well_formed()
            || (!previous.generation().is_invalid() && previous.asset_id().is_none())
        {
            return Err(UiBindingReloadPrepareError::MalformedPrevious { tree_id });
        }
        if !replacement.is_well_formed() {
            return Err(UiBindingReloadPrepareError::MalformedReplacement { tree_id });
        }
        if !previous.generation().is_invalid() && replacement.generation().is_invalid() {
            return Err(UiBindingReloadPrepareError::InvalidReplacementGeneration {
                tree_id,
                old_generation: previous.generation(),
            });
        }
        if let Some(old_asset) = previous.asset_id() {
            let Some(new_asset) = replacement.asset_id() else {
                return Err(UiBindingReloadPrepareError::MissingReplacementRootAsset {
                    tree_id,
                    old_asset: old_asset.to_string(),
                });
            };
            if old_asset != new_asset {
                return Err(UiBindingReloadPrepareError::RootAssetMismatch {
                    tree_id,
                    old_asset: old_asset.to_string(),
                    new_asset: new_asset.to_string(),
                });
            }
        }
        if previous.generation() == replacement.generation() && previous != replacement {
            return Err(UiBindingReloadPrepareError::GenerationCollision {
                tree_id,
                generation: previous.generation(),
            });
        }

        let retires_old_generation = !previous.generation().is_invalid()
            && previous.generation() != replacement.generation();
        Ok(Self {
            tree_id,
            old_generation: previous.generation(),
            published_generation: replacement.generation(),
            old_binding_count: previous.binding_count(),
            published_binding_count: replacement.binding_count(),
            retires_old_generation,
        })
    }

    // 仅由执行器在替代表面安装、旧表面释放后消费；迁移计数来自该次表面状态迁移，不在这里重新计算。
    pub(crate) fn publish(
        self,
        published: &UiCompiledBindingProgram,
        state_entries_migrated: usize,
        state_entries_reset: usize,
    ) -> UiBindingQuiescenceReceipt {
        let published_matches_prepared = published.is_well_formed()
            && published.generation() == self.published_generation
            && published.binding_count() == self.published_binding_count;
        let stale_handles_rejected =
            self.retires_old_generation && published.generation() != self.old_generation;

        UiBindingQuiescenceReceipt {
            tree_id: self.tree_id,
            old_generation: self.old_generation,
            published_generation: published.generation(),
            retired_binding_count: if self.retires_old_generation {
                self.old_binding_count
            } else {
                0
            },
            published_binding_count: published.binding_count(),
            state_entries_migrated,
            state_entries_reset,
            old_generation_retired: self.retires_old_generation,
            old_generation_quiescent: self.retires_old_generation
                && published_matches_prepared
                && stale_handles_rejected,
            stale_handles_rejected,
        }
    }
}
