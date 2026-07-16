---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/scene/viewport
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_app/src/bin/editor.rs
  - zircon_hub/src
  - tools/zircon_build.py
reference_sources:
  - dev/Fyrox/editor/src/lib.rs
  - dev/godot/editor
  - dev/UnrealEngine/Engine/Source/Editor
  - dev/bevy/crates/bevy_remote
  - dev/theatre
plan_sources:
  - docs/plans/zircon_editor/editor_layout/index.md
  - docs/plans/zircon_editor/editor_ui/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/zircon_runtime/frameworks/index.md
status: planned
---

# Zircon Editor 基础编辑器框架计划总览

本目录是「编辑器框架与功能域」的权威计划集：编辑器内核如何组织、如何与 runtime 交互、如何承载 PIE / 撤销 / 插件 / 资产 / 领域编辑器 / 发行等编辑器级能力。

与同级计划集**平行而不重复**：

- `editor_ui/`：运行时 UI 能力（布局引擎、输入派发、样式、组件目录、壳级承载）。本目录不重复规划 UI 渲染栈。
- `editor_layout/`：编辑器外观组织（设计语言、JetBrains 式停靠、布局预设声明、增量消息刷新）。本目录只定义**扩展点注册与功能域接入**，外观与停靠实现引用 `editor_layout/` 对应计划。

## 参考引擎路由（依 zr-reference-engine-routing）

| 问题域 | 主参照 | 副参照 |
| --- | --- | --- |
| 编辑器主循环 / editor-runtime 分层 | `dev/Fyrox/editor`（`Editor` + `Message` MPSC 主循环） | 当前仓库 `EditorModule` |
| PIE / 事务 / 资产编辑器 / 动画与行为树等重量级功能族 | `dev/UnrealEngine`（PIE world duplication、`FTransaction`、`FAssetEditorToolkit`、Persona、BehaviorTreeEditor） | Fyrox / godot 稳定 Rust 落点 |
| 撤销分层 / 编辑器插件扩展点 / 运行子进程 | `dev/godot/editor`（`EditorUndoRedoManager` 多 history、`EditorPlugin` 家族、`editor/run`） | UnrealEngine |
| 远程会话协议 / 反射 / 资产句柄 | `dev/bevy`（`bevy_remote` JSON-RPC、`bevy_reflect`、`bevy_asset`） | — |
| 时间轴 / 动画创作 UX | `dev/theatre` | `dev/slint` |

## 现状基线（2026-07-05 审阅，取证后修订）

- 已有：`EditorModule`（Lazy 启动，依赖 Foundation/Asset/Scene/Graphics/UI 五模块）；**消息总线 `EditorMessageBus` 已存在**（订阅/收件箱/`ViewDirtySet`，但载荷仅 `Empty|Text`）；场景撤销栈 `EditorHistory`（128 条 + gizmo 拖拽合并）、UI 资产撤销栈 `UiAssetEditorUndoStack`（serde 可回放）、操作层 `EditorOperationRegistry/Stack` **三套撤销并存**；`EditorEventRuntimeState` 单锁聚合 14 字段（事实上的内核，形态错位）；**扩展注册表 `EditorExtensionRegistry` 已有 13 类描述符表**（views/drawers/menu/component_drawers/asset_*/viewport_tool_modes/graph_*/timeline_*）；`ViewHost` 四态宿主 + `LayoutPreset` 四内建预设（**但 persistence.rs 为无 IO 空实现**）；命令注册表 `EditorCommandRegistry`（含 `command_palette_entries(context)`）与 `EditorKeymap`（TOML）；**play mode backend trait 已存在但语义是 native 插件活性快照切换，非运行游戏**；导出向导**八阶段枚举已存在**（Validate…CookAssets/Pack…Report）；**`--headless --operation` 无头 CLI 已存在**；hub 以 `["--project", path]` fire-and-forget 启动 editor。
- runtime 契约面成熟：`zircon_runtime_get_api_v2` → `ZrRuntimeApiV2` 19 字段函数表，包含 plugin-event mirror 与 operation submit/poll/harvest；session 五态 profile 且**每 session 独立 `CoreRuntime`+`LevelSystem`**（进程内 PIE 的天然隔离底座）；`LevelSystem::snapshot()/replace()`；`WorldInspection`；`DynamicScene{format_version}` 热重载队列 + `AssetReloadFrameApplyReport`；`ZrHostApiV3` 五域 ABI；`JobScheduler::schedule_after`（依赖调度内建）；`ResourceHandle<TMarker>` 21 marker；asset 域已有 `watch/pack/project` 模块与 19 个导入器。
- 主要缺口：无 Edit/Play 状态机与游戏运行（现 backend 是插件桥接切换）；撤销三套未收敛且无路由；消息载荷空心；13 张扩展表无统一生命周期（无 revoke）、无 inspector 字段级容器、无 DocumentToolkit；无 GUID 体系（`AssetUuid` 孤岛两处）与离线 registry；无迁移管线（版本策略三种并存）；无 job 门面（导出向导手工线程是孤例）；hub 无握手/回写/单实例；无设置分层/自动保存/日志汇聚/本地化。

## 计划清单与依赖

编号即建议执行序；括号内为强依赖。

