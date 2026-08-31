---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: zui-importer-provider-linkage-regression
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/entry/tests/profile_bootstrap/first_party_runtime_plugins.rs
tests:
  - python -B -m unittest tools.tests.test_frameworks_05_asset_ui_boundary tools.tests.test_zui_static_suffix_convergence -v
  - python -B tools/runtime_domain_dependency_audit.py --repo-root .
  - cargo +1.94.1 test -p zircon_app --lib --features first-party-runtime-plugins --locked runtime_profile_bootstrap --jobs 1 -- --test-threads=1
  - cargo test -p zircon_first_party_runtime_catalog --no-default-features --features ui-document-importer --locked --lib ui_document_importer --jobs 1 -- --test-threads=1
---

# Frameworks05：ZUI importer provider linkage 回退

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 successor 准入复核 / UI importer MVP 闭环
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：最低共享根因是 Asset 与 UI importer 插件扩展契约被破坏。Frameworks05 已声明
  `AssetImporterRegistry` 是唯一 loader 扩展面且 `asset -> ui` 必须为 0；provider 的编译链接与 App
  composition 是完成该 contract 所必需的上行闭环。

## 失败现象与复现证据

current HEAD `68edcd71042de817a74d4ad70efc07cfe2c72bfa` 上，commit
`7a20f921bb97ed428ae248cbcaf3c2fac5442ddf` 在先前 hard cut 之后重新创建
`zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs`，并把 `.zui` descriptor 重新装入
默认 Asset importer。该文件直接调用 `crate::ui::v2::UiZuiAssetLoader`。

- `python -B -m unittest tools.tests.test_frameworks_05_asset_ui_boundary -v` 为 2/3，唯一失败精确指向
  `import_ui_v2_asset.rs:9` 的 `asset -> ui` 引用。
- ZUI 全静态组发现 18 个测试，16 个通过、1 个 failure、1 个 foreign error；本 failure 的失败是
  `test_retired_runtime_importer_files_stay_deleted` 发现上述退役文件复活。另一条
  `native_host_contract.rs` missing-path error 属于 EditorUI guard 漂移，不属于本修复 scope。
- fresh dependency audit 为 2,749 production references / 72 domain edges，包含 `asset -> ui = 1`。
- `zircon_plugin_ui_document_importer_runtime::plugin_registration()` 已提供唯一 `.zui` importer，
  `RuntimePluginId::UiDocumentImporter` 与 builtin catalog row 也已存在；但
  `zircon_first_party_runtime_catalog` 没有该 optional dependency/feature/provider branch，App 的
  Client/Editor feature 也未链接它。App provider projection 还读取 raw project manifest，而 Runtime
  module selection 使用 `manifest_with_mode_baseline`，形成两套 effective selection authority。

## 最低共享层根因

Asset fallback 被当成 provider linkage 缺失的补偿路径重新引入，掩盖了 first-party catalog 与产品
feature composition 没有链接 `ui_document_importer` 的事实。结果是 Asset core 重新依赖 UI
implementation，默认 Client/Editor 仍不能从同一 effective manifest 证明 plugin registration，且旧
fallback 与插件 importer 同时存在双 owner 风险。

## 架构修复验收

- 为 first-party runtime catalog 增加独立 UI document importer feature/provider；App 用独立 feature
  将其链接进 Client/Editor，不得为单个 importer 强制启用整个 base plugin bundle。
- App provider projection 与 Runtime module selection 共用 `manifest_with_mode_baseline`；Client/Editor
  baseline 选择 UI 与 UI document importer，Server 不选择。显式 override 仍由同一 merge 算法处理。
- provider registration 在默认 Client/Editor composition 中恰为 1；未编译 provider 时保留 structured
  unavailable diagnostic，不允许静默回退。
- 同一 hard cut 删除 `import_ui_v2_asset.rs`、ingest module declaration 与 builtin `.zui` descriptor，
  不保留 forwarding module、alias、re-export 或 compatibility fallback。
- focused Asset/UI 与 retired-importer guards GREEN；dependency audit 恢复 `asset -> ui = 0`；catalog、
  App profile/bootstrap tests 和 managed Client/Editor product compile 通过后才可回传 fixed。

## 禁止临时方案

- 不得重新公开或保留 builtin UI importer 作为插件不可用时的 fallback。
- 不得新增 alias、compatibility feature、重复 manifest merge、test-only registration 或调用点特例。
- 不得通过放宽 source guard、排除 offending file 或默认链接全部 base plugins 隐藏缺口。

## 修复结果与回传

Current state: `implemented_static_review_accepted_catalog_rust_green_app_foreign_particles_blocked_r4`;
failure remains open.

- 已完成 hard cut：删除 Asset owner 中的 `import_ui_v2_asset.rs`、module declaration 与
  `zircon.builtin.ui.zui` descriptor；没有保留 alias、re-export、fallback 或 test-only
  registration。
