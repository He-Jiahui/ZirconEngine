---
related_code:
  - docs/plans/mvp/index.md
  - zircon_app/src/entry/entry_runner/editor
  - zircon_runtime/src/scene
  - zircon_editor/src
  - zircon_runtime/src/dynamic_api
implementation_files:
  - docs/plans/mvp
  - zircon_app/src/entry/entry_runner
  - zircon_runtime/src/scene
  - zircon_editor/src
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/minimum-viable-engine-foundation.md
tests:
  - zircon_app/src/entry/entry_runner/editor/tests
  - zircon_runtime/src/scene
  - zircon_editor/src
doc_type: acceptance-reference
---

# MVP 产品验收

当前 MVP 计划状态是 `in_progress`。本页描述退出条件，不宣称这些条件已经全部通过。

## 固定闭环

1. 从产品入口创建或打开 `RenderableEmpty` 项目。
2. 从磁盘权威加载项目资产 registry 和 settings。
3. 默认场景包含 camera、引用持久资产的可见 primitive 和 directional light。
4. Runtime 接收 keyboard/mouse 输入，产生非空 WGPU 帧并可确定性退出。
5. Editor 从 Hierarchy 选择 primitive，通过 Inspector 或等价 UI binding 进入 command/transaction 路径修改 transform。
6. 保存项目，销毁当前 host/session，再打开同一项目。
7. 重开后观察到同一实体、资产引用和修改后的 transform。
8. 同一验证副本上的产品二进制连续运行两次，均有可归因诊断并干净退出。

## 证据要求

直接改 TOML、直接改 `World`、测试专用旁路、静态源码扫描或单个单元测试都不能替代步骤 1-8。验收记录必须包含 source digest、profile/feature、产品命令、两次运行的 exit code、帧/输入证据、保存文件和重开观察值。

## 失败路由

按最底层共享原因修复：解析/registry -> Runtime scene/resource -> Editor command/session -> App host。跨 owner 的 failure 写入对应编号计划目录；MVP 目录只链接并标记阻断状态。完成前不要把“局部测试通过”写成 F0-F5 完成。
