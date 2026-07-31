---
related_code:
  - zircon_runtime_interface/src/project
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_hub/03-project-lifecycle-robustness.md
reference_sources:
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/godot/editor/project_manager/project_list.cpp
tests:
  - zircon_runtime_interface/src/project/tests
  - zircon_runtime_interface/src/project/persisted_asset_reference.rs
  - current-source Windows project contract tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface project 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/project/**` 当前源 **39/39** 个 Rust 文件、**1,682** 行已逐文件阅读，覆盖 asset reference、manifest summary/migration、project name、relative path、retired reference migration、embedded template pack和目录测试。测试清册为目录内22条加 `persisted_asset_reference.rs` 就地2条，共 **24** 条。生产反查覆盖 Runtime manifest open、Editor `ProjectAuthority` create/probe/recent、Hub create/recent/validation。该目录当前无工作区改动，本轮未修改源码。

## 性能结论

- `load_project_manifest_value_from_toml_str` 固定执行 `TOML text → toml::Value → serde_json::Value → migration`；summary随后再从 JSON Value反序列化 `SummaryDocument`，Runtime完整 manifest也从同一 JSON Value反序列化 `ProjectManifest`。每次 Hub recent refresh、Editor probe和Runtime open都会构造两棵宽 DOM并遍历至少三次，新增 **PERF-MVP-568**。
- template create先把14个编译期 `&'static [u8]` 全部复制为 owned Vec；manifest再 parse为 TOML table、pretty encode为新 String/Vec，紧接着 `ProjectManifestSummary::parse_toml_bytes` 再次走 TOML→JSON。Editor落盘后又 `ProjectManifest::load`并立即 `save`，同一次 F0 create对 manifest形成多轮 parse/migrate/serialize，且非 manifest模板大文件也先完整复制后才写 staging。该链路并入568。
- `SummaryDocument::validate`先用 `BTreeSet`去重 asset roots，再对原 Vec做两两 overlap检查，最坏 O(R²)；manifest未给 roots数量/总字节上限。`validate_project_name`仅为单组件判定收集 `Vec<Component>`，并为 Windows保留名分配 uppercase String；`RelPath::parse`正常路径固定产生 separator-replace String、component Vec和join String，`join_to`再建临时 PathBuf；`AssetRef::serialize`为 borrowed输入 clone完整 `RelPath`和sub String。新增 **PERF-MVP-569**。
- retired reference walker消费 owned JSON而不深 clone叶子，但每层 Array/Object仍重新 collect容器，且递归深度/节点数由上游 document决定。它只在明确迁移期执行，继续归 Runtime04 PERF-MVP-511/512的single inventory、streaming transaction与统一payload budget，不另建第二迁移 owner。
- `AssetRef`/`RelPath` typed constructor、builtin/project判别、fixed bincode golden、strict unknown-field与跨平台路径拒绝语义完整；任何微优化必须保持错误优先级和79-byte historical encoding，不能用不安全字符串旁路换性能。

## PERF-MVP-568 设计

1. Runtime04定义唯一 content-fingerprint/generation 绑定的 `ProjectManifestArtifact`：一次 TOML parse、一次 in-place version migration，直接从 migrated TOML value产生 typed runtime manifest或轻量 summary，不再经过 `serde_json::Value`。Runtime open从完整 typed manifest借用/投影 summary，不二次 parse。
2. template render在同一 migrated TOML table中改名、生成 summary并只 pretty encode一次；Editor create不得在写完同一 bytes后立刻 load+save。非 manifest entry保持 borrowed static bytes或共享 immutable chunk，Editor/Hub直接流式写 staging；只有实际改写的 manifest拥有新 Vec。
3. Editor10/Hub03以 manifest content fingerprint + generation共享 recent/probe/create结果；稳定列表不重读/重parse。mtime可作为便宜候选失效提示，但不能成为唯一内容权威；失败保留last-good summary并报告stale/invalid状态。
4. template/source和runtime完整字段仍只有一个 schema/migration owner；不得让 Hub、Editor和Runtime各自维护第二 parser/cache，旧 JSON migration中间层在所有caller迁移后硬删除。

## PERF-MVP-569 设计

1. `AssetRef` Serialize使用borrowed repr，成功路径 `RelPath`/sub clone bytes为0；`RelPath`用单遍canonical writer只分配最终 String，`join_to`直接把canonical文本join到root，不构造中间 PathBuf。
2. project name以流式 `components()` exact-one检查和 ASCII case-insensitive reserved-name判定删除 Vec/uppercase分配；保留空白、`.`/`..`、slash、prefix、forbidden、trailing和reserved错误优先级。
3. asset roots在完成canonical parse后排序/索引一次，以相邻prefix或trie检测duplicate/ancestor overlap，复杂度近 O(R log R)；manifest hard cap同时限制roots count、单path和累计bytes。
4. fixed JSON shape、bincode golden、Unicode/Windows/UNC、duplicate/overlap诊断和 path containment必须逐项回归；不得引入第二种 canonical spelling。

## 参考引擎对照

- Bevy `AssetPath`以 `CowArc`保存source/path/label，静态与借用路径可以零正文复制，只有显式 `into_owned`才取得共享owner；Zircon的持久 `AssetRef`仍应拥有最终规范文本，但解析、序列化和热路投影不必制造额外副本。
- Godot Project Manager为每个项目只创建一次 `ConfigFile`并从同一解析结果取name/version/description/tags/main scene，同时缓存project config modified time用于稳定排序。Zircon应采用更强的content generation/last-good artifact，而不是每个 recent consumer重复完整parse，也不应只信mtime。

## 动态验收

1. current-source interface/runtime/editor/hub project合同：v1→v2、future/invalid shape、semver、multi-root、template byte parity、unsafe path、transaction rollback、recent refresh与fixed bincode golden。
2. manifest 1 KiB/1 MiB/64 MiB、roots/plugins/scripts/export profiles 1/100/10k、recent projects 1/8/1k：记录file reads、TOML/JSON parses、DOM owners/map visits、serialize bytes、main/worker p95与RSS。每content generation parse/migrate/serialize各≤1，internal JSON DOM=0，stable recent parse/read=0。
3. template entries 1/100/10k、payload 1 KiB/1 GiB：记录embedded→owned clone bytes、staging writes、manifest reparse/resave与峰值RSS；非改写entry clone bytes=0，manifest parse/encode各1，Editor post-write load/save=0。
4. paths/roots 1/100/10k、segments 1/64/1k：记录String/Vec/PathBuf allocations、comparisons和wall；AssetRef serialize正文clone=0，RelPath只有最终owner，root overlap非O(R²)。F0 create/open与Hub recent产品trace通过。

current-source Cargo、规模 allocation/DOM counter、F0产品 trace和三方 hard-cut迁移未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
