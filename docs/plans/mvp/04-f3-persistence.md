---
related_code:
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/core/editing
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/asset/project
  - zircon_runtime_interface/src/serialization
related_tests:
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/tests/editing/inspector.rs
  - zircon_editor/src/tests/editing/transaction_engine
  - zircon_runtime/src/scene/tests
  - zircon_runtime_interface/src/serialization
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/mvp/03-f2-scene-runtime.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
status: blocked_by_f2
gate: F3
last_refined: 2026-07-24
---

# F3 持久化 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `subagent-driven-development`（推荐）或 `executing-plans`。涉及 schema、migration、atomic write 或 save token 时使用相应 Runtime 04/Editor 11 owner，并在测试阶段运行 persistence boundary coverage。

**Goal:** 对 F2 的同一个项目修改一个已命名实体的 transform，保存 project/scene，销毁所有 owner 后从磁盘重新打开，并精确保留实体 identity、transform 和 mesh/material asset refs。

**Architecture:** `EditorProjectDocument` 负责 editor project save/load 编排，scene serializer 负责 canonical persisted form，ProjectManager/registry 负责重开后的 reference resolution。F3 通过真实磁盘和新 owner generation 比较，不允许复用内存 world 或 active manager。

**Tech Stack:** Editor transaction/save token、SceneAsset/DynamicScene serialization、ProjectManager、canonical TOML/JSON envelope、atomic file replace。

---

## 1. 入口条件

- [ ] F2 canonical 项目已通过 runtime product gate，包含稳定 primitive entity 和可解析 mesh/material refs。
- [ ] F3 Session 已领取 project document、scene serialization、save/dirty contract 和测试文件 lease。
- [ ] 原始 project root、manifest hash、scene hash、primitive identity 和 registry summary 已记录。
- [ ] Editor 11/Runtime 04 中直接影响当前 scene/project writer 或 ref migration 的 failure 已分类。

## 2. 固定 F3 修改

F3 使用唯一、可精确比较的 authoring delta：

- 目标：F2 cube primitive 的 persisted entity identity。
- 修改：translation X 增加一个非零固定值，同时保持 rotation、scale、parent、mesh ref、material ref 不变。
- 保存：使用正常 editor save orchestration 和 transaction save token；不直接写场景文件。
- 重开：Drop editor document、project manager、runtime session 和 registry owner，再从 project root 创建全新 generation。
- 比较：entity identity/name、完整 transform、parent、mesh ref、material ref、default scene URI 和 registry resolution。

测试可使用常量表达固定 delta，但产品验收必须从命令/transaction 结果保存；F4 再把同一 delta 接到 UI command path。

## 3. 非目标

- 不验收 UI selection/Inspector 输入；它属于 F4。
- 不扩展所有场景组件 schema，也不引入 binary scene format。
- 不以 workspace layout roundtrip 代替 scene authoring state。
- 不接受只比较 node count、文件存在或序列化文本不为空。

## 4. M4.1 精确 project document roundtrip

### 目标

现有 `editor_project_document_roundtrips_world_and_workspace` 从“节点数量相同”提升为精确 F3 状态合同。

### 实现切片

- [ ] 在 `document_roundtrip.rs` 使用 product template 创建项目并通过 ProjectManager 完成 scan/import。
- [ ] 从 default scene 找到 persisted cube identity，使用 Editor transaction engine 应用固定 transform delta。
- [ ] 捕获 save token，调用 `EditorProjectDocument::save_to_project`，成功后才把对应 history baseline 标记 clean。
- [ ] Drop 当前 document/world/manager，重新 `ProjectManager::open`、scan/restore registry 并 `load_from_project`。
- [ ] 精确断言 entity identity/name、完整 transform、parent、mesh/material refs、default scene 和 workspace；asset refs 必须重新解析到重开 generation。
- [ ] 保存前后比较不应变化的 camera/light/cube ref，防止 writer 丢字段但 node count 仍相同。
- [ ] 增加失败断言：save 失败或 token 失效时 dirty baseline 不移动，磁盘保持上次有效 document。

### 测试阶段：F3 Document Roundtrip Gate

- [ ] 运行 editor project document、editing history/save token 和 inspector transaction focused suites。
- [ ] 运行 runtime scene/project serialization roundtrip 和 reference resolution focused suites。
- [ ] 对保存后重开结果执行结构比较，而非 raw text substring。
- [ ] 对 invalid save token、write failure 和 unresolved ref 运行负例并确认 typed error。

### 退出证据

- [ ] 新 manager/generation 精确保留 F3 修改和未修改字段。
- [ ] dirty/save baseline 与磁盘 commit 成败一致。
- [ ] asset refs 在重开 registry 中重新解析，不引用旧内存 handle。

