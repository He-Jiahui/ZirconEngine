# UI Layout 架构评审与 Taffy 收敛计划

## Summary
- 结论：当前 layout 方向总体合理，已经走向 Slate-style retained tree：`measure -> arrange -> UiSurfaceFrame -> render/hit/input` 同源。
- 不需要“再接入 Taffy 插件”。`zircon_runtime` 已直接依赖 `taffy 0.10`，并已有 Taffy bridge；Taffy 应作为 runtime core 的布局后端，不应做成可选插件。
- 推荐路线：保留 **Taffy + Zircon 自管布局** 混合架构。Flex/Grid/Wrap/Block 继续向 Taffy 收敛；Overlay、Canvas/Free、Scrollable、VirtualList、Editor docking/splitter 继续由 Zircon 掌控。

## Key Findings
- 已接入：`taffy` 依赖在 [zircon_runtime/Cargo.toml](/E:/Git/ZirconEngine/zircon_runtime/Cargo.toml:86)，桥接入口在 [taffy_bridge.rs](/E:/Git/ZirconEngine/zircon_runtime/src/ui/layout/taffy_bridge.rs:6) 和 [taffy_arrange.rs](/E:/Git/ZirconEngine/zircon_runtime/src/ui/layout/pass/taffy_arrange.rs:20)。
- 当前 Taffy 接入还偏“局部 solver”：每个可支持容器临时建 `TaffyTree`，而不是像 Bevy 那样维护 surface 级 Taffy tree/cache；复杂 slot、anchor/pivot/position、grid placement、canvas placement 会直接 fallback。
- 自研算法不是废案：`AxisConstraint` 的 priority/weight/stretch 语义对 Editor workbench、drawer、splitter 和 Slate-style slot 很有用，不能被纯 CSS flex/grid 直接替换。
- 与 Taffy 设计的距离：Taffy 官方定位是 tree-based、Flex/Grid/Block layout engine，支持 custom measurement；当前 Zircon 只映射了基础 flex/grid/wrap/gap/min/max/preferred，还没有完整 padding/margin/align/justify/aspect/measure callback/style tree 映射。参考：[Taffy docs](https://taffylayout.com/docs) 与 [docs.rs taffy 0.10.1](https://docs.rs/taffy)。

## Implementation Changes
- 将现有桥接明确升格为 `UiLayoutEngine` 后端策略：`UiLayoutEngineSelectionReport` 记录每个 surface/container 是 Taffy native、Zircon fallback 还是 unsupported。
- 扩展 Taffy style conversion：把 Zircon slot padding/alignment、linear sizing、grid placement、layout metrics、text/image measurement 分阶段映射进 Taffy；仍禁止 Taffy 接管 Overlay/Scrollable/VirtualList/Editor docking 语义。
- 保持 `UiSurfaceFrame` 为唯一空间事实：无论 Taffy 或 Zircon 求解，输出都必须写回同一 arranged tree，再由 render extract、hit grid、pointer route、debug reflector 消费。
- 清理“半接入”风险：对 fallback 原因加诊断，不允许静默退回 legacy solver；重要 editor pane 和 runtime `.ui.toml` fixture 需要能显示 Taffy/Zircon 路由结果。

## Test Plan
- Runtime focused tests：Flex、Grid、Wrap、Block Taffy 路由；slot padding/alignment、linear sizing、grid placement 的 Taffy/parity cases；fallback reason 覆盖 Overlay/Canvas/Scrollable/VirtualList。
- Authority tests：同一节点的 arranged frame、render command、hit entry、pointer path 在 Taffy 与 Zircon backend 下保持一致。
- Editor integration tests：Workbench shell、drawer、viewport toolbar、floating window、runtime UI fixture 不因 Taffy 路由发生坐标漂移。
- Validation note：本次尝试运行 `cargo test -p zircon_runtime taffy`，冷编译 10 分钟超时，未取得通过证据；后续实现阶段需用独立 target dir 完成 focused 和 workspace 门禁。

## Assumptions
- 默认目标是架构评审和后续收敛计划，不在本轮直接改代码。
- 不新增第四个 root crate，不新增 layout 插件包；Taffy 保持为 `zircon_runtime` 内部后端依赖。
- 以当前主线计划中的混合方案为准：Taffy 负责通用 CSS-like layout，Zircon 保留引擎/editor 专属语义。
