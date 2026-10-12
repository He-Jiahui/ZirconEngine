# Editor01 · 核心工作区按需挂载记录

日期：2026-10-09。对应[原 retained UI 计划](../01-retained-ui-architecture-performance-review.md)。源码已接入；原生测试、产品及性能验收待执行。

## 实际修复

默认 Unreal 工作区不再提前保留十个核心模块页面。现有 Window 菜单选择 Effect、Material、Behavior、Assets、VFX、Ability、Tags、Perception、Render 或 HUD 时，沿项目文档加载链取得所选组件及 named imports，在原 UiSurface 内准备并提交工作区子树。返回 Scene 恢复主视口。

切换沿同一 Surface 的 style、compiled bindings、input、focus、session 与 projection owner 完成。旧模块节点及输入路由被清退，节点 ID 在有界范围内复用。准备失败或公开提交前的 tree/value/style/focus/input/navigation 已改变时，拒绝候选并保留当前工作区；成功提交保留最新私有状态，不在 Debug 中暴露捕获状态。

绘制、区域投影和命中 owner 共同核对 canonical root、文档来源、Surface node、viewport ancestry 与可见正面积 frame。新增回归沿公开选择和真实 typed event 链检查，不能只凭事件派发有返回值判定页面已显示。

## 接入与审查证据

- 实际一次接入 19 个文件：18 个生产文件与原共享事件回归文件，其中四个新源码文件。
- 最终组合 patch SHA-256：`0319498073cc68ed5527f54dce5788b8811e98bcf03387781aaaec844463dd75`。
- 生产 patch：`c9a07d14536a98d325a381e3f5ddac03bc42d0c561903bea7882279b1ee98271`；修正后的 manifest-v2a：`00d2c37d7680de0c9247aaed270d377d74d8d911ba3d956b0a8515ab8634d44a`。
- 独立审查 `editor-module-reachability-core-review-v2/review.json` SHA-256：`0396dacbac95d0bf80a5d7d95a3884564e964fa8e29378b56a0e24064efdbbf3`，源码接受，零阻断，native pending。
- 实际接入 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-module-reachability-fix-v1/actual-canonical-apply-v2a.json` SHA-256：`5f77fb1c2fb16a4085c8c4ff870eb535a93e7e1101e91b4524847493217e6a45`。85 个依赖及四个新路径缺失守卫匹配，实际 LF 后像一致；apply、Rust 2021 fmt、scoped diff 检查 exit 0，共享 Git index/control 未变。

源码增加八项 Editor 回归与一项 Runtime 公开提交回归。原模块事件回归按实际 139 项 typed routes 核对，逐项 has-control 和派发断言保留。139 项中包含六个 Tools open 动作；动作存在不代表弹出菜单已具备。

## 合并验证与开放范围

八项 Editor 用例通过 `module_workspace_mount_tests` 与 `mounted_module_tests` 两个筛选合入既有 UI #15；Runtime 用例合入原 Runtime #2。仍为原 15 项验证配置，既有用例、features、Release producers 和预算保留。

六个 Tools popup 的实际显示、43 项对应扩展页面以及额外 Particle Library 入口仍是后续功能范围。原 96 项强静态可达性断言保持；动态加载需另有真实消费证据，不能添加登记根或整体退休资产来关闭该门。

按需挂载减少默认保留页面，但 stage 快照、候选树、完整加载、切换与清退的 CPU、RSS、allocation requests/bytes、p50/p95/p99/max 尚无实际测量。完整输入、IME、窗口、绘制像素、帧、退场和产品 F0 门按原计划继续开放。

完成状态见[Editor 完成列表](../../../astra/features/editor/2026-10-09-editor01-lazy-core-workspace-completion-list.md)。Runtime 合同见[对应记录](../../zircon_runtime/11a/2026-10-09-transactional-child-mount.md)。
