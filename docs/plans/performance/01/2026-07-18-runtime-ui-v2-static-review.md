---
related_code:
  - zircon_runtime/src/ui/v2
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/v2_asset/performance_guards.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
reference_sources:
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetHeap.h
  - dev/slint/internal/interpreter/dynamic_item_tree.rs
tests:
  - runtime style children-clone source-level RED to GREEN guard passed
  - SurfaceTree owned slot-map source-level RED to GREEN guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows v2_asset tests pending behind shared Cargo FIFO
  - load/compile/style scale counters and MVP product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI v2逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已逐文件完整阅读`zircon_runtime/src/ui/v2`生产文件16/16：cache/compiler/component_instancer/file_cache/loader/module/style及其runtime_state/tokens、surface_builder、surface_tree的interaction/layout/node/parse/slot/module。目录审查前无外部脏文件；连同前批，`ui`累计生产文件187/783。调用追踪覆盖`.zui`文件加载、compiled document、surface构建、属性/伪状态更新与MVP editor资产/preview入口。

## PERF-MVP-271/272：文件缓存不是稳定零I/O

内存命中前仍canonicalize/stat显式和全部传递source，persistent命中再重复；mtime属于map key，修改后旧entry不会被同identity替换。asset-id miss会递归排序资产目录并parse每个`.zui`建立临时id索引。persistent写入又深clone root/compiled/documents，并重开重parse所有source恢复aliases；传递style合并还有整document/token/style clone。

EditorUI05需建立Bevy `AssetInfos`式canonical path/index、dependency/dependent和watch generation，稳定load零filesystem call、单叶修改只失效dependents；首次parse必须同时产出可序列化alias/import index，落盘不再parse源码或深clone整artifact。

## PERF-MVP-273/274：实例化与编译重复全图投影

instancer同时clone输入document和输出skeleton，每node clone definition；每component instance重复验证prototype graph、线性扫imports，并clone完整component stack/patch/slot context。compiler又对root及每个component root各做fresh reachable DFS，重叠子树可O(C×N)，arena/component graph/surface tree/runtime baseline多层复制相同payload。

EditorUI05需让prototype DAG/import map每generation验证一次，并用Arc context+handle/range驱动实例任务；canonical compiled arena一次遍历建立root/component/source/control/slot索引，surface和preview只持artifact handle与必要可变状态。Slint dynamic item tree把description放在`Rc`并让实例持offset/index，是可参考的compiled-description/instance ownership分界。

## PERF-MVP-275：高频伪状态扫描全部规则与子树

static resolve对每node扫描全部rules；runtime hover/focus/press从指定root遍历整子树、逐node扫描全部pseudo rules，重建attributes/overrides/tokens三张map。selector path还clone component/id/classes并以`Vec<String>`分配、查重、排序状态alias。本轮用源码RED→GREEN删除遍历中每节点整份children clone。

EditorUI04需编译selector候选索引、interned state bitset、rule→affected-node关系和computed-style generation，单叶状态变化只生成受影响属性delta。UE Slate用带contained flag的unique invalidation heap处理dirty widget，并只在desired size变化时向parent传播layout invalidation；这里应采用同样的精确dirty work思路，而不是事件触发整子树restyle。

## PERF-MVP-276：SurfaceTree重复解析已编译布局

Arena→UiTree仍拼接props/state/layout/resolved TOML map，再逐node解析layout/slot/interaction；单源/双源layout都会clone table，path按节点format。本轮用源码RED→GREEN把BuildFrame已经拥有的slot attributes直接move进metadata，删除调用点和metadata处两次map clone。

EditorUI02需让compiled arena保存validated typed layout/slot/input contract和稳定interned identity，surface build只投影紧凑DTO，不重parse TOML。该变更须与PERF-MVP-260/261的persistent layout tree和slot索引共同验收，避免只把parse成本移动到另一轮全树构建。

## 无新增独立热点的文件

`cache.rs`仅为小型prototype store API，`loader.rs`只负责字符串/文件解析入口，`parse.rs`为常数级字段helper；它们的主要成本均由file-cache调用频率、compiled ownership或per-node surface投影触发，已并入PERF-MVP-271/272/276，不按helper重复立项。interaction的descriptor registry由OnceLock单次构建，单node event scan随绑定数线性，当前不构成独立热点。

## 责任计划与验收

EditorUI05收到file-cache、persistent artifact、instancing/compile三份failure，EditorUI04收到runtime style一份，EditorUI02收到typed surface layout一份。以1/100/10k nodes、1/100/1k rules、1/100/1k source/component graph记录filesystem calls、parse/DFS/rule probes、map/String/AST clone bytes、visited/changed nodes、caller/background CPU与peak RSS；current-source Cargo、MVP `.zui` load/preview/hot-reload/hover产品trace及像素对拍完成前，16/16仍留pending。
