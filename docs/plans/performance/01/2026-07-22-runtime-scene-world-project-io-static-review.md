---
related_code:
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/serializer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/godot/core/io/resource_saver.cpp
  - dev/bevy/crates/bevy_scene/src/scene.rs
tests:
  - zircon_runtime/src/scene/tests/asset_scene
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - current-source Windows zircon_runtime scene project I/O tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene world project I/O逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/world/{project_io.rs,project_io/**}`当前源 **8/8** 个Rust文件、**1,541** 行、**1** 个就地test已逐文件阅读；范围包含SceneAsset↔World投影、camera/mesh/physics/post-process/reference/script/transform转换、legacy project JSON保存/加载与normalize，并追到LevelManager snapshot consumer和SceneSerializer。

## 已直接修复

- `save_project_to_path(&self)`原先先`self.clone()`深复制完整World到`ProjectDocument`，再pretty serialize成第二份完整String。现增加只用于Serialize的`ProjectDocumentRef<'world>`借用World，移除这次全场clone；on-disk JSON合同不变。
- `normalize_loaded_state`原先仅为修改其他component maps先collect整份entity-id Vec。现按`entities` index读取Copy id，无临时Vec。
- 每个builtin resource fallback save都会重新parse四个locator并重算ResourceId。现用`OnceLock`一次构建builtin `(id, locator)`表，后续只比较Copy id并clone命中locator。
- 三组源码守卫先RED后GREEN，rustfmt/diff通过，本轮归PERF-MVP-462。current-source受管Cargo最近一次申请仍被`runtime10-runtime03-animation-frame-demand-producer-20260722`精确预约，未运行raw Cargo。

## 同步I/O与宽投影仍未解决

`LevelManager`先`level.snapshot()` clone World，再同步调用本文件pretty serialization和`fs::write`；本轮只删除第二次World clone，第一份snapshot、完整JSON String、目录/写盘与caller-thread wall仍存在。`to_scene_asset`逐entity构造宽NodeRecord、解析script-binding JSON、解析每个asset reference并物化完整SceneAsset；这些是保存artifact的必要工作，但不应阻塞editor主线程，也不应在unchanged generation重复。

PERF-MVP-453继续由Runtime04 open `project-source-index-targeted-import` failure承接，Runtime11共同提供bounded I/O lane：prepared project generation按world/content revision发布immutable scene artifact ticket，后台完成projection/serialization/atomic replace，save请求按generation single-flight/merge；shutdown显式flush。不得把非原子`to_writer`直接替换当前先serialize后write流程来冒充异步/崩溃一致性。

## 参考引擎对照

Godot `ResourceSaver`先按resource/path选择format saver，再由统一保存入口处理资源路径、timestamp与回调；scene consumer不各自实现目录扫描/保存真相。Bevy scene把resolve/dependency登记与World spawn边界分开，并允许cached resolved scene参与组合。Zircon对应应让ProjectManager/asset pipeline拥有prepared generation与保存artifact，World转换只做可在worker执行的纯投影。

## 动态验收

1. current-source Cargo：legacy JSON roundtrip、SceneAsset全组件/reference/script binding、default-node normalize、invalid/dangling/error与LevelManager save/load。
2. nodes/assets/world bytes 1/1k/100k和1MiB/1GiB，unchanged/1% change/burst/shutdown记录World clone bytes、NodeRecord visits、asset resolves、JSON bytes、main/worker wall、writes、peak RSS与p95。
3. PERF-MVP-462要求`save_project_to_path` World clone bytes=0、normalize entity-id Vec=0、builtin locator parse≤4/process；PERF-MVP-453完成后main-thread filesystem/pretty serialize=0、same generation projection/write≤1、atomic publish/crash recovery通过。

动态Cargo、规模counter与F4保存产品trace未完成，因此本目录继续保留在`pending.md`，不进入`review.md`。
