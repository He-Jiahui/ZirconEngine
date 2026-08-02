---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: document-toolkit-save-hook-contract-missing
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_workflow_node: M3
fixing_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/extension/toolkit/mod.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/core/asset/dirty/registry.rs
tests:
  - tools/tests/test_editor06_document_toolkit_contract.py
  - zircon_editor DocumentToolkit open/dirty/save/close lifecycle matrix
  - zircon_editor Editor09 save_all partial-failure and close-prompt matrix
---

# Editor06 DocumentToolkit 保存钩子合同缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行者：`editor09-document-toolkit-save-hook-handoff-20260722`
- 来源执行切片：Editor09 M3.1 save/save_all/关闭询问编排
- 修复责任计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 交接原因：具体文档的序列化、写入目标、默认布局与关闭生命周期属于 Editor06
  `DocumentToolkit`；Editor09 只拥有跨文档 dirty/save orchestration，不能定义平行 toolkit 接口。

## 失败现象与复现证据

Editor09 计划明确要求 save/save_all 统一调度 Editor06 `DocumentToolkit::save`，但原始审计时源码没有
`DocumentToolkit`、`SaveCtx` 或 toolkit lifecycle owner。UI asset 与 animation editor 仍分别在
`ui/host/*_editor_sessions/save.rs` 直接写盘，并分别维护 disk baseline 或私有 dirty bool。对
`zircon_editor/src/**/*.rs` 的 `DocumentToolkit`/`document_toolkit` 搜索为零结果；因此 Editor09 即使已取得
Editor03 原子 save token，也没有合法的文档写入钩子可调度，无法实现统一 save_all 或关闭询问。

## 最低共享层根因

原始审计时 Editor06 M3 仍停留在计划态：`core/extension/toolkit/` 未物化，workbench 未把打开的 asset editor
实例注册为带 `DocumentId`、`HistoryContextId`、布局、菜单和保存钩子的统一 toolkit。现有 host save
入口是按编辑器类型分叉的旧架构，既不能被 Editor09 枚举，也无法在写盘前后统一执行 Editor03
`capture_save_token`/`mark_saved_if_unchanged` 协议。

## 架构修复验收

- Editor06 实现 folder-backed `core/extension/toolkit/`，提供 typed `DocumentToolkit`、`ToolkitLayout`、
  `SaveCtx`/`SaveError` 与实例生命周期；workbench 能按稳定 `DocumentId` 枚举打开的 toolkit。
- toolkit 提供 `HistoryContextId` 与具体序列化/写入 hook，但不得缓存另一份 transaction dirty、
  `saved_top` 或 generation，也不得在 hook 内无条件清 dirty。
- UI asset 先行迁移，animation/material 等按 Editor06/07 责任迁移；旧 host 直接写盘入口在所有调用方
  切换后物理删除，不保留 alias、fallback 或类型特判。
- Editor09 在上层以 DirtyRegistry 枚举目标：写盘前捕获 Editor03 token，引用检查通过后调用 toolkit
  hook，成功后执行 compare-and-mark；单文档失败不清 dirty，save_all 返回稳定逐文档 typed outcome。
- 关闭询问统一消费同一 dirty snapshot，明确区分 save/discard/cancel；cancel 不关闭，partial save failure
  仅关闭已成功或明确 discard 的文档。
- focused lower-layer lifecycle、原 UI asset/animation save 往返、Editor09 save_all partial failure、保存中
  新事务与 close-prompt 矩阵全部通过，两路独立 review 为 `0/0/0`，形成受管 SHA 后回传 Editor09。

## 禁止临时方案

- 禁止在 Editor09 新建第二个 toolkit trait、按 editor type match 调 host save 方法，或保留旧方法作 fallback。
- 禁止以 UI 单线程、保存期间全局禁事务、写盘后无条件 mark clean 或缓存 dirty bool 规避 token 合同。
- 禁止把 save_all 简化为 best-effort `for` 循环并丢弃逐文档错误、取消或 stale-save 结果。
- 禁止用 test-only toolkit、弱化关闭询问测试或跳过 UI asset/animation 迁移宣称完成。

## 修复结果与回传

Open state: 待修复; no pass is claimed. Forward fix in progress:
`editor06-document-toolkit-hardcut-r1-20260801` 已接管
DocumentToolkit 的注册表、生命周期和 focused/upward 验证。Editor09 可继续与 toolkit 无关的 M3 切片，
但 save/save_all/关闭询问不得在本记录形成受管 fixed return 前伪完成。

## 产出记录与时间

- 2026-07-22：状态 `open_handoff_recorded`。已证明 Editor09 M3 保存编排的最低依赖 `DocumentToolkit`
  在源码中不存在，现有 UI asset/animation 仍各自直接写盘；canonical failure 已路由 Editor06，未增加
  fallback、兼容 API 或平行 dirty owner。
- 2026-07-22：Coordinator failure import 已登记 node `704994`；按 fixing plan 定向 `failure open` 返回本
  artifact 与 lifecycle key，状态为 `open`。handoff 模板验证未报告本文件错误；全库 audit 的既有跨 owner
  schema/cycle 诊断不计入本记录完成项。
- 2026-08-02：状态 `fixing`。当前会话已取得 DocumentToolkit 模块、生命周期测试和本 failure record 的
  精确租约；实读确认 folder-backed toolkit registry、save/close lease、不可变 snapshot 与 asset/animation
  public save dispatch 已取代本记录中的旧架构描述。静态合同 4/4 通过，下一步为受管 Rust focused/broad
  验证与独立复审；本条不是 fixed return。
- 2026-08-02：状态仍为 `fixing`。独立静态复审为 `0/0/0`：双索引 registry、save/close lease、私有
  immutable snapshot 与 host 的 token→hook→compare-and-mark 顺序均符合合同；focused gate
  `144a349f7bc84edfbc1f8e6377dbd2b7` 已提交。仍须等待该 gate 与当前树 broad gate 的 terminal evidence，
  然后才能 fixed return。
- 2026-08-02：状态仍为 `fixing`。发现公开 `apply_layout_command(CloseView)` 可绕过 document close
  lease，导致 layout/session 已移除而 toolkit snapshot 仍保留。该入口现统一委派 `close_view`，后者仅在
  内部 layout apply 成功后提交 close lease；新增 manager 公共入口回归，断言 UI asset 文档的 toolkit
  descriptor 随 CloseView 移除。格式与静态合同通过；该 runtime 回归、独立复审和受管 Cargo terminal
  evidence 尚待完成，禁止 fixed return。
- 2026-08-02：状态仍为 `fixing`。独立审查发现首版收束会把任意 `ViewInstanceId` 强制解析为
  `ToolkitInstanceId`，使空值或超长的非 document close 在 layout apply 前失败。`begin_document_close`
  现仅对可形成 toolkit ID 的实例查询 registry；未注册实例保留原有 no-op close 语义。manager 回归同时
  覆盖空 ID 与 257-byte ID。局部格式、静态合同与 diff 检查通过；修正后二审和受管 Cargo terminal
  evidence 仍待完成，禁止 fixed return。
- 2026-08-02：状态仍为 `fixing`。修正后二审为 `Critical/Important/Minor = 0/0/0`；调用点审计确认
  公开 Manager、事件执行与 retained-host 均经统一 `CloseView` 入口，且 manager 回归验证 toolkit descriptor
  随成功 close 注销。`rustfmt --check`、Editor06 静态合同 `16/16` 与 diff 检查通过；受管 Cargo terminal
  evidence 未返回，故仍禁止 fixed return。