0. `00-editor-architecture-overview.md` — **总体架构定形**（分层/聚合根/线程模型/帧数据流/事实源表/目录终态；各分计划的公共骨架，冲突时修订它并回改分计划）
1. `01-editor-kernel-and-runtime-interaction.md` — 编辑器内核与 runtime 交互门面（基座）
2. `02-data-sync-and-messaging.md` — 信息与数据同步（依赖 01）
3. `03-command-transaction-and-undo.md` — 命令 / 事务 / 撤销统一框架（依赖 01）
4. `04-pie-and-simulation.md` — PIE 模拟运行与 Unity 式运行时编辑（依赖 01、02；M3 实时同步依赖 02 M2/M3，M4 运行时编辑依赖 03 M2、05）
5. `05-scene-editing-hierarchy-and-gizmos.md` — 编辑场景 / hierarchy / gizmos（依赖 01、03）
6. `06-ui-extension-framework.md` — 编辑器 UI 扩展框架：drawer / window / inspector / field / 自定义区域 / 布局预设（依赖 01）
7. `07-domain-editors-and-graph-foundation.md` — 图编辑基座与领域编辑器：动画 / montage / 状态机 / 行为树 / 预览（依赖 03、06）
8. `08-tool-orchestration-and-commands.md` — 工具管理调度 / 命令系统 / 命令面板（依赖 01、06）
9. `09-editor-asset-management.md` — 编辑器资产管理（依赖 01、14）
10. `10-project-and-asset-reference-management.md` — 文件工程与资产引用管理（依赖 09）
11. `11-serialization-and-versioning.md` — 数据序列化与版本迁移（横切，宜早）
12. `12-plugin-management.md` — 编辑器插件管理（依赖 06、08）
13. `13-script-compilation-management.md` — 脚本编译管理（依赖 14；衔接 runtime/13）
14. `14-threading-and-job-scheduling.md` — 多线程调度管理（基座，宜早）
15. `15-build-export-and-publishing.md` — 发行与生成（依赖 09、10、12）
16. `16-cli-args-and-hub-integration.md` — 控制台入口参数与 zircon_hub 交互（依赖 01）
17. `17-editor-services-and-recovery.md` — 设置分层 / 自动保存 / 崩溃恢复 / 日志诊断等必要服务（横切）

建议波次：**W1 基座** = 01、11、14 → **W2 核心编辑** = 02、03、05、06 → **W3 功能域** = 04、07、08、09、10 → **W4 生态与交付** = 12、13、15、16、17。

## 全局硬约束（各分计划继承，违反即返工）

- 三包定形：`zircon_app` 组合与引导、`zircon_runtime` 世界权威、`zircon_editor` 创作权威、`zircon_runtime_interface` 中立契约。编辑器状态不得进入 runtime 序列化（既有 authoring token 守卫必须保持通过）。
- 动态边界只传 ABI 安全值与序列化载荷；不得跨界传 Rust trait 对象 / `wgpu` / Slint 对象 / runtime world 引用。
- 硬切换：新 owner 路径落地即迁移调用方并删除旧路径，不留兼容层、facade 目录或临时 `pub use` 桥。
- 根接线文件（`lib.rs`/`mod.rs`/`main.rs`）保持薄；深行为放 owner 模块。
- 非网络语义禁用 `server` 命名。
- 计划执行遵循 milestone-first：切片期只做代码/文档 + 轻量 `cargo check`，里程碑边界进入测试阶段统一编译/测试/修复并记录证据。

## 取证口径

各分计划的「参照证据（dev/）」与「现状与证据（zircon）」节均基于 2026-07-05 对源码的实读（类型/签名/枚举带文件路径与行号）。行号随迭代漂移属预期——执行任一计划前，按其证据节逐条重核（Grep 类型名即可），失实处修订计划再动工。UnrealEngine 部分证据（PIE 内部流程）为头文件级 + 公开文档确认，执行 04 计划前宜再读 `Editor/UnrealEd/Private/PlayLevel.cpp` 深核。

**v3 逐篇细化（同日二轮取证）推翻的初版判断**（各分计划证据节已内嵌修正标注，此处汇总备忘）：hierarchy 行已带 `entity/parent` 稳定锚（02，缺的只是世代号与 subtree_hash）；Move/Rotate/Scale HandleTool 三实现与吸附/坐标系设置（`grid_mode/translate_step/transform_space`）已存在（05，缺的是模式生命周期与多选/事务化）；扩展注册已有**批模型**（`EditorExtensionRegistration` + 能力门控，06）；菜单物化 `menu_bar_model/menu_model` 已存在且命令已单向链接操作（08）；`.zmeta` sidecar + `AssetMetaDocument{uuid,url,kind}` 已在、uuid 扫描期已铸造（09/10）；`ProjectManifest`（`zircon-project.toml`）已含 plugins/scripts/export_profiles（10）；`DynamicScene::ensure_supported()` 版本检查钩子已在（11）；`EditorPluginLifecycleStage` 十值词汇已定义（12，缺点火接线）；导出平台词汇 `ExportTargetPlatform` 八值已定型（15）。共同模式：**注册面/词汇/数据通道大多已备，缺的是生命周期、消费闭环与跨子系统接线**——各计划均已按此重新定界。

## 通用测试阶段基线

各分计划的测试阶段默认包含（按需增补）：

```bash
cargo check -p zircon_editor --lib --locked
cargo test -p zircon_editor --lib --locked
cargo test -p zircon_runtime --lib --locked   # 触及 runtime 契约面时
cargo test -p zircon_runtime_interface --locked  # 触及 ABI/DTO 时
cargo fmt --all --check
```

验收证据与状态记录写回各计划文档状态节；涉及的行为模块按源路径镜像更新 `docs/zircon_editor/**`。
