# Editor01 · Particle Library 按需挂载

日期：2026-10-09。承接[核心工作区记录](2026-10-09-lazy-workspace-child-mount.md)，对应[原 retained UI 计划](../01-retained-ui-architecture-performance-review.md)。源码已接入，原生、产品与性能验收待执行。

## 实际修复

公开 VFX 页面入口通过现有 typed callback 和 Controller 导航，按需加载 `res://ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui` 的 `WorkbenchExtensionParticleLibraryWorkspace`。提交沿现有同一 UiSurface 的 stage/commit，复用当前 pane、来源、style、input/session 与 projection owner。

真实来源约束下的可见正面积 frame 同时用于 pane 投影、paint 与 hit；入口与 15 个 authored local body actions 沿真实 callback 进入现有 Controller/journal。local preview 准入和 journal sequence 不等于主窗口已呈现，也不代表完整粒子 backend 功能已经完成。

返回核心模块或 Scene 时清退该 child 的节点、focus、输入路由及存储。加载或公开提交拒绝时保留当前工作区、真实 paint/hit 与输入状态；沿用原子失败合同。

## 接入与审查

- 一次实际接入六个文件，含一个新回归文件；四项新增 Rust cases 加一项既有负例迁移。
- 最终 v4b patch SHA-256：`caf70924d0db6e39226eef8c68f772136fa468afcf2a5552d49c19c95c480f35`；binding：`01781931d9de240b590ad1b59d3c2ce56ef4057435b37d10a2e2d498f2e0f69f`。
- 386-input binding：`d884ca72487f43d5b01fedab1b118c417f07d1194252ed576245c11e3b02a013`；独审 `editor-particle-library-lazy-mount-core-review-v4b/review.json`：`65f9542115e3e8c4e354ce9f6bcf4e7beb5441aa5a7716372cfd2d51f6ba3c4a`，源码接受，native/performance pending。
- 实际回执 `E:/Git/ZirconEngine/.codex/outbox/editor-particle-library-lazy-mount-v4/actual-canonical-apply-v4b.json`：`3d55232d36965b03cdfbe3db541ab3b2d509a5ad7888094aecbfe73d55f0cc0e`。386 守卫含 385 个已有 raw 与一个新路径缺失，六 LF 后像匹配，380 未受影响守卫保持；apply/check、实际 Rust 2021 fmt、scoped diff exit 0，共享 index/control 未变。

并行 viewport overflow 的字段、typed consumer、focus reconciliation 和 popup 逻辑完整保留。早期 v4a 与 0e49 记录只作历史；不声称旧 viewport 全字段或 geometry 与当前等值。实际 geometry/paint-hit/popup 轨迹仍需 native。

## 合并验证与开放范围

原 UI #15 追加 extension_workspace_mount_tests 筛选，四新增 cases 与既有 mounted_module_tests 同批。已实现 Particle 后，旧 unavailable VFX 负例转到仍不可用的合法 Shader/Render consumer，保留原实际来源、paint/hit 和失败原子性断言。Tools 实现该页面后还需迁移到真正公开 stale-commit 场景。

草案 v6 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v4/commands-draft-v6.json` SHA-256：`2fa9d2145ed07044256d52cd5e9ddfa350677a5aaae1e659df3afc50566b20fe`。维持 15 个原 arms、原 features/profiles/targets；UI 数量下界 132→136 属源码预期，实际名字、非零计数与最终新鲜 source binding 待验。原 139 项逐事件断言与 96 项强静态门保持，动态挂载不赋登记根信用。

43 个 Tools 扩展、真实 Generated owner、21 项 catalog preview、legacy caller/资产闭环及全部组件实际消费仍待完成。原 CPU/RSS/allocation requests/bytes、p50/p95/p99/max、像素、帧、IME/窗口和真实产品 F0 标准保持；stage 快照、加载与切换开销尚无实测。

详见[Editor 完成列表](../../../astra/features/editor/2026-10-09-editor01-particle-library-completion-list.md)。
