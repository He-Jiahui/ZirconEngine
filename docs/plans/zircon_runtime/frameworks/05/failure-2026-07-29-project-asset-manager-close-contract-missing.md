---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: project-asset-manager-close-contract-missing
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
tests:
  - cargo test -p zircon_runtime --lib --locked project_asset_manager
  - cargo test -p zircon_editor --lib --locked document
---

# Frameworks05: Project asset manager lacks a close-generation contract

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：document lifecycle typed-message producer
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：当前 `AssetManager` 只暴露 project 打开和快照查询；退役 runtime project generation、watcher 与资源同步是 runtime manager contract，Editor01 不得直接修改 `ProjectAssetManager` 内部状态或伪造 close 事实。

## 失败现象与复现证据

`AssetManager` trait 提供 `open_project`、`open_prepared_project` 与 `current_project_snapshot`，而 `ProjectAssetManager::open_prepared_project` 仅在切换打开时替换 project、watcher 与资源。不存在可由 Editor01 调用的 close/deactivate API，因此没有“当前 project 已原子退役”的提交点可合法发布 `DocumentMessage::Closed`。

## 最低共享层根因

runtime asset manager contract 缺少 generation-aware `close_project` 操作及其 resource/watcher retire 语义。关闭不能由 Editor UI 清理 tab、清空本地状态或读取内部 lock 来代替。

## 架构修复验收

- `AssetManager` 提供唯一的 close/deactivate contract；没有活动 project 时为明确 no-op，不产生虚假 asset change。
- 实现必须在同一 manager generation 边界内退役 project snapshot、watchers 和 project-owned resources；失败不得留下半关闭 generation。
- 关闭成功后 Editor01 从该 contract 返回的 committed root 发布一次 `DocumentMessage::Closed`；重复关闭、失败关闭和 UI focus/dirty 变化不得发布结构性事件。
- 回跑 Frameworks05 focused manager 测试及 Editor01 document lifecycle/Editor12 bridge 上行回归。

## 禁止临时方案

- 不得由 `zircon_editor` 直接写 `ProjectAssetManager` 内部 mutex、stop watcher 或清除 runtime resource。
- 不得以切换 welcome page、关闭 tab 或 manager drop 伪装为 project close。
- 不得保留旧 project snapshot、watcher 或资源作为兼容 fallback。

## 修复结果与回传

Current source contains the manager-owned `close_project` contract, the concrete retirement owner,
and focused no-active-project/resource/source-index removal coverage. The contract does not restore
an Editor-local close path or a legacy manager facade.

Independent architecture review found one remaining generation-order defect: the concrete close
owner drops the project-generation write guard before broadcasting its `Removed` changes. A
concurrent open may therefore commit and publish the next generation's `Added` changes first. The
successor must keep close publication inside the same generation fence, release retired watcher
join handles outside that fence, and add a regression that fixes the ordering contract.

Open state: `implementation_recovered_generation_publication_repair_pending`。未执行 fresh managed
Runtime/Editor upward gates，未生成 fixed return，也未声明 Editor01 close lifecycle 已验收。
