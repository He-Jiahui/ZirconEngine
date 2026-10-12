# Runtime11A · UiSurface 工作区子树提交记录

日期：2026-10-09。对应[原 UI 架构计划](../11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md)；本切片不关闭 Runtime 产品 UI owner、完整派发、计时、Accessibility 或性能门。

## 实际代码

UiSurface 增加公开的 child mount 准备/提交合同，供 Editor 在原 Surface 内替换所选模块子树。准备阶段构造候选 tree、style 与 bindings；提交前核对公开 live tree/value/style/focus/input/navigation 来源，拒绝陈旧候选。成功路径保留最新 private session/secure clipboard stores，清理退场模块的输入与投影，并复用有界节点 ID。

公开状态在 prepare 后变化时，不能覆盖当前工作区；回归直接修改真实 live property/focus 再调用公开 commit 检查失败原子性。Debug 排除捕获状态。Editor 的绘制和命中继续使用同一个 Surface 与实际文档来源。

## 实际证据

本合同随 Module 一次接入 19 个文件。实际接入 receipt：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-module-reachability-fix-v1/actual-canonical-apply-v2a.json`，SHA-256 `5f77fb1c2fb16a4085c8c4ff870eb535a93e7e1101e91b4524847493217e6a45`。独审 SHA-256 `0396dacbac95d0bf80a5d7d95a3884564e964fa8e29378b56a0e24064efdbbf3` 接受源码，native pending。实际 19 后像、85 依赖前守卫、apply/Rust 2021 fmt/scoped diff 检查通过，共享 index/control 未变。

新增 Runtime 用例 `child_mount_commit_tests::stale_public_child_mount_preserves_live_state_session_and_latest_private_stores` 源码已接入。追加到现有 Runtime #2 的 OR filters，使用原 `target-editor-host,test-support`；不增加配置。实际 native 名称、运行数量及成功结果尚待 receipt。

## 尚待完成

原生公开提交与 Editor 八项回归、实际产品输入/paint/hit/retirement、同配置 before/after CPU/RSS/分配及完整分位数均待验证。准备快照和候选树仍有构建成本；源码按需加载不能代替性能数据。Runtime11A 和 Runtime200 原窗口、多 Surface、IME、Accessibility、帧与全 workspace 门保持开放。

详见[Editor 源码记录](../../zircon_editor/01/2026-10-09-lazy-workspace-child-mount.md)和[Runtime 完成列表](../../../astra/features/runtime/2026-10-09-runtime11a-transactional-child-mount-completion-list.md)。
