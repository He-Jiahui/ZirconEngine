# Editor01：BlendSpace 真实父层与调用者修复记录

日期：2026-10-09。状态：三文件源码修复已采用；原生功能、产品与性能待验证。

原测试及截图调用者改为真实 Window popup→Ability→typed BlendSpace 选择，加载实际 animation/workbench_extension_blend_space_workspace.zui，使用 WorkbenchMainBandViewportPanel 下的 UnrealSceneViewViewport。当前 Source、render frame、共享 host paint/hit 和真实窗口呈现 owner 均按既有生产入口取得。

原 compact / wide 几何、Grid9、6 ticks / 8 points RunFwd、timeline、heatmap16×10、五权重归一，以及实际绘制颜色阈值 4/8/4 完整保留。移除的孤立 painter 仅是未断言 diagnostic；原 ignored capture 名称、body 和 ignore 状态保持。没有修改生产路径、实体资产或 all-component 强门。

[独立源审](../../../../../.codex/outbox/editor-blend-space-real-parent-callers-core-review-v1/review.json)和[实际采用](../../../../../.codex/outbox/editor-blend-space-real-parent-callers-v1/after-core-actual-v3/actual-canonical-apply.json)：98 双次输入核对、95 未改依赖、实际三文件格式与 scoped diff 通过；共享索引和既有索引锁内容及元数据保持。采用 SHA-256：7056bb8a12e0f5b6472a8ec93a546c8a8444bfcd0961c2859d62fbb02b85cb1c。

原 28 普通用例在 V8 #15 的 OR filters 中无重叠，最终 V9 在同 arm 追加这 28 个精确名称，最低 146→174；它们均是现有用例，新增测试为零。ignored 截图不激活，不以截图 helper 代替真实 Editor 验收。15 arms、14 原性能 producer、features / profiles / 原阈值保持。

Generated、两个 paint/hit fixture 的锁修正及 strong gate 仍是候选；三个旧聚合资产迁移与实体退役、完整性能和真实产品资格保持未完成。没有 native 执行或协调器提交声明。

对应[计划完成列表](../../../astra/features/editor/2026-10-09-editor01-blend-space-caller-completion-list.md)。
