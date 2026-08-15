# `zircon_editor` 差距审查

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

待审范围：authoring state 与 runtime world 分离、transaction/undo-redo、selection、scene/prefab、asset import/reimport、viewport、inspector、tool mode、command routing、layout persistence、扩展 API、崩溃恢复和大型项目工作流。

编辑器 UI 与 transaction 目录当前有其他活跃会话修改；后续深审必须基于新快照复核，不在本轮读取中推断结论。

## 编号审查

- [01 · Editor Retained UI 架构与性能审查](01-retained-ui-architecture-performance-review.md)：事件、布局、命中、绘制、文本/图像缓存、presenter生命周期与工程级性能验收。
