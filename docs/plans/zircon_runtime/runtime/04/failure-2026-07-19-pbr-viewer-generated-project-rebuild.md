---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: pbr-viewer-generated-project-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/bin/zircon_shader_pbr_viewer/project_assets.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs
tests:
  - cold and warm generated-fixture startup counters
  - single project-open and import-generation regression
  - viewer scene and pixel parity
---

# Runtime04：PBR viewer生成project重复构建

## 现象与根因

PBR viewer每次启动都在临时目录生成固定96×192 sphere（18,721 vertices、110,592 indices）、material和scene的大TOML文件。为获得model/material UUID，writer先`ProjectManager::open + scan_and_import`；runtime asset manager随后再次打开同一project。稳定诊断fixture没有版本化artifact owner，因此cold/warm运行都重复CPU mesh生成、序列化、文件I/O、扫描和导入。

## 修复验收

- 固定sphere/material/scene成为版本化builtin或checked-in/generated-once fixture；warm启动mesh generation/TOML write/import为0。
- 若必须生成，使用content/version key的persistent artifact并原子发布；同generation project open/scan/import至多1次。
- viewer仍通过真实AssetManager/SceneRenderer路径加载，不得改为test-only构造器或跳过引用验证。
- cold/warm各记录generation samples、serialized bytes、fs writes、open/scan/import count、F2 wall/p95；场景、material和多角度像素等价，结果回传PERF-MVP-428。

## 禁止临时方案

不得仅延长临时目录寿命，不得按进程内静态bool跳过跨进程验证，不得让fixture私有缓存绕开Runtime04 artifact/version合同。

## 修复结果与回传

Open state: `待 Runtime04发布版本化viewer fixture并回传cold/warm启动计数与产品像素证据`。
