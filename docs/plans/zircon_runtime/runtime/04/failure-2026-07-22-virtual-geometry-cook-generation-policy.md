---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: virtual-geometry-cook-generation-policy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/virtual_geometry_cook
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/mesh_importer.rs
tests:
  - cargo test -p zircon_runtime --lib asset::tests::virtual_geometry_cook --locked --jobs 1 -- --nocapture --test-threads=1
  - runtime, glTF, OBJ and model importer feature-off/on, cold/warm and large-mesh matrices
---

# Runtime04：Virtual Geometry cook generation与请求策略缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset virtual_geometry_cook 5/5逐Rust文件性能审查，PERF-MVP-509
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：mesh content identity、cook artifact generation与last-good必须由Runtime04统一拥有；请求开关由Plugins12 typed importer capability提供，不能散落在各格式插件。
- 生命周期键：`virtual-geometry-cook-generation-policy`

## 失败现象与复现证据

runtime内置primitive构造以及glTF、OBJ、model importer插件均对所有非蒙皮primitive同步调用VG cook，即使MVP没有启用VG consumer。相同content/config没有generation cache；leaf bounds、递归hierarchy和page encode串行执行，父层反复扫描全部source summaries。重复导入、插件切换或热重载会再次承担完整cook CPU/RSS。显式inspection dump还会多轮排序/扫描并物化完整String和payload word Vec。

## 最低共享层根因

import contract没有typed VG request policy，asset pipeline没有content+config keyed immutable cook artifact/single-flight，task system也没有为可确定性合并的cook stages提供有界调度。

## 架构修复验收

- Plugins12发布project/profile/plugin capability驱动的typed VG request；feature-off所有runtime/plugin importer路径cook calls=0。
- Runtime04以mesh content hash、cook schema和config建立唯一immutable artifact generation；same generation并发/重复import共享result，失败保留last-good。
- Runtime11按triangle/bytes/RSS预算并行leaf bounds、cluster与page encode，stable ordinal merge；cancel、supersede、plugin unload与shutdown有界。
- hierarchy由child summaries组合，禁止父层反复全扫leaf source；diagnostic dump只显式请求并流式写出。
- 参考UE NaniteBuilder按cluster/page/encode阶段使用`ParallelFor`并稳定assembly；不引入与Zircon格式、可移植性或MVP不匹配的Nanite完整实现。
- triangles/clusters/pages 1/1k/1M、feature off/on、cold/warm/1% change、四条import路径记录cook/source visits、jobs/queue/RSS、artifact owner/bytes和caller blocked：off=0，on≤1/content+config generation，warm=0，work近O(T+C+P)，caller blocked=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止每个importer/plugin维护私有VG cache、thread pool或环境变量开关。
- 禁止仅把同步cook搬到未设容量的detached task。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
