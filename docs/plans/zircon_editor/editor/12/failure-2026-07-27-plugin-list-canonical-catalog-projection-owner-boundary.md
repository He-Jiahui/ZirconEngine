---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: plugin-list-canonical-catalog-projection-owner-boundary
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/plugin/catalog_snapshot.rs
  - zircon_editor/src/core/plugin/projection.rs
  - zircon_editor/src/core/commandlet/runner.rs
  - zircon_editor/src/core/commandlet/tests.rs
tests:
  - Editor12 generation-owned plugin catalog projection identity/build-count regression
  - Editor08 plugin-list repeated commandlet projection regression
  - cargo test -p zircon_editor --lib --locked commandlet
---

# Editor12: plugin-list requires the canonical generation-owned catalog projection

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Editor08 Plugin-list canonical descriptor/commandlet repair
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：Plan08 owns command descriptor routing and commandlet authorization, but Editor12
  owns the immutable editor-plugin catalog generation and its public read-only projection.

## 失败现象与复现证据

Static trace of the current Plugin-list consumer is:

```text
run_plugin_list_commandlet
  -> EditorPluginDescriptor::builtin_catalog_projection
  -> EditorPluginDescriptor::builtin_catalog
  -> builtin_editor_plugin_descriptors
  -> GENERATED_EDITOR_PLUGIN_CATALOG.iter().map(...).collect::<Vec<_>>()
```

The `builtin_catalog_projection` helper creates a new owned descriptor list and a new owned
projection for every commandlet invocation. The current repeated Plugin-list regression executes
the command twice and checks JSON equality, but it does not prove generation identity or a zero
rebuild on the second call. This duplicates the catalog projection owned by the open Editor12
failure `failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md` and violates the
Editor08 handoff requirement to consume the existing Plugin12 catalog rather than rebuilding it.

## 最低共享层根因

Editor12 has not yet published a generation-owned immutable editor-plugin catalog projection at
the core boundary for headless consumers. Plan08 therefore added a descriptor-local projection,
which is a second source of catalog materialization and cannot be the architectural repair.

## 架构修复验收

- Editor12 publishes one ordered immutable projection for an unchanged plugin generation, with a
  consumer-safe borrowed or `Arc` access path and an explicit invalidation rule for generation
  changes.
- A focused Editor12 regression proves repeated same-generation projection access performs no
  descriptor/catalog rebuild and changes identity only after a real generation change.
- Plan08's `plugin-list` commandlet consumes that canonical projection without rebuilding the
  generated descriptor catalog or introducing a commandlet-local cache.
- The existing Plugin-list descriptor discovery, missing-capability exit code `3`, unknown-command
  exit code `2`, and stable JSON commandlet regressions pass through the returned API.

## 禁止临时方案

- Do not retain `EditorPluginDescriptor::builtin_catalog_projection` as a descriptor-local
  alternate catalog API.
- Do not rebuild a `Vec` from `GENERATED_EDITOR_PLUGIN_CATALOG` per commandlet invocation.
- Do not add a commandlet-local static cache, fallback generated-list path, compatibility alias, or
  test-only projection bypass.

## 修复结果与回传

`EditorPluginManager::builtin_shared()` 已成为 builtin generation 的唯一 shared owner；其
`EditorPluginCatalogStore` 发布 immutable `EditorPluginCatalogSnapshot`，snapshot 在构建时由
registration reports 生成并持有 `Arc<EditorPluginCatalogProjection>`。Plugin-list 直接消费该
snapshot projection，未保留 commandlet-local cache、descriptor-local projection 或 generated-list
fallback；旧 `EditorPluginDescriptor::builtin_catalog_projection` API 已删除。

该修复仍处于 `resolving_failure`。静态契约、格式和 diff 检查已通过，但受管 Editor12 Cargo
验证 `f8726b18912e49d7a74dcb10051f3006` 在 coordinator materialization prepare 阶段因 source
baseline 将外部已跟踪 Runtime15 路径误判为 unowned overlay 而失败，未创建 Cargo run，故不能作为
Rust 编译、独立复审、fixed return 或 commit 的证据。该协调器问题已交接至
`failure-2026-07-27-validation-copy-cargo-materialization-nonterminal.md`；Plan08 仍不得消费本
failure 的 fixed return。

## 产出记录与时间

### 2026-07-27 Editor08 Plugin-list -> Editor12 catalog ownership handoff

- 状态：`open`。
- 完成项目与验证证据：已静态追踪 `run_plugin_list_commandlet` 至 `GENERATED_EDITOR_PLUGIN_CATALOG.iter().map(...).collect::<Vec<_>>()`，确认每次调用重建 owned projection；现有双调用测试只证明 JSON 等价，不能证明 generation identity/build-count。Editor12 为最低共享 catalog owner，Plan08 不保留 descriptor-local projection 作为修复。

### 2026-07-27 Editor12 generated-builtin canonical projection

- 状态：`resolving_failure`。
- 完成项目与验证证据：`EditorPluginManager::builtin_shared()` 单一持有 builtin catalog；`EditorPluginCatalogSnapshot` 以 `Arc` 持有从 registration report（含 editor capabilities）构建的 ordered projection。Plugin-list 两次读取共享同一 projection identity，且没有 commandlet cache；descriptor-local `builtin_catalog_projection`、`OnceLock` projection 和 generated-list fallback 均已硬切除。新增 catalog store/projection capability regression 与 manager state-generation regression；`python tools/tests/test_editor12_plugin_catalog_store_contract.py`（8/8）、`python tools/tests/test_editor12_plugin_manager_contract.py`（2/2）、`rustfmt --check`、`git diff --check` 通过。受管验证 job `f8726b18912e49d7a74dcb10051f3006` 在 materialization prepare 失败，未运行 Cargo；待 coordinator baseline attribution 修复后重新创建 immutable snapshot gate，再进行独立复审、lifecycle return 与 managed commit。

### 2026-07-28 Editor12 canonical catalog/projection recheck

- 状态：`resolving_failure`。
- 完成项目与验证证据：当前 immutable catalog store、lifecycle replacement 与 commandlet consumer 的精确静态门禁重新通过：catalog store 10/10、catalog projection 3/3、plugin manager 20/20；`rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。当前共享 Coordinator01 materialization `416b041cd7524ae6a983f8801bf9bcfc` 仍无 source hash、无 Cargo run；本切片不创建重复 copy，继续等待既有 `validation-copy-cargo-materialization-nonterminal` 修复。故不将静态结果计为 Rust 运行时、独立复审、fixed return 或 commit。
