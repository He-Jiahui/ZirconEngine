//! 跨 App session 与 Editor gateway 共享的视口表面绑定状态。
//!
//! 调用者在执行 `bind_viewport_surface` / `unbind_viewport_surface` ABI 之前保留短期 transition
//! token，ABI 调用期间不持有 registry 锁；调用返回后用结果提交状态，提前退出则由 token 的
//! `Drop` 回滚。App teardown 通过 `bound_viewports` 按稳定 handle 顺序逐个解绑。

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use zircon_runtime_interface::ZrRuntimeViewportHandle;

/// Shared host-side ownership for viewport-to-native-surface bindings.
///
/// A caller reserves a transition, makes the ABI call without holding this
/// registry's mutex, then completes the transition with the call result. If a
/// caller exits before completion, the transition token restores the last
/// published state when it is dropped.
///
/// 该 registry 由同一 Runtime session 的 App 和 Editor gateway 共享。每个 viewport 同时只有
/// 一个绑定/解绑操作可在途；查询只返回已发布为 Bound 的视口。
#[derive(Default)]
pub struct ViewportSurfaceBindings {
    /// 保护 viewport 状态表；锁仅覆盖状态预约、完成或回滚，不跨越 ABI 调用。
    registry: Mutex<ViewportSurfaceBindingRegistry>,
}

#[derive(Debug, Default)]
/// 每个视口的当前发布状态及在途转换。
struct ViewportSurfaceBindingRegistry {
    /// 缺席表示从未发布绑定；在途态只由相应操作 token 创建。
    states: HashMap<ZrRuntimeViewportHandle, ViewportSurfaceBindingState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 单个 viewport 的已发布绑定或互斥转换状态。
enum ViewportSurfaceBindingState {
    /// Runtime 表面绑定已成功发布。
    Bound,
    /// Bind/rebind ABI 调用正在执行。
    Binding,
    /// Unbind ABI 调用正在执行。
    Releasing,
}

/// A reserved bind or rebind operation.
///
/// Dropping an unfinished operation restores its preceding published state.
///
/// token 借用 registry，且必须在对应 Runtime ABI 调用结束后调用 `finish`；丢弃 token 会恢复
/// 预约前是 Bound 还是未绑定。
#[must_use = "an unfinished binding operation is rolled back when dropped"]
#[derive(Debug)]
pub struct ViewportSurfaceBindingOperation<'a> {
    /// token 生命周期内用于提交或回滚的共享状态表。
    registry: &'a Mutex<ViewportSurfaceBindingRegistry>,
    /// 正在绑定或重绑定的视口。
    viewport: ZrRuntimeViewportHandle,
    /// 预约前的已发布绑定状态；用于失败/提前退出后的回滚。
    was_bound: bool,
    /// 阻止 finish 后 Drop 再次回滚。
    completed: bool,
}

/// A reserved unbind operation.
///
/// Dropping an unfinished operation restores the published binding.
///
/// token 借用 registry，且须在 Runtime unbind ABI 调用结束后完成；失败或提前退出会保留原绑定。
#[must_use = "an unfinished release operation is rolled back when dropped"]
#[derive(Debug)]
pub struct ViewportSurfaceReleaseOperation<'a> {
    /// token 生命周期内用于提交或回滚的共享状态表。
    registry: &'a Mutex<ViewportSurfaceBindingRegistry>,
    /// 正在解绑的视口。
    viewport: ZrRuntimeViewportHandle,
    /// 阻止 finish 后 Drop 再次回滚。
    completed: bool,
}

/// Reports that a viewport already has a transition in progress.
///
/// 调用方可从错误中取回冲突视口，用于把内部状态冲突转换成上层错误。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportSurfaceOperationInFlight {
    viewport: ZrRuntimeViewportHandle,
}

impl ViewportSurfaceOperationInFlight {
    /// 返回当前已有操作占用的视口 handle。
    pub const fn viewport(self) -> ZrRuntimeViewportHandle {
        self.viewport
    }
}

impl ViewportSurfaceBindings {
    /// Reserves a bind or rebind for `viewport`.
    ///
    /// ## 参数
    ///
    /// `viewport` 是 Runtime ABI 绑定表面的视口 handle。
    ///
    /// ## 返回值
    ///
    /// 返回必须在 ABI 调用后完成的预约 token；若同一视口已有 bind/unbind 在途，则返回
    /// [`ViewportSurfaceOperationInFlight`]。已有 Bound 状态允许 rebind，并在失败时保留旧绑定。
    pub fn begin_binding(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<ViewportSurfaceBindingOperation<'_>, ViewportSurfaceOperationInFlight> {
        let mut registry = self.lock_registry();
        let was_bound = match registry.states.get(&viewport) {
            None => false,
            Some(ViewportSurfaceBindingState::Bound) => true,
            Some(ViewportSurfaceBindingState::Binding | ViewportSurfaceBindingState::Releasing) => {
                return Err(ViewportSurfaceOperationInFlight { viewport });
            }
        };
        registry
            .states
            .insert(viewport, ViewportSurfaceBindingState::Binding);
        Ok(ViewportSurfaceBindingOperation {
            registry: &self.registry,
            viewport,
            was_bound,
            completed: false,
        })
    }

