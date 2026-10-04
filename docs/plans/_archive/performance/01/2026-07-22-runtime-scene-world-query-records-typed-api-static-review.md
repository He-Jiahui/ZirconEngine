---
related_code:
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
  - dev/bevy/crates/bevy_ecs/src/query/iter.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure/cache_rebuild.rs
  - current-source Windows zircon_runtime ECS tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene world query/records/typed API逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/world/{query.rs,records.rs,typed_api.rs,typed_api/fixed_components.rs}`当前源 **4/4** 个Rust文件、**1,595** 行已逐文件阅读；范围包含QueryState所需的world/archetype/location桥接、NodeRecord事务投影、typed bundle/component CRUD、27类固定组件适配与反序列化presence rebuild。

## 已直接修复

`spawn_empty_at`原先在只新增一个显式ID空实体后调用`refresh_stable_entity_locations()`，重新计算全世界component signatures和archetype rows。现改为`refresh_entity_archetype(entity)`，只发布新实体的EMPTY archetype row，query-cache、derived-state和world generation边沿保持不变。行为与源码守卫先RED后GREEN，rustfmt/diff通过，归PERF-MVP-463。

## 固定组件双存储与批量恢复

固定组件当前同时存在于World的27张专用`HashMap<EntityId,T>`和generic `ComponentStorage`：insert把同一组件clone进专用map后再写storage；get/get_mut/remove/is-fixed依赖长TypeId/downcast链；`rebuild_fixed_component_presence_for_entity`又把每类存在值逐项clone到storage，并在组件间产生多次中间archetype move。全量deserialize最后仍重建archetype index，导致中间迁移全部成为浪费。

PERF-MVP-464与Runtime08 failure要求硬切单一storage authority：typed fast path、serde/project I/O与reflection通过generated/static adapter table访问同一row；restore先计算每entity最终signature并一次发布，不得增加第三份presence truth或保留旧双写兼容路径。

## QueryState cache miss仍扫描全世界

`matching_query_archetypes`已利用component→archetype索引，且`matching_query_archetype_entity_count`给出精确reserve；但`visit_entity_locations_matching_archetypes`随后仍遍历全部`World.entities`，每entity查询stable location并对matched ids做binary search，因此稀疏query cache miss仍为O(N log K)。现有结构测试明确锁定stable world entity order；archetype record又使用swap-remove，所以直接按archetype分组遍历会改变query结果顺序，本轮没有冒险改写。

PERF-MVP-466要求Runtime08增加唯一stable query-order identity/index，使matched rows可按稳定顺序流式合并或通过dense-order bitset访问，cache rebuild工作接近matched rows而非全世界。不得用每次query collect+sort全量候选来转移成本。

## NodeRecord批事务全World复制

`insert_node_records`为了失败原子性先`self.clone()`完整World，再逐record clone并走普通insert，最后整体替换authority。该语义正确，但undo/import小批次对大世界也复制所有component maps、registries和derived state，并产生重复中间signature/archetype更新。PERF-MVP-467交接Runtime08与Editor03：预验证最终identity/schema/reference和signatures，以affected row/component undo delta或copy-on-write pages实现事务，单次commit world generation；失败只丢delta，不复制/替换完整World。

## 参考引擎对照

Bevy ECS的`Table`让entity rows与component columns由一个storage owner管理，swap-remove返回被移动entity供location修正；Zircon应采用这一“单一row authority + 显式location修正”原则，而不是复制其unsafe API。Bevy query iterator直接消费matched storage rows；Zircon因公开稳定顺序合同需额外维护稳定order identity，不能简单照搬archetype顺序。

## 动态验收

1. current-source Cargo：typed insert/get/get_mut/remove、bundle、serialize roundtrip、archetype move/despawn、cached/uncached/mutable/combinations query稳定顺序、batch success/failure/undo。
2. entities/components 1/1k/100k、query match 0.1%/1%/100%、batch 1/1%/100%记录component clone bytes、TypeId branches、archetype moves/rebuilds、unmatched entity visits、World/NodeRecord clone bytes与p95。
3. PERF-MVP-463当前要求explicit spawn full rebuild=0；464/466/467最终分别要求单组件authority=1、sparse query不扫unmatched entities、batch full World clone=0。

受管Cargo当前仍受共享预约阻塞，规模counter与F2/F4产品trace未完成，因此本目录继续保留在`pending.md`，不进入`review.md`。
