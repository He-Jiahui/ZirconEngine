---
source_report:
  - docs/plans/performance/01/2026-08-23-plugin-sdk-editor-schema-and-contribution-canonicalization-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Plugin SDK editor schema与贡献canonicalization受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：新增`zircon_plugins/plugin_sdk/src/**` editor slice current 3/21和
  `zircon_runtime_interface/src/editor_contribution.rs`静态覆盖/M0；完整SDK、current Cargo、allocator benchmark、F4 WPR/RSS/
  power仍open。本Session不直接编辑受保护ledger。
- PERF-MVP-629 + Plan02 M1/M5 + Plugins01/10/12 + Editor12：让SDK宏生成process级immutable/borrowed
  `EditorPluginSchema`，metadata query不得构造plugin、consumer或report；选中provider后每editor session只创建一个
  `EditorPluginInstanceGeneration`。
- Plugins12 + `tools/cargo-zircon` scaffold：删除“plugin -> clone declaration -> clone descriptor”的generated façade链；保留
  unknown/open plugin ID与静态schema source of truth，不用全局`OnceLock<EditorPluginDeclaration>`共享mutable consumer。
- RuntimeInterface editor contribution：保留本轮`sort_unstable + adjacent duplicate`M0，受管C=0..100k benchmark补
  comparison/allocation counter；不得退回BTreeSet/HashSet第二索引，也不得删除canonical ABI order。
- Editor12：registration materialization直接消费validated DTO/schema owner ranges，失败publish=0；stable menu/status/capability
  projection的registration/report build=0。
- `docs/plans/performance/review.md`：只有SDK 21/21 current复审、current Cargo/Rust tests、canonicalization allocator counter、
  schema/instance/session矩阵及F4 startup/reload WPR/RSS/power通过后迁入。本轮不迁移、不提交milestone、不发送完成企微。
