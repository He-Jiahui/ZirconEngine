---
related_code:
  - zircon_runtime/crates/zr_resource/src/io/transaction
  - zircon_runtime/src/asset/mutation/delete_preflight.rs
  - zircon_runtime/src/asset/registry/deletion.rs
  - zircon_runtime/src/asset/project/manager/deletion.rs
  - zircon_runtime/src/asset/project/reference_diagnostics.rs
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/deletion.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs
  - zircon_editor/src/core/asset/refactor/delete.rs
  - zircon_editor/src/core/asset/refactor/deletion.rs
  - zircon_editor/src/core/extension/toolkit/document_toolkit.rs
  - zircon_editor/src/core/extension/toolkit/registry.rs
  - zircon_editor/src/core/extension/toolkit/save/error.rs
  - zircon_editor/src/ui/host/editor_asset_manager/api.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/project_deletion.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_save_batch.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/retained_host/app/assets/deletion.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/asset_deletion_blocker.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/asset_deletion_blocker.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_asset_deletion_blocker.rs
tests:
  - zircon_runtime/src/asset/mutation/tests.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/deletion.rs
  - zircon_runtime/src/asset/project/reference_diagnostics.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/engine/tests.rs
  - zircon_editor/src/core/extension/toolkit/tests/saving.rs
  - zircon_editor/src/core/asset/refactor/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/asset_deletion_blocker_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
doc_type: output-record
status: implementation_complete_m3_2_module_gates_green_full_workspace_blocked_by_shared_runtime
---

# Editor10 安全删除事务与引用阻断模态

本记录覆盖 Plan10 M3 的基础设施闭环：在不引入 force-delete 兼容入口的前提下，建立从离线 registry 引用检查到 Runtime 持久删除事务、Editor 后台 job、资产工作区命令、referencer blocker 模态、悬挂引用诊断和保存准入的数据链。M3.2 产品实现已完成；因 Runtime/Editor 全工作区仍被共享基线编译错误阻断，本记录不关闭 M3 完整测试阶段，也不把强制删除当作安全删除的隐式分支。

## 架构结果

