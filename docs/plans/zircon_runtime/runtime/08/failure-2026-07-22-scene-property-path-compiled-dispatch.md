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
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
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

Open state: `implementation_complete / managed_validation_pending`; no Cargo pass is claimed.

### 2026-08-11 current-source research and design reconciliation

- 主参考 Unreal：`dev/UnrealEngine/Engine/Source/Runtime/PropertyPath/Public/PropertyPathHelpers.h` 的 `FPropertyPathSegment`/`FCachedPropertyPath` 保留可序列化文本段，但在 resolve 后缓存 `UStruct` 与 `FFieldVariant`，结构不变时不重复 field lookup。Zircon 采用同一 compile-boundary 原则，但以 Rust typed writer enum 和显式 scene/schema generation 代替可失效的裸 field pointer。
- Rust 主实现参考 Bevy：`dev/bevy/crates/bevy_animation/src/lib.rs` 在导入边界由 full path 生成稳定 `AnimationTargetId`；`animation_curves.rs` 的 `AnimatableProperty`/`AnimatedField` 将写入编译为 typed accessor，并以预哈希 `(TypeId, reflected-field-index)` 作为 evaluator identity。Zircon 对应发布 `PathId`、`ComponentFieldId` 与 typed writer variant，稳定帧不再携带字段名查找。
- 交叉参考：Godot `dev/godot/scene/animation/animation_mixer.cpp` 在 `_update_caches` 解析 `NodePath`，以 track unique id 建 `TrackCache` 和 track-number projection，稳定 blend/apply 直接消费 cache；Fyrox `dev/Fyrox/fyrox-animation/src/value.rs` 明确将 position/scale/rotation 作为快速 typed binding，并把通用 reflection property 标为慢路径。两者共同支持“编辑/导入时 resolve，帧循环只消费 compiled binding”的裁决。

### 已完成实现

- `scene/world/compiled_binding` 已成为唯一 runtime binding owner：`SceneBindingGenerations` 统一 intern canonical entity/property identity，`CompiledScenePropertyTarget` 绑定 resolved entity、root generation、`PathId` 和 `ComponentFieldId`；name/reparent/remove、same-ID reuse、replacement World 与 dynamic schema catalog 变化均显式使目标 stale。
- `CompiledScenePropertyWriter` 已将 Transform、MeshRenderer、animation runtime、Camera、Light 和 dynamic component field 编译为 typed variant。`read_compiled_scene_property`/`write_compiled_scene_property` 的稳定路径不调用 `get_entity_by_path`、`set_property` 或 property-entry visitor；raw `ComponentPropertyPath` 只保留 typed error 所需的诊断正文。
- `animation/sequence/compiled.rs` 在 asset/import 边界一次解析 entity 与字段并保存 writer；`zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs` 仅在 asset revision 或 world/schema generation 失效时重编译，帧 apply 直接消费 compiled writer。不存在稳定帧字符串 fallback。
- single-field `World::property` 已与 Inspector 全量 entries 分离，直接分派且最多记录一次 requested-entry visit。现有 10k stable-access fixture 断言 `path_lookup_requests = 0`、`canonicalization_bytes = 0`、`property_entry_visits = 0`，并覆盖 stale rejection、dynamic schema generation、clone/deserialize diagnostics reset。

### 待完成证据

- 仍须执行 frontmatter 声明的受管 `property_paths` focused gate、animation plugin consumer gate 与 1/100/10k scale fixture，并记录 raw terminal count、allocation/rebind/p95 证据。取得 terminal GREEN 前不得把本文件改为 `fixed-*`，也不得声称 Cargo/性能验收通过。
