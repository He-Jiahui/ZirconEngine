---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: template-projection-deep-copy-and-cache-generation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/host_value_toml.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_ui_asset_conversion
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes
reference_sources:
  - dev/slint/internal/core/properties.rs
  - dev/slint/internal/core/items/component_container.rs
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_format_binary.cpp
tests:
  - template generation cache hit/miss/build-count matrix
  - 1k editor event projection build-count and clone-byte stress
  - partial builtin load, hot reload and alias graph generation parity
  - lock-hold/I-O trace for concurrent template registrations
---

# EditorUI05：模板投影深拷贝与缓存代际缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/ui/template_runtime` 当前工作树 44/44 Rust 文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：compiled `.zui` document、prototype/import graph、file cache、热重载失效与 immutable asset generation 由 EditorUI05 持有；EditorUI08 只能按 dirty generation 消费，不能在 workbench host 旁建立第二份模板缓存。

## 失败现象与复现证据

`build_session` 为每次模板路径创建 key 时 canonicalize + metadata，并用 `{path, modified, len}` 向 process-global cache 查询；`runtime_host` 在持有该 cache mutex 时执行 `load_store(paths)` 的读盘、解析和编译。旧 key 没有 generation 回收，cache hit 仍深 clone compiled/document；import 注册又把同一 document 复制到 base reference、每个 component alias 和 root alias。

pane 侧把 typed performance/plugin/export payload 完整转换为 TOML tables/arrays；随后 projection 构建完整 retained tree，host-model 再 clone node/binding/attribute data，adapter 再映射成最终 host values。`build_shared_surface` 和 host-model API 没有 document/pane/theme/size generation cache，因此上游每-event reflection 会把这一链重复执行。审查已直接消除 adapter 的双重 property mapping 与 binding-index 整行 clone，但完整树的多代物化仍在。

Retained presentation下游再次放大：每个可见 workbench node都把完整 `BTreeMap<String, RetainedUiHostValue>`递归深转为另一份 TOML map/array/table，只为 style、command-palette/notification options和typed canvas读取；大量 scalar property随后又按 key重复查询/格式化。节点祖先索引与明显 row clone已在性能切片直接修复，但 property representation与generation owner不能由 presentation converter私设第二份 cache。

Pane conversion 211文件审查补充了同一 representation 问题：button style只为补3个alias就无条件clone整张attribute map；CommandPalette/NotificationCenter同一node分别为plain options与structured options重复解析完整TOML entry列表；UI asset detail先clone prop/state rows，再按4个section重复扫描与移动node vector。性能切片已完成PERF-MVP-142、144、145的局部确定修复：specialized entry一次parse、无alias map借用、prop/state rows借用；property view和changed-section generation仍必须由本计划拥有。

Host-contract data 77文件审查补充PERF-MVP-148：`TemplatePaneNodeData`把约160个common、text/image、collection、world、timeline、heatmap、drag/input/action字段放进每一个node。Slint以item-specific结构存属性，Bevy以common `Node`加稀疏widget component组合；本计划应发布compact common header与component-family typed payload，不能继续让普通text/button按最大node形状支付初始化、clone与cache footprint。

Slint 用 `PropertyTracker::evaluate_if_dirty` 控制 component factory/属性重算，并在 repeater 中定点处理 row change；Godot loader 以 path `ResourceCache` 复用资源并记录 modified time。这些参考共同支持 generation-owned immutable document/projection，而不是在每个 consumer 加局部 clone cache。

## 最低共享层根因

模板文件缓存、prototype store、compiled document、pane payload、surface 和 retained host model 没有统一的 generation/identity/consumer cursor。缓存值以 owned deep-clone DTO 暴露，失效只靠 path metadata key 追加新条目；全局 mutex 同时承担索引一致性与慢 I/O/compile，导致内存增长、重复构建和跨线程串行化。

## 架构修复验收

- 以 canonical asset id + content/compiler/schema/import generation 建立 compiled document owner；命中返回 `Arc`/稳定句柄，旧代按活动引用与明确预算回收。
- cache mutex 只保护快速索引与 generation publish；stat/read/hash/parse/compile 在锁外 worker 执行，提交前校验 source/import generation，重复请求合并为一个 in-flight build。
- prototype/import graph 每个 compiled document 只存一份；base/component/root alias 映射到 stable handle，禁止复制完整 document。
- typed pane state 保持 typed immutable snapshot/delta；只在真正 dirty 的 node/row 边界投影，禁止 typed → TOML → projection → host-model 多轮全树 materialize。
- style/options/canvas读取typed property view或编译期schema projection；未变 node不深转整张 TOML map，changed node只转换实际消费字段并记录fallback原因。
- command/notification typed entry每node generation只解析一次，同时生成plain/structured rows；UI asset detail以control index与section generation只更新changed section，未变prop/state rows clone/build=0。
- common node header保持identity/frame/clip/最小state；text/image/input/collection/world/timeline/heatmap/drag等冷字段进入按component family拥有的typed payload，10k普通node不分配无关payload。
- surface/host projection cache key 至少包含 document、pane、theme/layout 与 viewport-size generation；同一帧每 domain build 不超过一次，未变 generation build=0。
- 1/100/10k node、1/100/1k alias 与 1k event fixture 记录 build count、clone bytes、peak RSS、cache entries/evictions、mutex wait/hold 和交互 p95；hot reload、partial builtin load、route/order/diagnostic bytes 保持等价。

## 禁止临时方案

- 不得在 workbench、pane 或 retained adapter 各自添加第二份无失效 cache。
- 不得只把全量 clone 移到后台线程；提交仍须是 generation-checked 的增量或共享 snapshot。
- 不得依赖 path/mtime/len 永久追加旧 cache entry，或在全局锁内读盘、解析、编译。
- 不得保留 canonical V2 document 与 legacy projection 双真源来换取缓存命中。

## 修复结果与回传

Open state: `待 EditorUI05 实现 compiled-document/prototype generation owner、锁外 build、stable alias handle 与 typed incremental projection，并向性能计划回传 clone/build/cache/lock 产品证据`。
