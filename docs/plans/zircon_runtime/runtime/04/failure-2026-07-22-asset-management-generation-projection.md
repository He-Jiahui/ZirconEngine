---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-management-generation-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/resource_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_streamer_accessors
  - zircon_runtime/src/asset/assets/scene/asset.rs
  - zircon_runtime/src/asset/assets/scene/management.rs
tests:
  - cargo test -p zircon_runtime --lib asset::tests::pipeline --locked --jobs 1 -- --nocapture --test-threads=1
  - stable polling, one-percent delta, query page, selected detail and scene entity matrices
---

# Runtime04：asset management generation投影缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset pipeline逐Rust文件性能审查，PERF-MVP-500
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：compact rows、kind/source/issue索引与summary必须随registry/resource generation原子发布，不能由Editor每帧重扫或维护第二套资产权威。
- 生命周期键：`asset-management-generation-projection`

## 失败现象与复现证据

kind查询每次全registry scan+sort；完整record sets为多类资产重复scan并深clonepayload。scene record和scene entity record分别加载/投影同一scene；overview、family summary、status与issue只需聚合却先物化全部详情。resource list排序在比较器中反复`primary_locator.to_string()`，让稳定Editor轮询持续产生全量clone/sort/String分配。

PERF-MVP-519已把scene overview、scene summary和entity summary的17/18次row扫描收敛为单遍，并删除entity list先建scene aggregate再clone rows的中转；但每entity overview仍clone完整direct-reference Vec只取数量，scene record仍内嵌全部entity rows，稳定consumer仍可重复构建/复制宽投影。PERF-MVP-520要求这些rows、counts与reference indices归入同一scene generation。

## 最低共享层根因

registry/resource generation只发布底层entry与payload，没有面向管理查询的compact rows、稳定ordered views、delta索引、summary counter和selected-detail边界。

## 架构修复验收

- generation发布时增量维护compact management rows、kind/source/issue/stability索引和summary counters。
- Editor09缓存`generation + query + page`，stable 60Hz不重建全量record sets；详情仅按visible/selected id懒取。
- scene summary/detail/entities共享一次generation parse/load，禁止同请求重复完整scene projection。
- scene/entity rows与aggregate counters随scene generation一次发布；reference count读取compact index，不构造AssetReference Vec，entity page不复制全scene rows。
- resource list使用借用stable order key/locator直接比较，不在sort comparator分配String。
- assets/scenes/entities 1/1k/100k、stable/1% change/page记录scans、sort/key alloc、deep-clone bytes和p95：stable build/sort/clone=0，changed近delta+page。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止Editor私建另一套全registry索引或定时全量cache refresh。
- 禁止把深clone推迟到另一个helper但仍为summary/list创建全部详情。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

2026-07-30 retained-host startup caller补充：`sync_asset_workspace`首帧同步调用`ResourceManager::list_resources()`；当前实现克隆registry全部`ResourceRecord`并以`primary_locator.to_string()`排序，随后把完整Vec交给runtime state。即使catalog已同步，这一步仍独立产生全表clone、排序和locator String，启动成本随N增长。Runtime04应把compact rows与stable ordered view随resource generation发布，retained host只传generation/Arc或MVP visible page；不得在Editor缓存第二份registry。验收增加startup/stable调用的registry scans、record deep-clone bytes、sort/key String bytes：warm unchanged必须为0，changed近delta+visible page。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-startup-current-review.md`；无动态pass声明。

2026-07-30 retained-host tick补充：任意非空resource event batch都会再次调用完整`list_resources()`，随后`EditorState::sync_resources`把每条locator重新格式化为String并从零构建map，同时无差别标记render/presentation dirty。现有256项/600us drain slice不约束这段O(N log N)+O(N)消费；持续resource backlog可每tick重建。generation projection必须直接发布共享stable ordered rows/map与affected-domain invalidation，stable batch全表clone/sort/map build=0；记录batch apply wall、registry scans、locator bytes、map entries和不相关render invalidation。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-assets-current-review.md`；无动态pass声明。