- Runtime preflight 以 `AssetUuid` 为唯一目标，不保留路径 fallback；根资产删除检查同一 source 的全部 root/subasset UUID，排除 source 内部 companion edge，只把外部 referencer 作为阻断项，并按 locator/UUID 稳定排序。
- Runtime commit 在 generation 写锁外准备候选 `ProjectManager`、registry/resource/catalog 删除态与文件事务；获得写锁后复核 catalog generation 和 preparation epoch，文件提交、候选安装、ResourceManager 发布及 `Removed` change 仍服从同一 generation 门。
- `zr_resource` durable transaction journal 升级为单次 write 携带多个 retirement；删除同时退休 source 与 `.zmeta`，对每项锁定 expected digest。部分 retirement 后崩溃恢复会从 durable evidence 恢复 registry、source、sidecar 并清理 journal，不留下半删除状态。
- Editor preflight 保留 Runtime topology 结果并叠加 `ProjectOnly` write policy；允许项进入统一 Job System，删除与 relocation/import 共用工程资产 mutation mutex。关闭工程时只请求取消，后台 transaction terminalize 前不拆 owner。
- 资产工作区右键命中稳定 UUID，context-menu item 经 `AssetCommand::DeleteAsset`、binding codec、host event、editor effect 到 retained-host side effect；没有字符串命令旁路或旧架构 fallback。
- referencer 非空不再作为状态栏字符串错误返回。retained presentation 保存完整、去重且稳定排序的 referencer 模型；绘制只 materialize 对话框可见行并使用构建期 overflow label，pointer overlay 吞掉底层点击，唯一 Close 动作通过 typed host callback 清理模态。全量 presentation rebuild 保留该数据，并在窗口尺寸变化时共享 referencer storage 重新布局。
- Runtime `ProjectManager` 持有按文档替换的有界引用诊断存储：写权限保持 crate-private，Editor 只读 snapshot/latest event；每次有效观察递增 sequence，同一文档的新观察替换旧项，成功 load/save 清除该文档诊断，无关 I/O/解析失败不伪造“已修复”。`DanglingAssetReference`、持久 `AssetRef` 悬挂/subasset 悬挂和无法持久化的 `ResourceHandle` 均投影为 typed kind。
- Editor17 的唯一 Activity/日志源消费本次操作 sequence 之后的新事件，输出稳定 key/value 诊断并携带 asset jump；`EditorManager::project_reference_diagnostics` 提供面板数据源，不另建第二份 sticky UI 状态，也不让旧事件冒充当前失败。
- `DocumentToolkit::validate_references` 为无默认实现的必选契约。Editor09 批量保存 preflight 校验当前 live document，真正执行写入前在 toolkit save lease 内再次校验，排除排队期间状态变化绕过；UI asset 的 stale import/canonical source 与 animation document 的当前序列化引用均由各自文档 owner 校验。
- 本切片没有执行性能优化。删除检查只消费离线 registry reverse index，不在 UI 线程扫描目录；候选准备和文件读取位于 generation write-lock 外。若后续 profiling 证明删除准备或 commit 有瓶颈，再按 optimize 计划记录 trace、规模曲线和功耗证据后实施结构性优化。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| Editor10 M3 | source + sidecar durable safe-delete transaction | `实现完成-事务域动态门通过-Runtime全包受共享基线阻断` | 2026-08-28 | `PreparedFileWrite` 支持多个 retirement，journal/stage/commit/rollback/recovery 全链按 retirement vector 处理；`one_generation_can_durably_retire_a_source_and_its_sidecar` 与 `recovery_restores_every_retirement_after_a_partial_multi_file_delete` 已加入。D 盘独立 manifest 执行 `cargo test --manifest-path D:\zt\zr-resource-validation\Cargo.toml --lib io::transaction:: --offline`：32 passed、0 failed、123 filtered、0.36s。 |
| Editor10 M3 | Runtime registry/resource/catalog 删除 generation | `实现完成-集成测试源码完成-全包行为门待共享Runtime修复` | 2026-08-28 | 删除 preflight 覆盖未引用根资产、labeled subasset、稳定 referencer 顺序及整 source companion edge；pipeline 集成测试锁定 source/meta 消失、asset/resource 查询清零、单次 Removed 发布和 reopen 后 registry 持久态。快照 `cargo check -p zircon_editor --lib --offline` 首轮被 `zircon_runtime_host` 非穷尽匹配阻断；D 盘临时补齐后继续被 `zircon_runtime` 78 个既有错误阻断，尚未执行 Runtime 删除集成测试。 |
| Editor10 M3.2 | Editor 删除命令、job 与引用阻断模态 | `实现完成-模块动态门4/4通过-Editor全包待共享Runtime修复` | 2026-08-28 | typed command/effect/side-effect 链、UUID context target、preflight、mutation mutex、poll/cancel/close 行为与 blocker presentation 已接通。D 盘最小模块夹具直接 `#[path]` 编译 E 盘生产 data/painter/pointer 源，`cargo test --manifest-path D:\zt\editor-plan10-blocker-validation\Cargo.toml --offline`：4 passed、0 failed；覆盖完整 referencer retention、紧凑窗口 frame containment、overlay click consumption/Close callback、painter 数据消费。相关 Rust scoped rustfmt 与 21 文件 trailing-whitespace 扫描通过。 |
| Editor10 M3.2 | 悬挂事件、诊断面板数据源与 Editor09 保存前校验 | `实现完成-生产源码隔离门4/4通过-全工作区门待共享Runtime修复` | 2026-08-28 | Runtime 新增 per-document replacement snapshot + bounded latest typed event，scene load/save 发布 direct/persisted/subasset/unresolved-handle 诊断；Editor Activity 投影仅消费本次 operation 后的新 sequence 并带 asset jump，manager 暴露面板 snapshot。`DocumentToolkit` 硬切必选 live-reference hook，批量 preflight 与实际 save lease 双重校验，UI asset/animation owner 均已接入且没有兼容默认。D 盘隔离工程直接 `#[path]` 编译 E 盘生产 `reference_diagnostics.rs`、`document_toolkit.rs`、`registry.rs` 与 save error，执行 `cargo test --manifest-path D:\zt\plan10-reference-validation\Cargo.toml --offline`：4 passed、0 failed、0 ignored，0.02s；覆盖跨文档替换清理、有界 latest event、invalid-before-write 和 valid-save。scene error 映射测试源码已加入但 Runtime 全包受共享基线 146 errors 阻断，尚未执行。 |

当前阶段没有满足完整里程碑提交条件：Editor/Runtime 全包门尚未进入本切片 scene 行为测试，M3 的 rename/fix-up 与完整回归也未关闭。因此未创建 git commit、未向协调器登记 accepted milestone、未发送企微完成通知。