- 已完成 provider 链接：first-party runtime catalog 新增独立 `ui-document-importer`
  feature/provider，App `ui` 产品 feature 只链接该 provider，不强制启用完整 base plugin bundle。
- 独立复核首次结论为 Critical 0 / Important 1 / Moderate 0 / Minor 0。Important 精确指出 provider
  projection 已应用 baseline，但 profile module selection 仍向 availability 传 raw profile manifest；在
  缺少 importer registration 时可能不生成 `missing_required`。因此原先“单一 selection authority / structured
  unavailable 已闭环”的表述已撤回，failure 未提前回传 fixed。
- r1 受管 Cargo 自然结束后按 coordinator immutable-scope 协议取消，r2 完整继承 15 份 current blob，新增
  `zircon_app/src/entry/builtin_modules.rs`；transfer fingerprint
  `4737dafd69099df9761f0f4ba533bd200a698d8f3da7d1c6370cb402e51a1596` 已原子 apply。已删除
  importer 路径另由 r2 领取 missing-path lease 与 current attribution，没有复活文件。
- 已完成 selection authority follow-up：`effective_project_plugin_manifest` 是 App config bootstrap 的唯一
  effective manifest owner；它调用 Runtime 的 `manifest_with_mode_baseline` 并合并 render-profile overlay，
  同一个值同时传给 first-party provider projection、profile/target availability 与 module selection、feature
  dependency projection、plugin lifecycle projection。provider 文件不再拥有第二份 overlay 函数。
  Client/Editor baseline 同时选择 `Ui` 与 required `UiDocumentImporter`，Server 均不选择，显式 importer
  disable override 仍由同一 merge 算法处理。
- unavailable diagnostic 继续由既有 Runtime availability 单一 owner 保持：新增 public profile bootstrap
  负向回归移除 `UiDocumentImporter` registration，并要求 canonical
  `required runtime plugin UiDocumentImporter is unavailable` fatal detail；没有新增 catalog-local fallback、
  preset 重复 selection 或第二套诊断 owner。独立复核指出默认入口开始自动链接 first-party provider 后，
  旧 VirtualGeometry unavailable 测试会随 advanced-render feature 改变；该测试现已显式传入空 registration，
  因而稳定验证 unavailable owner。两条 Rust 回归均已写入，仍需 managed Cargo 实际执行。
- 第二轮独立复核继续发现两处 MVP 启动/执行 authority 问题并已修复：第一，默认
  `BuiltinEngineEntry::for_config` 曾走无 registration 的 private selection，在 `ui` 产品 baseline 把 importer
  标成 required 后会直接 fatal；现在默认入口自动消费 linked first-party provider。第二，provider 与 module
  selection 虽使用同一 merge 算法，却曾分别物化 effective manifest；现在 `engine_entry` 只物化一次，并把
  同一 `ProjectPluginManifest` 引用同时传给 provider projection 与
  `builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_registrations`。无 provider 的旧 private
  `builtin_modules_for_config` 已硬删除，不保留第二条默认路径。
- coordinator scope 继续按 immutable discipline 轮转：r2 因 `profile_bootstrap.rs` 达 1,122 行而取消；r3
  在未接管 blob 前因上述 single-instance review finding 取消；r4 新增 `engine_entry.rs` 与 folder-backed test
  child scope，并通过 ownership-transfer fingerprint
  `0d8be45fd9dea976b27e0eb29d14ffa17af43e9640b2adecdf2de6403091a58c` 原子继承 r2 的 16 份 current blob。
  first-party/profile/render tests 已从 1,122 行 parent 抽取为 697 行 parent + 449 行 child，未新增平行 test
  authority 或修改测试语义。
- RED/GREEN：第一轮跨 crate 静态守卫先精确失败于 Runtime baseline 缺少
  `UiDocumentImporter`；独立复核 follow-up 守卫又先失败于 App projection 未消费
  `effective_project_plugin_manifest`，实现后 Frameworks05 Asset/UI 与完整 ZUI suffix suite 合计 22/22 GREEN。
  App、Runtime manifest、catalog 共 10 份涉及 Rust 文件的 Rust 1.94.1 scoped `rustfmt --check` GREEN，scoped
  `git diff --check` 也为 exit 0。
- dependency audit：最新 current-source 为 2,753 个 production references / 71 条 domain edges，`asset -> ui` 从
  1 收敛为 0。完整 `cargo metadata --locked` 在补齐 catalog dependency 与 importer package
  lock record 后 GREEN。
- managed validation：catalog ticket `fc0ee858c7cb45d3b1cde38e6e716360` 使用 source manifest
  `60f6198836fd1e349397293106efcf97f3145bc0a75a6d82a0659ba374eca54d`；locked metadata 已通过，
  但 isolated materialization 在 Cargo 启动前被 Runtime74 的 5 个 foreign baseline drift 拒绝：
  `binding_targets.rs`、`binding_param_resolver.rs`、`control_scope.rs` 以及两份对应
  `control_scope` 测试。该 ticket 不构成本 failure 的 Cargo 通过证据；当前等待共享 Cargo 窗口后
  继续 coordinator-managed current-source 集成诊断，尚未声明 Client/Editor product compile、
  independent review 或 failure closeout 通过。
