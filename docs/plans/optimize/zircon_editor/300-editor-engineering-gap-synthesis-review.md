---
related_code:
  - zircon_editor/src/core
  - zircon_editor/src/scene
  - zircon_editor/src/ui
  - zircon_app/src
implementation_files:
  - zircon_editor/src/core
  - zircon_editor/src/scene
  - zircon_editor/src/ui
  - zircon_app/src
---

# Editor 工程化差距综合审查（Review-only）

## 结论

Editor 已有 retained host、document revision、transaction、selection、viewport、asset catalog、plugin contribution、runtime gateway 和部分 recovery 底座，但大量产品 surface 仍只是 descriptor、静态 projection、collapsed/Space 内容或没有 factory/caller 的 operation。核心差距不是缺少 UI 控件，而是 Editor 没有把 authoring document、Runtime effect、compiled artifact、Preview/PIE world、generation 和 durable receipt 统一起来。

## P0：产品正确性与数据安全

### ED-P0-01：文档、World、Runtime session 不是同一条 authoring authority

`zircon_editor/src/lib.rs:1-41` 只做 crate facade；真正的 project/session/document/scene/viewport 行为分散在 `core`、`scene`、`ui`。现有 Editor247/253/256/268 报告确认 Editor world clone、Play snapshot、Inspector projection、Runtime gateway 和多套 selection/history cache 并存。Editor 关闭 tab、Close Project、Play/Simulate、切换 scene 的 dirty gate、generation、history retirement 与 Runtime session shutdown 不能由单一 receipt 证明。

**重构：** 建立 `EditorProjectAuthority -> DocumentRegistry -> AuthoringTransaction -> RuntimePreviewSession -> EffectReceipt` 链；所有窗口/视图只消费 generation-qualified immutable snapshot。关闭/切换必须先冻结输入、处理 dirty/recovery、停止 operation、等待 runtime effect，再退休 document/session。

### ED-P0-02：大量 surface 宣称可用但无真实 provider/handler

Editor 现有目录中可见 Prefab、Physics、AI、Gameplay、Terrain、Particles、Rendering、Lighting、Media、Localization 等功能面，但现有报告反复确认它们存在 descriptor、capability 或固定 Workbench 文案，却没有 operation factory、document mutation、compiler job、artifact install 或 Runtime consumer。`zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs:279` 等代码还出现固定 fixture 数据投影风险；`scene_projection.rs:377` 的 placeholder path 也表明表面内容可脱离 domain authority。

**重构：** 未闭合 domain 必须显式 `Unavailable/Prototype`，禁止 `stable/complete`、静态 `queued/success` 或固定业务数字。每个 Available domain 必须同时提供 provider、typed document、transaction、operation factory、job/receipt、preview/runtime mirror、save/reopen 和 failure path。

### ED-P0-03：保存/恢复不能证明无损与原子

Editor266/265/225/256 已确认多个 writer、autosave、journal、animation/UI asset 私有 history 和 ProjectAuthority 各自实现 staging/rename/dirty/recovery。Scene load/save 对 Prefab、TileMap、Sprite2D、Skeleton 等仍可能丢字段；Save 成功与 projection/Runtime apply 失败的终态也可能被压平。

**重构：** 由唯一 `EditorPersistenceAuthority` 管理 source/document/artifact/autosave/journal/recovery，采用 revision/CAS、multi-file transaction、directory durability receipt、last-known-good 和明确恢复决策。禁止私有无界 undo stack、裸 `fs::write` 和先清 dirty 再写盘。

## P1：规模和可扩展性

1. **Scene/Viewport**：selection、highlight、gizmo、picking 和 displayed frame 仍可能只看 primary、固定 pointer/camera 或 clone 全树；需 per-window/per-pane `DisplayedFrameIdentity`、shared spatial query、批量 qualified mutation 和 overlay composition。
2. **Asset workflow**：catalog/index/detail/preview/import/reimport 有多套 generation、source/provider 猜测和全量 rebuild；需 exact AssetTypeId、lazy/paged projection、preview budget/LRU、provider admission 和 Runtime-owned operation receipt。
3. **Plugin/provider**：catalog 只装配少数 provider，unknown dependency 或未编译 provider 可能静默跳过；需 immutable composition plan、dependency closure、trust/capability、activation transaction、revoke/quiesce 和 failure snapshot。
4. **Command/operation/jobs**：registry serializer 会丢 executor/factory，operation、compile、navigation bake、export job 仍有同步 polling/私有 reader/晚发布；需唯一 EditorJobAuthority、typed DAG、resource-vector admission、cancel acknowledgement、terminal result store 和 WorkerSupervisor。
5. **Animation/script/gameplay**：已有 document/compile 局部基础，但 graph/timeline/string payload、Script source/LSP/debugger、GAS/AI/Network/Media 等产品链大多缺少 stable identity、compiled artifact、PreviewWorld 和 runtime receipt。
6. **UI host**：RetainedHost/HostContractState 仍承担过多跨域可变状态，callback 单槽/可重入、projection 全量复制、native window 与 runtime viewport identity 分裂；需每窗口 host session、typed callback reply、frame token、input capture、IME/A11y 和 shutdown barrier。

## 参考引擎约束

- Unreal Editor 的关键参考是 UObject/asset/package/transaction/PIE/World 分层及工具只通过对象/事务系统改变内容；不复制其历史兼容负担。
- Fyrox 提供 Rust 资源、scene graph、editor/plugin 组织参考；必须补足 Zircon 的跨 DLL、large project、failure receipt。
- Bevy 的 ECS/App/Plugin composition 可作为数据驱动系统边界，但其 editor/动态卸载能力不应被臆测扩展。
- Godot 的 Editor/Scene/Resource 与 lifecycle 分层说明 tool script 不能替代 durable resource transaction。
- `dev/Graphics` 只作为 Unity Graphics 的 shader/material/asset pipeline 和 profiling 参考，不用来证明 Editor 产品能力。

## 实施依赖与验证矩阵

`ED-M0 current-source freeze -> ED-M1 project/session/persistence -> ED-M2 document/transaction -> ED-M3 Runtime gateway/frame/input -> ED-M4 scene/asset/plugin providers -> ED-M5 preview/PIE/compiler -> ED-M6 domain products -> ED-M7 scale/fault/recovery`。每层需单元/集成、multi-window、10k entities/assets、operation cancellation、crash/reopen、provider failure、runtime disconnect 和产品端到端证据；性能需测量并记录 P50/P95/P99，不得直接宣称超过 Unreal。

当前 MVP F0-F5 仍优先于高级 Editor domain。所有当前工作树 dirty/untracked 相关结论实施前需 `source_recheck_required`。

## 状态与产出记录

- 2026-09-02：完成 review-only 综合归纳；未修改 Editor/App production code，未运行 Cargo 或 GUI/PIE 动态验证。
- 状态：`in_progress`；Tooling 明确排除。
