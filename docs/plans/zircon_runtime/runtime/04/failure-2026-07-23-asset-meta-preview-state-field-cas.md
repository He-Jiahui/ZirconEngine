---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: asset-meta-preview-state-field-cas
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/foundation/persistence/atomic_file.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/request_preview_refresh.rs
tests:
  - cargo test -p zircon_runtime asset_meta_preview_state --locked
  - cargo test -p zircon_editor preview_refresh --locked
---

# Runtime04：AssetMeta preview_state 缺少字段级 CAS 更新合同

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 asset catalog immutable generation / preview worker failure repair
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：`.zmeta` v7 及其原子写入由 Runtime04 持有；Editor09 只能消费该 authority，不能建立第二份 sidecar 或私有文件锁。

## 失败现象与复现证据

Editor09 preview worker 只需持久化 `PreviewState`，但 `AssetMetaDocument` 当前仅提供整文档
`load`/`save`。若 importer、watcher 或 migration 在 preview job 启动后更新同一 `.zmeta` 的
`source_digest`、`import_settings`、tags、entries 或 schema 字段，preview 若保存启动时的 clone
会整文件覆盖新字段；即使保存前重新 load 并合并 `preview_state`，外部写入仍可发生在 load 与
atomic replace 之间，缺少可证明的 compare-and-swap 边界。

2026-07-23 Editor09 独立复审将该 lost-update 窗口判为 Important。Editor09 已把 preview
decode/encode 移到 bounded worker，并在提交 generation 前校验 catalog revision、本资产 row identity、
source hash 与 meta path；剩余最低根因是 Runtime04 没有字段级 sidecar mutation authority。

## 最低共享层根因

`AssetMetaDocument::save` 是整文档无条件替换，Runtime asset owner 没有以
`{path, uuid, url, source_digest / document generation}` 为前置条件的字段级更新 API，也没有让
importer、migration 与 editor preview 共享同一路径写入序列。调用方无法在不复制 Runtime truth、
不覆盖其它字段的前提下只提交 `preview_state`。

## 架构修复验收

- Runtime04 提供 manager/meta authority 自有的 `preview_state` 字段级 CAS；更新必须重新读取当前
  `.zmeta`，校验 UUID、URL、source digest（或更强 document generation），仅修改 preview 字段。
- importer/watcher/migration 与 preview 字段更新走同一受管同路径写入序列；CAS 失败返回 typed stale
  结果，禁止覆盖当前文档。
- 并发测试用 barrier 在 preview read 与 commit 之间更新 settings/tags/entries/digest，最终文档保留
  外部字段；digest 变化时 preview update 必须 stale，digest 未变的独立字段更新也不得丢失。
- Editor09 worker 消费该 API 后，sidecar I/O 不持 editor live state lock 或全局 publish gate；最终只在
  短 gate 内验证并发布 generation row。

## 禁止临时方案

- 不得在 Editor09 新建 `.editor.meta`、第二份 preview sidecar、进程外 shadow truth 或调用点私有锁。
- 不得继续保存 preview job 启动时克隆的整份 `AssetMetaDocument`。
- 不得以“保存前再 load 一次”冒充原子 CAS，也不得弱化并发测试来隐藏 lost update。

## 修复结果与回传

Open state: `待 Runtime04 实现同路径字段级 CAS，并由 Editor09 复跑 preview worker 并发提交门；当前不声明 sidecar persistence 已通过。`