- current-source managed Cargo job `5933aa1c25454b4487f3998b6523dbad` 使用 D 盘 leased target
  `D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`，
  运行 1,738 秒后 exit 101 并正常 release。它在目标测试生成前被 6 条 foreign current-source 编译错误
  阻断：`refresh_runtime_dependency_closure` 与 `PreparedProjectSourceRelocation` 三处 relocation 导出问题、
  `ResourceLocator::as_str`、`ResourceLocator` 被 thiserror 解释为 source、以及
  `resource_publication.rs` 一处 E0282。上述路径不在本 session scope，本次不越权改写；该 job 不构成
  catalog 或 App Rust GREEN 证据。
- 2026-08-23 10:05:52 起共享 Cargo lane 由 UI12 job
  `b2faf5f3a58e4708b7980c1b54f35f75` 合法占用；Frameworks05 未取消或抢占 foreign job，也未在其间启动
  新 Cargo。当前继续完成 current attribution、独立复核与 failure 记录，等待真实 admission window 后再补
  focused catalog/App Rust 与产品 compile 证据。
- 第二轮最终独立 source review 已接受：Critical 0 / Important 0 / Moderate 0 / Minor 0。reviewer 独立确认
  默认 `for_config` 只物化一次 effective manifest，并把同一引用用于 provider 与 builtin selection；missing-importer
  negative、显式 disable、Server exclusion、catalog 单次 O(n) 投影和 Asset hard cut 均成立。独立验证为 Python
  22/22、dependency audit 2,753 references / 71 edges / `asset -> ui = 0`、expanded `rustfmt --check` 12 files
  GREEN、scoped `git diff --check` GREEN；Rust positive/negative 仍必须由 managed Cargo 给出动态证据。
- App focused reservation `512be425bf8c42469582577a792b2e3b` 被消费为 job
  `9488aee1514f4b51a6e13b86b09e0e35`、run `4e9d8f3b905f4367bbdea33fa3201be5`，使用 D 盘 retained pool
  `D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`。
  exact command 从 10:27:41 运行到 11:15:54，完整编过 `zircon_runtime`、first-party importer/provider 前置闭包与
  sound runtime 后，才在 foreign `zircon_plugin_particles_runtime` 编译阶段 exit 101：
  `render/gpu/neutral_buffers.rs:262` 对 `BufferViewMut` 调用不存在的 `fill`，以及
  `render/runtime_prepare.rs:113`、`:115` 两条 E0502 可变/不可变借用冲突。Frameworks05 没有改写这两个
  Particles 路径；Windows PID 复核为空后，coordinator `finish --exit-code 101` request
  `cce5bf3e970d489ab0c183c8d89e2b95` 写入 terminal failed，随后 release request
  `5be858bcbff14de6b564a14e18b41f2d` 成功，`live_process_pids = []`。因此该 run 证明本 scope 已通过此前
  Runtime/first-party compile frontier，但不构成 App focused test GREEN。
- Particles ownership matrix request `dd44fe631f61461095ccb0eb76ca3b13` 显示
  `neutral_buffers.rs` 为 `attribution_missing`，`runtime_prepare.rs` 仍指向 archived owner 且 attribution stale，
  两者均无 live lease；该跨计划 current-source compile blocker 必须由 Particles owner 修复或正式接管，不能混入
  Frameworks05 commit。
- catalog exact reservation `111aa36b69c649c08a0fba047c7f0bb5`、command fingerprint
  `dc75da51b35bc907c779179e82c4409a170410b521e2b1113c4926a3fb359248` 首次 consume 被 UI12 check job
  `1cc68d7bb3704c2e9543f8f62920a435` 以 `cargo_reuse_pool_busy` 正确拒绝；Frameworks05 未取消或抢占 foreign
  job。UI12 自然 release 后，同一 reservation 被消费为 job `7e54a4d6916142ed94140b00fb1a0d9b`、run
  `d6822f00b78949dea6a487366e6114aa`，复用 D 盘 pool。完整 build 用时 37m44s，真实执行
  `runtime_catalog_projects_the_selected_ui_document_importer_provider`，结果 1 passed / 0 failed / 8 filtered；
  `zircon_plugin_ui_document_importer_runtime` 与 `zircon_first_party_runtime_catalog` 均编译成功。coordinator
  finish request `66b1fbbcfd43481d84b05f06a07d6c38` 记录 exit 0，release request
  `d16b8db33c514b96874d1d23811451b5` 成功且 `live_process_pids = []`。catalog provider 动态门已 GREEN；
  剩余 Rust 阻塞仅为 App product closure 中已交接 Plugins09 的 Particles compile regression。
