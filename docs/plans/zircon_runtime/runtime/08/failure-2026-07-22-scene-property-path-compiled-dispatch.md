---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: scene-property-path-compiled-dispatch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/scene/entity_path.rs
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/animation/sequence
  - zircon_plugins/animation/runtime/src/sequence
tests:
  - cargo test -p zircon_runtime --lib property_paths --locked --jobs 1 -- --nocapture --test-threads=1
  - animation bindings and scene property scale fixtures
---

# Runtime08：scene property path编译分派交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene world property_access 9/9逐Rust文件性能审查，PERF-MVP-329
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08拥有Entity/Component identity、property path和ECS数据边界；Runtime/Plugins04 animation及Editor05 Inspector共同消费。
- 生命周期键：`scene-property-path-compiled-dispatch`

## 失败现象与复现证据

animation target fallback逐binding调用`get_entity_by_path`，当前全entity枚举并在每ancestor为同名检测再全扫，最坏O(N²×depth)。单字段`property()`通过完整property-entry visitor线性枚举并在命中前构造值；`set_property()`每调用为component和全部segments新建normalized String/Vec后字符串分派。稳定clip/scene因此仍按tracks×frames重复解析、分配和组件探测。

本轮只删除path segment比较的临时String，没有把全表路径解析、字段枚举和写入规范化冒充解决。

## 最低共享层根因

`EntityPath`/`ComponentPropertyPath`是owned文本DTO，没有scene/schema generation绑定的interned identity、dense entity target或typed field accessor。Inspector enumeration与single-field read共用同一entry projection visitor，compile/edit boundary和frame apply没有分层。

## 架构修复验收

- Runtime08发布interned PathId/ComponentFieldId和generation-owned compiled accessor，包含dense entity/component/field/axis/index信息；raw文本仅保留serde/display/error owner。
- animation clip/import或编辑命令边界一次compile，frame apply只验证world/schema generation并走typed O(1) read/write；reparent/rename/schema reload只增量rebind受影响路径。
- Inspector全量property entries按selection/component generation与PERF-MVP-456共享artifact；single-field API不得枚举/构造其他字段值。
- entities/depth/bindings/tracks 1/100/10k记录entity/sibling/entry visits、normalize/path/value clone bytes、alloc/rebind/p95：stable resolve≤1/generation、per-frame normalized allocations=0、single field entry visits≤1。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止同时保留compiled accessor和每帧字符串fallback两套稳定authority；generation mismatch可显式rebind或返回typed stale error。
- 禁止给每个consumer各建Path interner；identity与schema generation必须由Runtime08单一owner发布。
- 禁止缓存owned ScenePropertyValue跨generation而不区分metadata/value变化。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
