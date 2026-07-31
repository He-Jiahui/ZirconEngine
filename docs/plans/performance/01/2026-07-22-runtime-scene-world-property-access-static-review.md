---
related_code:
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/core/framework/scene/entity_path.rs
  - zircon_runtime/src/animation/sequence
  - zircon_plugins/animation/runtime/src/sequence
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_plugins/04-animation.md
reference_sources:
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/godot/core/object/object.cpp
  - dev/godot/core/object/class_db.cpp
tests:
  - zircon_runtime/src/scene/tests/property_paths/read_paths.rs
  - zircon_runtime/src/scene/tests/property_paths/runtime_mutation.rs
  - zircon_runtime/src/scene/tests/property_paths/write_validation.rs
  - current-source Windows zircon_runtime property-path tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene world property access逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/world/property_access/**`当前源 **9/9** 个Rust文件、**2,883** 行、**2** 个就地tests已逐文件阅读，并追到`ComponentPropertyPath` owner、Runtime/Plugins04 animation sequence每track apply/target fallback和property-path测试。范围包含path resolution、fixed/dynamic read枚举、physics/collider shape entry projection、全部write dispatch与value conversion。

## 已直接修复

`get_entity_by_path`原先对每个候选entity的每个ancestor调用`path_segment_for_entity`，构造临时String后只用于比较；animation绑定target fallback会把分配放大到bindings×candidates×depth。现用borrowed name/target直接比较普通、重复名`#entity`与空名`Entity{id}`格式，实体十进制ID用无分配canonical parser校验，保留重复兄弟路径语义。源码守卫先RED后GREEN，已有duplicate sibling行为夹具继续作为语义门，scoped rustfmt/diff通过；本轮记PERF-MVP-460。

current-source受管Cargo test lane被`runtime10-runtime03-animation-frame-demand-producer-20260722`精确预约，本轮未启动Rust编译/测试，也未运行raw Cargo。

## 仍待PERF-MVP-329的P0根因

- `get_entity_by_path`仍枚举全部entities；每个ancestor为了判定同名兄弟又扫描全部entities，最坏接近O(N²×depth)。Runtime与Plugins04 animation target/apply在未命中cached entity时逐binding调用。
- fixed `property()`没有直接field dispatch，而是从Name开始调用`visit_property_entries`枚举schema，命中前会 eagerly 构造每个`ScenePropertyValue`；后置animation/physics/compound collider字段会产生resource/enum/path/parameter clone与递归格式化。动态field随后再走第二条查找。
- 每次`set_property`都把component和每个segment规范化为新String，并构造`Vec<String>`后进入约600行字符串match；animation sequence按track/frame调用，稳定clip仍重复分配/解析。
- `property_entries`用于Inspector全量枚举时先做capacity-hint完整component/dynamic scan，再做实际projection；应与PERF-MVP-456 inspection generation共享artifact，而不是另建永久cache。

Runtime08按PERF-MVP-329建立interned PathId/compiled accessor：动画clip/import或编辑命令边界一次解析entity/component/field/axis/index，按world/schema generation发布dense target/accessor；frame apply只验证generation后O(1) typed read/write。失败/未知路径必须在compile或generation rebind产生同等typed error，不保留字符串慢路作为稳定双权威。

## 参考引擎对照

Bevy clip以`AnimationTargetId`为HashMap key定位target curves，运行系统按target component直接取curves，不在每track帧内重新遍历名字路径。Godot把property名表示为`StringName`，`Object::get`进入`ClassDB::get_property`后通过`property_setget.getptr(p_property)`直接取得getter/index；property-list枚举是单独API，不作为单字段get的实现。Zircon应同样分离Inspector枚举与单字段热访问，并用generation显式处理场景/插件reload失效。

## 动态验收

1. current-source Cargo：duplicate/empty/deep entity path、fixed/dynamic/physics/compound read-write、invalid/nonfinite/read-only error、animation target fallback与generation mutation。
2. entities/depth/siblings/bindings/tracks为1/100/10k，stable/reparent/rename/schema reload记录entity/sibling/property-entry visits、normalized/path/value clone bytes、alloc、rebind与p95。
3. PERF-MVP-460要求segment compare临时String=0；PERF-MVP-329完成后stable binding entity/path resolve≤1/generation、per-frame normalized String/Vec=0、single field read entry visits≤1、Inspector stable full projection=0。

动态Cargo、规模counter和F2/F4产品trace未完成，因此本批继续保留在`pending.md`，不进入`review.md`。