    /// Reserves an unbind for `viewport`.
    ///
    /// `Ok(None)` means the viewport has no published binding and no ABI call
    /// is required.
    ///
    /// 对已绑定视口返回 release token，须在 unbind ABI 返回后调用其 `finish`；对缺席视口返回
    /// `Ok(None)`，以便 teardown 可以安全跳过无绑定项。
    pub fn begin_release(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<Option<ViewportSurfaceReleaseOperation<'_>>, ViewportSurfaceOperationInFlight> {
        let mut registry = self.lock_registry();
        match registry.states.get(&viewport) {
            None => Ok(None),
            Some(ViewportSurfaceBindingState::Bound) => {
                registry
                    .states
                    .insert(viewport, ViewportSurfaceBindingState::Releasing);
                Ok(Some(ViewportSurfaceReleaseOperation {
                    registry: &self.registry,
                    viewport,
                    completed: false,
                }))
            }
            Some(ViewportSurfaceBindingState::Binding | ViewportSurfaceBindingState::Releasing) => {
                Err(ViewportSurfaceOperationInFlight { viewport })
            }
        }
    }

    /// Returns all currently bound viewports in deterministic teardown order.
    /// 返回已发布绑定的视口，并按原始 handle 升序排列供确定性 teardown 使用。
    pub fn bound_viewports(&self) -> Vec<ZrRuntimeViewportHandle> {
        self.lock_registry().bound_viewports()
    }

    /// 在同一个 registry 上取得锁并恢复 poisoned mutex 中保留的状态。
    fn lock_registry(&self) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
        lock_registry(&self.registry)
    }
}

impl ViewportSurfaceBindingOperation<'_> {
    /// Commits or rolls back the reserved bind after the ABI call returns.
    ///
    /// Returns whether at least one viewport remains bound.
    ///
    /// `succeeded` 应是对应 Runtime bind/rebind ABI 的返回结果。首次绑定失败时删除预约项；
    /// rebind 失败时恢复原绑定；成功时发布 Bound。返回值用于上层确认仍有任一已绑定表面。
    pub fn finish(mut self, succeeded: bool) -> bool {
        let any_bound = {
            let mut registry = self.lock_registry();
            if !matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Binding)
            ) {
                registry.has_bound_viewports()
            } else {
                if succeeded || self.was_bound {
                    registry
                        .states
                        .insert(self.viewport, ViewportSurfaceBindingState::Bound);
                } else {
                    registry.states.remove(&self.viewport);
                }
                registry.has_bound_viewports()
            }
        };
        self.completed = true;
        any_bound
    }

    /// 只在读取或提交该 token 对应的状态时持锁。
    fn lock_registry(&self) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
        lock_registry(self.registry)
    }

    /// 若尚未完成且状态仍为 Binding，恢复预约前状态并标记 token 完成。
    fn rollback(&mut self) {
        if self.completed {
            return;
        }
        {
            let mut registry = self.lock_registry();
            if matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Binding)
            ) {
                if self.was_bound {
                    registry
                        .states
                        .insert(self.viewport, ViewportSurfaceBindingState::Bound);
                } else {
                    registry.states.remove(&self.viewport);
                }
            }
        }
        self.completed = true;
    }
}

impl Drop for ViewportSurfaceBindingOperation<'_> {
    /// 回滚未显式完成的 bind/rebind 预约。
    fn drop(&mut self) {
        self.rollback();
    }
}

impl ViewportSurfaceReleaseOperation<'_> {
    /// Commits or rolls back the reserved unbind after the ABI call returns.
    ///
    /// Returns whether at least one viewport remains bound.
    ///
    /// `succeeded` 应是对应 Runtime unbind ABI 的返回结果。成功删除发布项，失败恢复 Bound；
    /// 返回值指出操作后是否仍有任一已绑定表面。
    pub fn finish(mut self, succeeded: bool) -> bool {
        let any_bound = {
            let mut registry = self.lock_registry();
            if !matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Releasing)
            ) {
                registry.has_bound_viewports()
            } else {
                if succeeded {
                    registry.states.remove(&self.viewport);
                } else {
                    registry
                        .states
                        .insert(self.viewport, ViewportSurfaceBindingState::Bound);
                }
                registry.has_bound_viewports()
            }
        };
        self.completed = true;
        any_bound
    }

    /// 只在读取或提交该 token 对应的状态时持锁。
    fn lock_registry(&self) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
        lock_registry(self.registry)
    }

    /// 若尚未完成且状态仍为 Releasing，将该视口恢复为 Bound。
    fn rollback(&mut self) {
        if self.completed {
            return;
        }
        {
            let mut registry = self.lock_registry();
            if matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Releasing)
            ) {
                registry
                    .states
                    .insert(self.viewport, ViewportSurfaceBindingState::Bound);
            }
        }
        self.completed = true;
    }
}

impl Drop for ViewportSurfaceReleaseOperation<'_> {
    /// 回滚未显式完成的 unbind 预约。
    fn drop(&mut self) {
        self.rollback();
    }
}

impl ViewportSurfaceBindingRegistry {
    /// 判断当前是否至少有一个已发布为 Bound 的视口。
    fn has_bound_viewports(&self) -> bool {
        self.states
            .values()
            .any(|state| *state == ViewportSurfaceBindingState::Bound)
    }

    /// 过滤出已绑定视口，并按 raw handle 排序。
    fn bound_viewports(&self) -> Vec<ZrRuntimeViewportHandle> {
        let mut viewports = self
            .states
            .iter()
            .filter_map(|(viewport, state)| {
                (*state == ViewportSurfaceBindingState::Bound).then_some(*viewport)
            })
            .collect::<Vec<_>>();
        viewports.sort_unstable_by_key(|viewport| viewport.raw());
        viewports
    }
}

/// 获取 registry 锁；若先前持锁线程 panic，则沿用仍可访问的状态。
fn lock_registry(
    registry: &Mutex<ViewportSurfaceBindingRegistry>,
) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
    registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
#[path = "tests/viewport_surface.rs"]
mod tests;
