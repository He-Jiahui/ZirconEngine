---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-importer-generation-index
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_plugins
tests:
  - cargo test -p zircon_runtime --lib asset::tests::assets::importer --locked --jobs 1 -- --nocapture --test-threads=1
  - importer, matcher, plugin reload, raw descriptor and selection storm matrices
---

# Runtime04：asset importer generation索引缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset importer 40/40逐Rust文件性能审查，PERF-MVP-503
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：importer matcher、capability和plugin load/unload必须随Runtime04/Plugins12统一generation发布，查询端或Editor不能建立第二套registry truth。
- 生命周期键：`asset-importer-generation-index`

## 失败现象与复现证据

registry只存importer Vec。每次select全扫full suffix或extension candidates，capability ranking对DiagnosticOnly候选clone完整message；注册每个matcher都重扫全部existing matcher并重新规范化/格式化key，最坏O(I²M²)。Default与active registry还重复构造数十descriptor、clone importer Arc Vec。查询期matcher String分配已在PERF-MVP-502止损，但全扫和generation缺口仍在。

## 最低共享层根因

descriptor注册结果没有一次规范化成extension/full-suffix/id/plugin索引，也没有plugin transaction拥有的immutable generation和compact availability状态。

## 架构修复验收

- generation维护extension→ordered candidate slots、longest full-suffix index、id→slot、plugin→slots和compact availability/status Arc。
- 注册transaction一次规范化matcher并增量校验duplicate；plugin unload只更新owned slots，失败候选不发布。
- select只访问相关candidate list，不clonedescriptor/status message；raw descriptor whitespace/dot/case语义与当前一致。
- importers/matchers/plugins 1/100/10k、selects 1/1k/1M记录visits、normalize/String/status clone、index update bytes和p95：stable select近O(candidates)、alloc=0、generation build≤1。
- 参考Bevy `AssetLoaders.extension_to_loaders: HashMap<Box<str>, Vec<usize>>`，但保留Zircon priority/availability/longest suffix与插件卸载合同。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止Editor或每个plugin缓存第二份extension map。
- 禁止只缓存最后一次path选择而保留reload后stale importer Arc。

## 修复结果与回传

### 2026-07-31 current-source forward repair

- `zircon_runtime/src/asset/importer/registry.rs` now publishes an immutable
  `Arc<AssetImporterRegistryGeneration>`. A generation owns stable optional
  slots plus extension/full-suffix candidate indexes, importer-id slots and
  plugin-owned slot lists. `AssetImporterRegistry::clone` is therefore a
  generation snapshot; register and unload publish copy-on-write successors.
- Registration normalizes every matcher once before publication, validates
  duplicate ids/matchers against the current indexed generation, and only then
  mutates the successor. Capability availability and its diagnostic status are
  captured in a shared `Arc` at that boundary, so candidate selection reads the
  compact rank without cloning diagnostic messages or descriptors.
- Full suffix lookup visits only dotted tails in the queried file name and
  their indexed candidates; extension lookup visits its one indexed candidate
  list. Both use allocation-free ASCII-folded lookup hashes, preserving the
  pre-existing raw descriptor whitespace/dot/case matching semantics.
- Plugin unload removes only slots owned by that plugin and their recorded
  matcher references. The regression in
  `asset/tests/assets/importer/registry_priority.rs` asserts fallback selection
  after unload and that a cloned published generation still resolves the old
  importer.

Static formatting and scoped `git diff --check` are clean. No Cargo command was
started locally: the exact managed importer command in this record remains the
required terminal acceptance evidence, including the 1/100/10k registration and
selection matrices. The 2026-07-31 independent review found `0 Critical / 0
Important / 0 Minor` in the scoped generation, matcher and plugin-unload paths.
This failure remains `open` until the managed matrix has terminal evidence.
