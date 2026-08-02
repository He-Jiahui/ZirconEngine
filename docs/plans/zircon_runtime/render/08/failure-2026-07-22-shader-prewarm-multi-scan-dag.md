---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: shader-prewarm-multi-scan-dag
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/08-material-shader-permutation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/bin/zircon_shader_prewarm
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
tests:
  - cargo test -p zircon_runtime --bin zircon_shader_prewarm --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib shader_prewarm --locked --jobs 1 -- --nocapture --test-threads=1
---

# Render08：shader prewarm多轮asset scan与include DAG交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：P5 Runtime bin 40/40逐Rust文件性能审查，PERF-MVP-448
- 修复责任计划：`docs/plans/zircon_runtime/render/08-material-shader-permutation.md`
- 交接原因：Render08已经拥有prewarm manifest/variant cache和PERF-MVP-357/358；最低根因是CLI仍没有消费唯一source inventory与indexed dependency artifact。
- 生命周期键：`shader-prewarm-multi-scan-dag`

## 失败现象与复现证据

每asset root先由resource export递归扫描，随后`collect_shader_sources`再次递归；`collect_material_sources`先`AssetRegistryIndex::inspect_project`全量构建，再第三次递归找zmaterial。include dependency对每source clone imports并从头递归局部图；material按label/id线性find并clone完整`ShaderPrewarmSource`。pass×quality×geometry复制WGSL的既有PERF-MVP-357因此又叠加多轮I/O与图工作。

本轮仅删除exported resource-record Vec clone与nested JSON records Value clone，不改变目录遍历、DAG、source ownership或compile queue，交接保持open。

## 最低共享层根因

Runtime04 asset registry、Render08 prewarm manifest和CLI scanner分别构造重叠的source/material/resource真相；include依赖没有generation-owned indexed DAG。PERF-MVP-357的source-table schema若不同时吸收inventory/DAG，仍会在生成source table之前支付重复扫描和O(S×E)工作。

## 架构修复验收

- 一次bounded asset inventory产出resource record、shader source、material reference与revision；每root cold directory pass≤1，warm unchanged不重读正文。
- include import_path编译为indexed DAG，拓扑hash与反向依赖按changed closure增量更新；work近O(V+E)，cycle/missing错误确定性不变。
- material按resource id/label O(1)借用content-addressed source artifact，不clone完整WGSL/hashes。
- variant只引用PERF-MVP-357 source id；bounded worker执行assemble/Naga/WGPU/compress/write，in-flight与RSS有硬预算。
- 通过current-source Runtime bin/lib gates及1/100/10k files/sources、1/1k/100k variants的cold/warm/1% change计数。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止简单并行三轮目录扫描；必须先收敛唯一inventory，否则只会放大I/O与RSS。
- 禁止在source-table旁保留每variant完整WGSL兼容字段。
- 禁止以mtime单独作为content identity；revision/hash与错误语义必须可复现。

## 修复结果与回传

Open state: `source implementation and second static review complete; managed current-source validation pending`.

- The CLI now owns one bounded asset inventory per root. The compact warm index checks only file/directory metadata on an unchanged default run; external inputs hydrate the payload conservatively. Both the payload and index reject unsafe relative paths, missing root/ancestor directory records, symlink/reparse substitution, and untracked payload-map keys before reuse.
- Include dependencies are indexed and SCC-condensed, source payloads are content-addressed through the manifest source table, and high-fanout external hashes are interned once per batch. Inventory/DAG integrity and execution-budget failures remain typed through the implementation boundary.
- Invalid execution budgets now produce a report-level `preflight_error` with zero synthetic variant counters before registry, cache, or asset I/O; the CLI writes that JSON report and returns exit code 2.
- Focused regression owners cover shared-source provenance, warm/cold snapshot recovery, tampered paths/ancestor chains, high-fanout external hashes, bounded preflight reporting, and ignored scale fixtures. `rustfmt --check` passes for the touched production/test owners; scoped `git diff --check` reports only repository CRLF warnings. No Cargo/WGPU result is claimed until the managed validator returns.