## 5. M4.2 Canonical writer 与 migration 边界

### 目标

同一 current-version project 重复保存稳定，不触发隐式 migration、字段漂移或无意义 diff；future/corrupt input fail closed。

### 实现切片

- [ ] 确认 scene/project writer 只输出当前 schema/version envelope，不双写 retired version 字段。
- [ ] current document 保存两次后 canonical scene bytes 相同，第二次没有 dirty transition。
- [ ] v0/v1 migration 只在读取历史 fixture 时执行；当前 F3 project 的 `migrated_from` 必须为空。
- [ ] future schema/version、invalid transform shape、invalid AssetRef 和 interrupted atomic write 保留上次有效文件并返回 typed error。
- [ ] 不为 F3 引入 serializer payload clone 或第二套 Value DOM；若现有 Editor 11 performance failure直接阻断大文件，本门只修相同 owner 根因。
- [ ] 文档 owner 若描述旧 schema/双写行为，按 `code-module-docs-maintenance` 做最小同步；无公共合同变化不新增说明文档。

### 测试阶段：F3 Serialization Boundary Gate

- [ ] 运行 runtime_interface serialization envelope/migration focused suites。
- [ ] 运行 current scene save→reload→save byte-stability 和 project reference roundtrip。
- [ ] 运行 future version、invalid ref、write interruption/atomic replace 负例。
- [ ] 再运行 M4.1 完整 project document roundtrip，证明边界修复未破坏 editor orchestration。

### 退出证据

- [ ] current F3 project 重复保存幂等。
- [ ] 无隐式 migration、retired field 或 ref identity 漂移。
- [ ] 失败保存不破坏最后一个有效 persisted document。

## 6. M4.3 产品保存与进程级重开

### 目标

使用 staged editor host 完成 project save，进程退出后由第二个 editor/runtime 进程从磁盘观察 F3 delta。

### 实现切片

- [ ] 为产品 integration harness 暴露正常 SaveProject operation 的完成/失败结果和 persisted generation；不得增加直接 filesystem save shortcut。
- [ ] 把 F3 fixed delta 通过 editor transaction owner 应用，随后从相同 host 执行 SaveProject。
- [ ] 退出第一个 editor process，确认 project directory 可重命名后恢复原名。
- [ ] 第二个 process 打开项目，从 editor snapshot/runtime inspection 读取 cube identity/transform/refs。
- [ ] product diagnostics 记录 pre-save dirty generation、save token、persisted generation 和 reopened generation，不记录完整文档内容。

### 测试阶段：F3 Product Persistence Gate

- [ ] 在 F1-F2 canonical 项目副本上启动 staged editor，应用 transaction delta 并保存。
- [ ] 等待明确 save completion，再干净退出；仅收到 UI click 不算保存完成。
- [ ] 用新进程重开同一项目并输出结构化 state summary。
- [ ] 比较 state summary 与磁盘 parsed document，断言 transform/refs 完全一致。
- [ ] 再启动 staged runtime，确认保存后的场景仍可渲染可见 primitive。

### 退出证据

- [ ] 保存前 dirty、保存成功后 clean、重开后 clean 的状态转换完整。
- [ ] 第二进程和 runtime 都消费已修改的 persisted scene。
- [ ] 项目未依赖第一个进程的内存 world/registry。

## 7. F3 阶段退出清单

- [ ] M4.1、M4.2、M4.3 全部通过。
- [ ] 比较包含 entity identity、完整 transform、parent、mesh ref、material ref 和 default scene URI。
- [ ] save token/dirty baseline 与 atomic disk commit 一致。
- [ ] 两次 current save 幂等，失败 save 保留上次有效文件。
- [ ] editor 进程退出后新进程和 runtime 均观察相同结果。
- [ ] F4 复用同一 delta 和项目，不创建新的简化 persistence fixture。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## Current-Source Review (2026-08-02)

- `document_roundtrip.rs` now creates the RenderableEmpty product template through
  `ProjectAuthority`, applies the fixed Cube transform through `EditorTransactionEngine`, captures
  a save token, drops the initial document/level/project generation, and compares the reopened
  Cube identity, full transform, parent, mesh/material references, default scene, and workspace
  against the persisted project.
- `EditorProjectDocument::save_to_project` keeps the scene as the authoring authority: it captures
  the prior workspace, atomically persists the requested workspace before the scene, and restores
  the exact workspace bytes if the scene save fails. Current focused source tests cover byte-stable
  repeat saves, failed scene/workspace writes, dirty-baseline behavior, and persisted settings
  provenance.
- The production `SaveProject` event records dirty/save-token/persisted-generation diagnostics and
  does not mark history clean until persistence completes. These are current-source contracts only;
  the declared staged editor/runtime process evidence remains required before F3 can be accepted.
