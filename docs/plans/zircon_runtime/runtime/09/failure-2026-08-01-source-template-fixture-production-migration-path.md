---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: source-template-fixture-production-migration-path
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/ui/template/asset/schema/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime/src/ui/tests/asset/fixture_migration.rs
  - zircon_runtime/src/ui/tests/asset_schema_migration.rs
  - zircon_runtime/src/ui/tests/asset_contract_spine.rs
tests:
  - cargo test -p zircon_runtime --no-default-features --lib ui::tests::asset::fixture_migration::ui_asset_loader_rejects_source_template_documents_without_asset_header --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --no-default-features --lib ui::tests::asset_schema_migration --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime_interface --lib ui::template::asset::schema --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime09：source-template fixture 生产迁移路径残留

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-01 plans/code/obsolete-test 并行审阅
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 交接原因：Runtime09 已指定 `.zui`/带 `[asset]` UI asset 为生产 authority，但正式 loader 仍可把无 `[asset]` 的历史 `UiTemplateDocument` 转成当前 tree document；Runtime15 又以源码字符串门禁固化了这个中间态。

## 失败现象与复现证据

`UiAssetSchemaMigrator::migrate_toml_str` 在缺少 `[asset]` 时解析旧 `UiTemplateDocument`，再调用 `migrate_source_template_fixture_document`。显式 fixture helper 的消费者全部是单测，生产侧唯一入口是 `UiAssetLoader` 经该 fallback 间接接受旧格式。与此同时产品 importer 已拒绝 `.ui.toml` 与 `.v2.ui.toml` source-template matcher，形成“importer 禁止、formal loader 接受”的双语义。

配套 `source_template_fixture.rs`、两个 public helper、两个 report enum variant、四个转换测试以及 Runtime15 源码字符串命名门禁只为这条退役路径服务。继续保留会让无 `[asset]` 输入绕过 canonical UI asset header/version policy，并让过时测试承担编译成本却不保护当前产品合同。

## 最低共享层根因

历史离线转换责任被留在生产 schema migrator 内，测试又反向把兼容分支当作必须长期存在的 API。Runtime09 的 hard cutover 未完成：新 authority 已存在，但旧 source shape、报告词汇和结构锚没有在同一变更中退休。

## 架构修复验收

- `UiAssetSchemaMigrator` 对所有生产输入先要求 `[asset]`，继续支持 current/older tree 与 flat node-table，不再猜测 `UiTemplateDocument`。
- 删除 source-template converter module、public fixture helper、interface report variants及所有转换行为测试，不留 alias、shim、re-export 或 test-only converter。
- 新增真实 loader 行为测试，断言无 `[asset]` source-template 返回 `UiAssetError::ParseToml("ui asset source is missing [asset]")`。
- 删除 Runtime15 纯源码字符串命名门禁，更新 test-file budget 与当前 runtime/interface/UI 文档；历史 archive/output evidence 保留。
- 通过 managed Windows focused runtime/interface gates 后写 fixed return；测试体未执行不得把本记录改为 fixed。

## 禁止临时方案

- 禁止把 converter 移到 `tests/support`、保留 public helper 给“以后迁移”、恢复 report enum alias，或在 loader/importer 两侧维持不同 source-shape policy。
- 禁止用 include-string/源码字符串断言代替旧格式拒绝行为测试。
- 离线历史资产转换必须产出 canonical tree source 后再进入 runtime，不得复活 production compatibility branch。

## 修复结果与回传

Open state：`implementation_complete / runtime_gate_blocked`。2026-08-01 已原子删除生产 fallback、转换 module/API、report variants、三处过时转换测试 owner 与 Runtime15 字符串门禁，并新增 loader rejection 行为回归。

- managed Windows interface gate `5ca7c0c6ff084cab909962000c1cf0c9`：`cargo test -p zircon_runtime_interface --locked --lib ui::template::asset::schema` 退出 `0`，作业已释放；interface crate 与筛选命令通过。
- 同一 current-source 上的 runtime lib-test 编译作业 `38911e64b09e4cefa6b2e5d6069676f6`：退出 `1`（内部 Cargo `101`），15 个其他 owner 的编译错误、327 个 warning、目标测试执行 `0`，作业已释放。错误位于 Dynamic Scene/Asset、Runtime11、Plugins01 等边界，不触及本记录修改路径。
- 因 loader rejection 行为测试尚未实际执行，本记录继续保持 `open`；不得用 interface gate 或静态检查代替 runtime 行为 GREEN，待 Runtime09 owner 在共享编译边界恢复后重跑首个 `tests` 命令并写 fixed return。
