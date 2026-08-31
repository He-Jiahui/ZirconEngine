---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: editor-asset-catalog-project-close-deactivation-missing
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/api.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/editor_asset_state.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/host.rs
  - tools/tests/test_editor09_project_close_deactivation_contract.py
tests:
  - python tools/tests/test_editor09_project_close_deactivation_contract.py
  - cargo test -p zircon_editor --lib --locked editor_asset_manager
  - cargo test -p zircon_editor --lib --locked document
---

# Editor09: Project close leaves the editor asset catalog active

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：document lifecycle typed-message producer
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：runtime `AssetManager::close_project` 可以退役 project generation，但 editor asset catalog、preview state 与 source-sync generation 属于 Editor09 的单一投影 owner；Document lifecycle 不得直接清空其内部状态。

## 失败现象与复现证据

`DefaultEditorAssetManager::refresh_from_runtime_project` 在 runtime `ProjectAssetManager` 没有活动 project 时直接返回 `Ok(())`，没有撤销 `EditorAssetState` 内的 project、catalog、locator/UUID 索引或 preview 状态。因此任何 Editor01 close producer 即使正确调用 runtime close，仍会让资产浏览器继续读取上一工程的 catalog。

`EditorUiHost::restart_ui_asset_workspace_watcher` 已能在无项目时释放 watcher 并转换 refresh pipeline，但目前没有 project-close 编排调用它；只关闭 UI tab、切换 welcome 页面或忽略 refresh 都不能构成关闭事实。

## 最低共享层根因

Editor09 缺少 generation-aware 的 editor asset projection deactivation：runtime project 从 `Some` 转为 `None` 时，catalog/preview/source-sync state 必须以新的空 generation 原子替换，并以 canonical editor asset change stream 通知消费者。

## 架构修复验收

- `EditorAssetManager` 提供唯一的 runtime-project deactivation/refresh 语义；无活动 project 不得保留旧 catalog、project root、locator/UUID 索引、preview cache 或在途 source-sync generation。
- 关闭必须发布一个新的空 catalog generation，旧 preview/sync completion 不能在切换后回写；不可用 `Ok(())` 保留旧投影，也不可提供旧 catalog fallback。
- Editor01 仅在 runtime `close_project` 返回 committed root 后编排 Editor09 deactivation、UI asset watcher transition，并发布一次 `DocumentMessage::Closed`。
- 回跑 Editor09 asset manager focused tests，再上行回跑 Editor01 document lifecycle 与 project-close producer tests。

## 禁止临时方案

- 不得由 DocumentLifecycleAuthority、asset browser view、welcome page 或 retained host 直接写 `EditorAssetState`。
- 不得把“没有 runtime project”解释为保留最近 catalog 的成功 refresh，或用空白 UI 遮挡旧数据。
- 不得保留旧 project catalog、preview 或 source-sync completion 作为 close 后 fallback。

## 产出记录与时间

| 日期 | 状态 | 产出与证据 |
| --- | --- | --- |
| 2026-08-27 | source hard-cut complete / managed validation pending | 当前源码重审发现原实现仍有两条关闭竞态：`refresh_from_runtime_project` 复制裸 `ProjectManager` 后可在 Runtime close 之后迟到提交，且空投影 deactivation 不推进 source-sync epoch；catalog/publish/source-sync identity 还使用 saturating 或无检查递增。现已硬切为 `current_project_generation_snapshot -> ProjectAssetGenerationToken -> commit_if_project_generation`，O(n) catalog generation 构建与 preview rebase 位于 Runtime fence 和 Editor source gate 外，fence 内只执行代际比较、状态交换和瞬态集合迁移；任何关闭/重开后的旧快照只能计入 `runtime_project_generation_superseded_count`，不能回写目录。deactivation 改为不可失败的本地 retirement，空投影也推进 checked source-sync epoch，committed close 不再吞掉“目录未清空”错误后发布成功；catalog revision、publish epoch 与 source-sync epoch 均使用 checked progression，耗尽时 fail-stop，禁止复用身份。进一步反例审查确认 Runtime token 依赖全局 catalog-input sequence 区分同根重开，遂将该 allocator 从裸 `fetch_add` 硬切为 checked CAS，使耗尽 sequence 不能回绕成旧 token。生产 owner 从 822 行拆为 `sync_from_project.rs` 460 行与 `project_sync/tests/mod.rs` 361 行，未保留旧测试镜像。对照 `docs/plans/optimize/zircon_editor/04-*`、`docs/plans/optimize/zircon_runtime/24-*` 以及 UE `IAssetRegistry`/`UAutoReimportManager` 后确认边界：吸收单一 registry owner、短终端发布和明确退役，不臆造 UE 同进程 project-close API。新增 4 条 Python 源码契约由 RED 2/2 收敛为 GREEN 4/4，并新增 catalog/source-sync/runtime-sequence 近耗尽 Rust 回归；rustfmt 与 scoped `git diff --check` 通过。未启动 Cargo、并发压力或动态 profile，本行不宣称性能基准、fixed 或 accepted；failure 保持 open。 |
| 2026-07-29 | open | 当前源码检查确认 runtime project 为 `None` 时 `DefaultEditorAssetManager::refresh_from_runtime_project` 直接返回成功而不清空投影；该最低层缺口已从 Editor01 document producer 移交 Editor09。 |
| 2026-08-23 | implemented / validation-pending | 当前源码已硬切原 `Ok(())` 保留路径：无活动 runtime project 的 refresh 统一调用 `deactivate_runtime_project`，committed close 先撤销 editor asset projection 再停 UI asset watcher。deactivation 原子替换为空 catalog generation、清空 project/catalog/preview/source-sync 状态并广播一次 `CatalogChanged`；新增 poisoned state-lock 与 residual source-generation 回归，生产锁取得改为 owner-local recovery helper，残留 source-sync epoch 也不能被误判为 no-op。受管 focused Cargo 尚未执行，failure 保持 open，未标 fixed/accepted。 |
| 2026-08-23 | implemented / validation-pending | state lock recovery 已收敛到 `editor_asset_state` 的唯一 read/write helper，`sync_from_project`、catalog snapshot 与 project deactivation 全部硬切使用它；新增 sync-from-project poisoned-lock 回归，避免工程打开/刷新在此前关闭错误后的 poisoned state 上再次崩溃。受管 focused Cargo 尚未执行，failure 保持 open，未标 fixed/accepted。 |
| 2026-08-23 | implemented / validation-pending | recovery helper 同时覆盖 asset-details 查询与后台 preview-refresh worker；manager 域对 `EditorAssetState` 的生产 direct `read/write().expect("editor asset state lock poisoned")` 搜索为 0，避免预览完成/释放路径在同一 poisoned state 上绕过工程关闭与同步语义。受管 focused Cargo 尚未执行，failure 保持 open，未标 fixed/accepted。 |

## 修复结果与回传

Open state: `source_hardcut_complete / static_contract_green / managed_validation_and_fixed_return_pending`。Editor01 仍需等待 Editor09 focused Cargo、并发关闭/迟到同步回归和上行 document lifecycle 验证；完成 source-bound 验证前，本 canonical artifact 不改名为 `fixed-*`。
