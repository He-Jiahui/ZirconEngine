---
handoff_kind: failure
status: open
created_at: 2026-10-01
summary_slug: penpot-editor-execution-packet-owned-bindings
origin_plan: docs/plans/designment/02-milestone-execution-and-evidence.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/designment/02
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/execution_packet.rs
tests:
  - tools/build/build-editor.ps1 -OutputDirectory D:/cargo-targets/penpot-editor-20260930-01a0f04d-6fc0351c
---

# Penpot 主工作台引擎验证：penpot-editor-execution-packet-owned-bindings

## 来源执行者

- 来源计划：`docs/plans/designment/02-milestone-execution-and-evidence.md`
- 来源执行切片：当前 App Editor / Runtime DLL 产品构建与真实双端工作台截图。
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 协调 session：`astra-rg166-device-lowering-20260930-01a0f033`。
- 交接原因：错误在最低共享运行时 owner，来源 UI 计划不覆盖其候选修改。

## 失败现象与复现证据

- 托管 Windows 产品入口 `tools/build/build-editor.ps1` 通过 `validate-matrix.ps1` 启动 job `ff63696a9e12459cb4c2fe0171bb6c0c`；源码 input manifest `b60bab36ef66a0e1b80228fe5ae8194c9828f8ed32fe657f13455f5ca2a939fe`。job 已 release，产品未发布，Cargo exit 101 / wrapper exit 1。
- 完整输出：`D:\cargo-targets\mvp-test-fixtures-45460\penpot-workbench-6fc0351c5c1e43178d5e205f123e139f\tmp\editor-build-r2\outer-stderr.log`；SHA-256 `36889d0508cf2d19c79e55e59623c1dd2f712c60644854359e7453e9145ab6f0`。该完整外层日志属于此 job；pool 内 earlier-check diagnostics 不属于该失败构建。
- 18 条总错误中，本 fixing scope 占 1 条。2026-10-01 只读核对确认 affected primary source 的当前 checkout、sealed source 和 manifest hashes 一致，失败仍适用；related tests 没有被本轮执行。
- UI timer scope 在交接时仍由原 session live lease 保护；plugin 和 graph 原 sessions 分别 registered / resolving_failure，但缺 live lease，修复者应先重新取得精确 scope 并保留来源 attribution。来源 session 未接管这些源文件。

- `zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/execution_packet.rs`: `3279a421c4fdf23ccb52eb65ae798d6cd0dbc9455232c6144828f4f9ba9f27a2`

## 最低共享层根因

Owned per-pass device binding rows 声明为 Box<[CompiledRenderGraphAccessAllocationBinding]>，收集阶段却把 allocation table 返回的引用压入 vector，形成 Box<[&Binding]>；E0277 在 execution_packet.rs:823。

## 架构修复验收

- Binding 是 Copy。per-pass plan 存储 *binding 拥有值；transition validation 的 access_owner lookup 仍保留同一原表引用。不得 clone pipeline、移除 binding validation 或新增特例成功路径。
- 执行 execution_packet_carries_exact_device_ranges_and_queue_transitions 与 execution_packet_carries_exact_access_ids_in_compiled_order 聚焦回归。
- 修复 owner 验证最低层后，来源计划重跑同一托管产品构建；通过仍须取得正常 Editor 与 native capture 的实际 PNG / source、token、font、media、locale receipts。编译过关不等于画面一致。

## 禁止临时方案

- 不添加兼容 shim、复制 authority、候选自证、unsafe 借用绕过或单调用点成功旁路。
- 不删除或弱化测试、trust / allocation / timer 验证及产品验收门。
- 不把 pending receipt、静态 mock 或旧产品截图标为 passed / accepted。

## 修复结果与回传

待修复，状态保持 `open`。来源 Penpot 导入诊断继续推进；实际引擎产品门未完成。
