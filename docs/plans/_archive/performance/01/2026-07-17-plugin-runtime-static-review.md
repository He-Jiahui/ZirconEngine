---
title: Plugin runtime static performance review
date: 2026-07-17
status: static-reviewed-dynamic-pending
related_code:
  - zircon_runtime/src/plugin
  - zircon_runtime/src/scene/runtime_extension
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/plugin_sdk
plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# 插件 runtime 静态性能审查

## 已读范围与结论

- `zircon_runtime/src/plugin/native_plugin_loader` 当前 53 个 Rust 文件已完成 53/53 静态通读：43 个 production files 与 10 个 test files 均已逐文件阅读。当前 Windows `native_plugin_loader` Cargo gate 与规模/并发性能基准仍在进行或 pending，因此目录继续留在 `pending.md`，不提前写入 `review.md`。
- `zircon_runtime/src/plugin/bridge` 当前 4 个 Rust 文件已完成 4/4 静态通读（import、weak、table、reports）；双锁稳态调用与 ABI snapshot 误用分别进入 PERF-042/045，其余 reload/diagnostics 路径继续等待并发 benchmark。
- `zircon_runtime/src/plugin/extension_registry` 由 `git ls-files` 枚举为当前 35 个 Rust 文件，已完成 35/35 静态通读，覆盖 access/apply、owner、typed storage、全部 register/validation 子文件、registry/revocation 与现有 tests。静态阅读完成不等于动态验收：当前源码 warm Cargo、注册规模/分配基准与产品 trace 未完成，因此目录继续留在 `pending.md`。unchanged bridge finalize 重建与 namespace split 临时分配已分别作为 PERF-046/047 直接修复。
- `zircon_runtime/src/plugin/package_manifest` 当前 15 个 Rust 文件已完成 15/15 静态通读；其结构/constructor/accessor 都是 manifest 构建边界，未发现逐帧可达代码。`package_id()`/`asset_roots_or_default()` 的 owned projection 目前只在项目同步/测试调用，和 load-report 重复 manifest clone 一并由规模 trace 判断，不单凭返回 `String/Vec` 改 API。
- `zircon_runtime/src/plugin/runtime_profile` 当前 5 个 Rust 文件已完成 5/5 静态通读。availability assembly 重建 builtin catalog、provider Vec 线性去重后再 clone 成 HashSet、selection Vec 线性合并的问题进入 PERF-048；当前调用点位于 bootstrap/export assembly，未发现 frame/tick 可达，因此按 P2 启动规模问题处理。
- `zircon_runtime/src/plugin` 当前 9 个 root Rust 文件已完成 9/9 静态通读（module façade、bridge/runtime-profile/native exports、maturity/core profiles/capability/UI descriptor/error）。这些文件除 UI descriptor 的注册期 runtime projection 外主要是 DTO、enum 与 re-export，没有发现新的逐帧执行入口；UI descriptor conversion 的 String clone 由 UI component 安装规模测试覆盖，不在无调用频率证据时改为共享缓存。
- `zircon_runtime/src/plugin/export_build_plan/from_project_manifest` 的 3 个 child Rust 文件已完成 3/3 静态通读；`project_manifest_validation` 6 个 Rust 文件已完成 6/6。feature normalized-id、namespace segment Vec 与 formatted owner prefix 已作为 PERF-049/050 直接修复；跨 validator/sanitize/provider/profile 的重复线性扫描进入 PERF-051。export 根文件、materialize/platform-host/root 其余文件仍 pending。
- `zircon_runtime/src/plugin/export_build_plan/platform_host_files` 的 browser/mobile 2 个 Rust 文件已完成 2/2 静态通读。模板生成本身是 export-time；但生成产物的 browser pointermove、Android multi-pointer move 与 iOS touchesMoved 都逐事件同步跨 ABI，运行时高频输入问题进入 PERF-052。browser 文件中 early-return 后保留的旧模板是编译/维护债，不计产品 runtime 热点。
- `zircon_runtime/src/plugin/export_build_plan/materialize` 8 个 Rust 文件已完成 8/8 静态通读。ZIP 整文件读入 Vec 已作为 PERF-053 改为流式复制；per-package recursive lookup/manifest parse、preview/archive 重复枚举、unchanged generated/native 无条件串行覆盖与线性 export-row lookup 进入 PERF-054。
- `zircon_runtime/src/plugin/export_build_plan` 当前 39 个 Rust 文件已完成 39/39 静态通读（root 20、from-project child 3、validation 6、platform-host child 2、materialize 8）。`ExportValidateReport` 默认复制完整 generated contents 并由 CLI 再 JSON/文件/stdout 放大，进入 PERF-055；全目录仍需 warm Cargo、规模/RSS/I/O 基准与生成产物验证，继续 pending。
- `zircon_runtime/src/plugin/runtime_plugin` 已完成 287/287 个当前目标子组 Rust 文件的逐文件静态通读：此前 runtime-plugin/feature-report/root/descriptor/registration/module-validation/feature-validation 112/112，加上 `package_validation` 175/175。package namespace/semver segment Vec、provider identity String clone 与 owner-prefix String 已作为 PERF-057/059 直接修复；跨 capability/interface/module/root/contribution/status 的 Vec uniqueness/owned-capability repeated scan 进入 PERF-058。插件静态累计进度为 447/578，动态验收尚未满足。
- `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog` 87/87 与 `builtin_catalog` 44/44 Rust 文件已逐文件静态通读，至此本轮 `zircon_runtime/src/plugin` 范围累计 578/578 静态覆盖。catalog constructor 的逐插件全量 diagnostics/bridge-graph rebuild、completed manifest 二次 completion、predicate 临时 Vec 与 order membership 已作为 PERF-060 直接修复；feature definitions/completion/resolution/merge/lifecycle 的 generation projection 缺失进入 PERF-061。builtin catalog 主要是静态行与启动期 descriptor 构造；classification 的短 String 投影由 catalog generation build-count 基准判断，不误报成逐帧热点。

- 首方 runtime/editor catalog 与 SDK 的注册 builder 位于冷启动/注册期；当前产品 catalog 通过 `HashSet` 去重 enabled package，未发现直接逐帧扫描。
- `RuntimeExtensionRegistry::world_runtime_extension_plan` 在 world 安装期构造注册闭包，`WorldDriver` 随后缓存 schedule stage plan 与 scene-hook stage plan；这符合“注册期编排、逐帧走冻结计划”的目标。`WorldRuntimeExtensionPlan::try_merge` 会克隆完整 plan 和 key，但只应出现在安装/热重载边界，需用产品 trace 验证频率而不能按静态形态误判为帧热点。
- `NativePluginLiveHost` 的 command、broadcast、save、restore 与 play-mode 组合入口在取得 `loaded: Mutex<BTreeMap<...>>` 后，直接在 guard 存活期调用插件提供的 foreign ABI callback。单个慢插件会阻塞所有 descriptor、state 和 lifecycle 操作；若 callback 重入同一 live host，则非重入 mutex 可能自锁。广播还会把所有插件 callback 串行包在同一全局锁内。
- ABI payload 必须从插件 owned buffer 复制到 host `Vec<u8>` 后再调用插件 free callback；这是跨动态库所有权边界的必要复制，当前不作为简单删除项。需要通过 payload-size benchmark 决定是否另加 caller-provided buffer/streaming ABI。
- MVP profile 默认选择 Sound 与 Rendering。Rendering umbrella 的四个 default feature 是否真的进入最小产品帧图仍需当前源码 capture 证明；静态注册文件本身只在启动期生成 descriptor/executor registration。
- Sound 正在共享工作区迁移到 Kira。当前 `KiraEngine::sync_graph` 先调用 `diff_graphs`；后者对 next graph 调用一次被丢弃的 `compile_graph(after)?`，随后又生成 `compiled_after`，回到 `sync_graph` 的增量路径后再编译一次。多数 track/effect/send mutation caller 还先调用 `validate_graph`，所以一次编辑可能在 sound-state 全局 mutex 内重复验证/构图四遍。此项已路由 Plugins 02，未碰正在迁移的 sound 源文件。
- Native host bridge callback 查找会在全局 context mutex 内取得 handle 后 clone 整个 `NativeHostBridgeCallContext`，再锁外调用 method。`FrozenBridgeTable` clone 只是 Arc 增计数，但 context 内的 method `BTreeMap` 会逐 bridge call 深复制。正确最小修复是 registry 存 `Arc<NativeHostBridgeCallContext>`，锁内 clone Arc，继续在锁外执行 method；不能把 method 调用挪回锁内。
- `NativePluginLoader::discover` 递归 `read_dir` 整棵 root，没有显式 symlink/junction/depth policy 或 unchanged generation cache。产品已经有显式 `plugins/native_plugins.toml` 路径，应优先使用；editor discovery/status/hot refresh 需要 watcher/generation cache 和 canonical cycle policy，而不是每次递归重扫。
- `load_candidates_for_module_kinds` 原先为保留返回报告而深 clone 全部 `report.discovered`。该加载期冗余复制已用 `mem::take` 消除：候选 Vec 被临时移出，加载逻辑只借用候选，结束后原样恢复到报告；源码守卫和 rustfmt 已过，Cargo loader test 仍 pending。
- Native registration manifest replay 对每个 system 分别调用 `runtime_bridge_method_slot` 与 `runtime_bridge_call_scope_from_installed_bindings`。前者每次 clone 整份 package manifest 后线性查 method，后者再次 clone manifest 和全部 bindings，并为每个 system 建完整 call scope；应改成每插件一次解析/索引/共享 owner，不能仅在单个 helper 内微调 clone。
- `load_reported_plugins_result` 连续生成 runtime package 与 feature registration reports，两条路径各自重建/合并全部 `package_manifests`；随后每个 report 都重新全扫 diagnostics 与 loaded entry reports并排序去重，manifest Vec 合并又以 `contains` 做线性去重。需要一次 load projection，而不是分别 memoize 单个 getter。
- `diagnostic_mentions_plugin` 在上述重复扫描里原先每条 message `format!` 两个 needle；已换成无分配 prefix/boundary 扫描，并覆盖正文中间匹配与相似 ID 不误匹配的测试。
- Runtime export-root hot update 已经按值消费 discovery candidates，却原先为每个单插件 load report 再 clone candidate/package manifest；现改为保存 `plugin_id` 后直接 move candidate，保持逐插件 rollback 与结果排序。
- Registration manifest 的 system row 已有 `access: Vec<String>`，但 replay 未消费；所有回放 system 均注册为 `NativeDynamicAccess`，其 `SystemParamAccess` 是保守全 World 写者。于是互不相交的 native systems 也与全部 ECS world access 冲突，worker 调度不能并行；修复必须同时定义稳定 access id、校验和 thread affinity，不能直接把 foreign callback 标成无冲突。
- Typed bridge 稳态调用仍是双锁路径：`BridgeImport::call` 锁 `binding: Mutex<Option<WeakBridge<T>>>` 并 clone handle，`WeakBridge::provider_with_slot` 随后锁共享 cached provider mutex；即使 generation 稳定也要两次互斥获取。reload/unbind 是低频写，call 应消费按 generation 发布的 immutable/read-mostly snapshot。
- 通用 `SystemRegistrationBuilder` 与 `RuntimeSceneSystemRegistrationBuilder` 把传入的单个 `FnMut` 放入共享 `Arc<Mutex<_>>`；每次 run 都锁，且同 registration 构建到多个 World 的实例共享 callback 状态并跨 World 串行。native replay callback 实际无状态也走这条锁路径；应改成 per-instance factory + stateless shared callable 双契约。
- PERF-034 的 Arc context 已消除每 call method-map 深 clone，但 `bridge_context_for` 仍对进程级 `Mutex<BTreeMap<u64, NativeHostApiV3Context>>` 加锁；所有 plugin scope 和 calling threads 共用该 registry。需要 generational handle + concurrent-read context owner，scope drop 只撤销新 lookup，在途 Arc 保持有效。
- Native ABI bridge call 原先为判断 enabled 调 `interface_snapshot(slot)`，逐 call clone interface-id、读取完整 diagnostics并物化报告；现改为直接取得 `BridgeEntry` 并调用 `status()`，保留 provider/generation 正确性但不构造诊断快照。
- `RuntimeExtensionRegistry::finalize` 的 typed extension freeze 已幂等，但 bridge finalize 原先无条件重建 table并重绑 imports；asset/world/UI/module 多 consumer 会重复构建。现让已有 table 直接复用，export/revoke 的既有 invalidation 仍驱动下一次真正重建；行为测试用 disabled 状态证明 unchanged finalize 未换表。
- Event catalog、plugin option 与 scene-hook namespace validator 原先把每次 `value.split('.')` 收集为临时 `Vec<&str>`；event catalog 会按 namespace、event id 与 payload schema 放大这类短命分配。现以无分配 `contains('.')` 保持原有“至少两段”错误优先级，再直接流式遍历 `split`；三文件源码守卫已完成 RED→GREEN，合法/非法行为仍待 warm Cargo suite。
- Runtime profile availability 的 assembly helpers 每次重建 builtin descriptor catalog；`for_id` 也构造全部六个 builtin profiles 后取一个。registration provider ids 先在 Vec 中线性去重，随后又复制成 HashSet；manifest selections 同样用线性 find 合并 required。该问题需要 bootstrap generation 级共享 projection，且调用点只确认在启动/导出路径，不能描述成稳定帧热点。
- Export profile feature projection 的 required/missing/projection 搜索原先都在 nested `any` 中调用 `normalize_profile_feature_id`，为每次短 id 比较 `format!("{owner}.{feature}")`。现改为借用字符串：完整 id 直接比较，短 id 用 `strip_prefix(owner).strip_prefix('.')` 比较 suffix；源码守卫覆盖无 allocating helper，行为 test 覆盖短/完整/跨 owner/不匹配。
- Export project feature validator 的 namespace predicate 和 full diagnostics 原先各自收集 split segment Vec；full diagnostics 再格式化 owner prefix。现用 dot-presence/流式 segment flags 与 borrowed `strip_prefix` dot-boundary 检查，保持可同时报告“缺 dot/空段/非法段/owner 不匹配”的原顺序。
- Export validation 仍在 duplicate diagnostics 与 identity sanitize 中分别用 Vec 线性查重，profile diagnostics/projection 多次 nested search，external provider 每 feature 全扫 packages。该复杂度问题需要一次 generation projection，不能用更多局部 HashSet 形成多份 authority。
- Generated browser host 对每个 `pointermove` 同步调用 WASM export；Android host 对 ACTION_MOVE 的全部 pointers 逐一 JNI dispatch，iOS 对 moved touches 逐一 Swift→C dispatch。move/viewport metrics 可在 host 边界按 frame 合并，press/release/begin/end/cancel 不可丢；应复用 Runtime12 公共语义而不是平台模板各自 throttle。
- ZIP archive 原先对每个 native package entry `fs::read` 成完整 Vec 再 deflate，额外峰值内存与最大单文件同阶；现用 `File` 和 `std::io::copy` 直接流入当前 ZipWriter entry，保持 generated in-memory contents 路径不变。
- Native package materialize 对每个 package 独立递归扫描 plugin root并逐 manifest parse；materialize/preview/archive 重复该查找。generated/native 输出又不比较内容就串行覆盖。正确边界是 export generation 级 package/file inventory + 增量写与有界 I/O，而不是三个 consumer 各自 cache。
- Export validate report 把 generated `path/purpose/contents` 全部深 clone 到 summary，CLI 再序列化完整 JSON，可同时写文件并始终 stdout。默认报告应只携 metadata/digest，完整内容改显式 artifact；这是 public schema/consumer 迁移，不直接删除字段。
- Runtime feature namespace validator 原先为每个 feature id、capability id、module id 等 namespaced field 收集 `Vec<&str>`，identity validator 还格式化 `{owner}.` 前缀。现改为 `contains('.')` 保持“至少两段”的诊断优先级、流式 token 检查和 borrowed dot-boundary owner 判断；未改 diagnostic 文本与顺序。
- Package namespace validator 存在同型 segment Vec；semver validator 也为固定 MAJOR/MINOR/PATCH 三段收集 Vec。两者现分别改为 dot-presence + 流式 split，以及无分配 iterator 解构；semver 只有错误诊断路径才计算实际多余段数。
- Package validation 的 asset importer、capability/dependency/status、interface/method、module/system、root、contribution、feature provider 等 uniqueness state 普遍是 Vec，并在每行 `contains` 后追加；owned capability 还先收集 Vec、再由每条 status 线性查。该注册期规模复杂度必须用一次顺序保持 projection 统一治理，不能把几十个局部 Vec 各自换成独立 authority。
- Embedded feature provider 唯一键原先把 borrowed feature/provider id 各 `to_string()` 一次，只为存入本次 validation state；现直接保存 borrowed tuple。event catalog 与 module system owner 检查也改为 borrowed `strip_prefix` + dot boundary，避免 per-package/per-module prefix String。
- `RuntimePluginCatalog::from_plugins/from_descriptors` 原先按插件调用 public incremental `register`，而 `register` 每次 `rebuild_diagnostics`，后者复制当前所有 diagnostics 并从头构建 bridge dependency closure。构造 N 个插件会重复 N 次全量派生；现先生成全部 reports，再用 `from_registration_reports` 一次 rebuild，public 增量 mutation 语义不变。
- `runtime_extensions_for_project` 原先先 `complete_project_manifest`，再把 completed manifest 传给 public `feature_dependency_report`，后者再次 completion。现 report core 直接消费 completed manifest，去掉一次全量 selection/feature completion；feature definition map 在 completion/report 间仍重复，纳入 PERF-061。
- Catalog 的 `feature_manifest_for_selection` 每 query 重建整份 definition map；project selection defaults 是 registration×selection 互扫，feature completion 是 selection×definition 与 definition×selection 互扫，fixed-point resolution 反复扫描 pending 并用 `Vec::remove` 移位，available feature merge 再扫描 registration 与 manifest selection。需要 catalog generation 级有序 feature graph/index，不宜局部 cache。
- Feature owner-primary 与 target-support predicate 原先分别 collect dependency/runtime-module Vec 后只做 count/any；registration ordering 用 `ordered_registration_indices.contains` 两遍线性 membership。现均改为流式 iterator/bool seen，保持原始输出顺序。

## 参考引擎对照

- Bevy `dev/bevy/crates/bevy_app/src/plugin.rs` 把 `build/ready/finish/cleanup` 明确约束在应用配置与启动生命周期；运行期系统进入 schedule，不靠每帧重新遍历插件注册 builder。
- Bevy ECS 运行期并行来自注册阶段冻结的 component/resource read-write access；Zircon native manifest 虽有 access 文本却没有进入 `SystemParamAccess`，因此当前保守写者是安全 fallback，不是可接受的最终性能契约。
- Godot `dev/godot/core/extension/gdextension_manager.cpp` 的 `frame()` 遍历稳定的 `Ref<GDExtension>` 并直接调用 frame callback，没有把 callback 包在一个可见的 manager 全局 mutex 中。这里的可迁移原则是“锁内快照稳定句柄，锁外执行不受信任/耗时 callback”，而不是复制其容器实现。

## 待动态验证

1. 1/8/32 个 native plugin 的 command broadcast，记录 callback wall time、live-host lock wait、p50/p95/p99 和分配。
2. 慢 callback 与 callback 重入 fixture；并发 descriptor/query、hot reload、unload 不得死锁，卸载不得早于所有快照调用完成。
3. 0 B、1 KiB、1 MiB payload 的 invoke/save 往返，拆分插件执行、host copy 与 free callback 成本。
4. 当前源码 MVP 启动、enter/exit play mode 各重复三次，确认注册/descriptor plan 没有意外进入逐帧路径。
5. 10/100/1000 track mixer mutation 记录 graph validate/compile 次数、wall time、分配与 sound-state mutex 持有时间。
6. 1/100 bridge methods 的 1M ABI calls，记录 context registry lock wait、method-map clone/allocation 与 callback p95。
7. 1k/10k plugin-tree unchanged refresh、单 manifest 修改和 symlink/junction cycle fixture，记录 enumerate/stat/read/parse count。
8. 1/100/1000 registration systems × 1/100 bridge methods，记录 package-manifest clone、binding clone、scope/context build count 与 replay wall time。
9. 1/100/1000 package、每 package 0/10 features 与 10k diagnostics 的 load-report projection，记录 manifest merge、diagnostic scan、sort 与分配次数。
10. 两个 disjoint、两个冲突和一个 main-thread-only native system fixture，记录 schedule conflict graph、worker thread id、overlap 与确定性顺序。
11. typed bridge 1/16 calling threads × 1M stable calls，并发 reload/disable；记录 mutex wait、throughput、generation miss 与 provider lifetime。
12. plugin system 单/双 World 与 stateful/stateless factory fixture；记录 callback mutex acquire、跨 World overlap、状态隔离和 reload generation lifetime。
13. native host context registry 1/16 calling threads × 1M calls、并发 scope drop/reload/stale handle；记录 registry mutex wait、lookup p95 和 in-flight lifetime。
14. native bridge 1M enabled/disabled/absent calls，记录 interface snapshot/string allocation、provider status lock 与 diagnostics counters。
15. 顺序执行 asset/world/UI/module apply 与重复 finalize，记录 bridge build/bind 次数；再执行 export/revoke 验证恰一次 invalidation rebuild。
16. 1/100/1000 条 event catalog 与 option/scene-hook 注册，记录 namespace validation allocation、wall time 与错误输出，确认分配随 descriptor payload 而非 split segment 临时容器增长。
17. 1/100/1000 个 runtime provider/manifest selection 构建 availability，记录 builtin catalog/profile build count、String clone、查重复杂度和报告 byte/order parity；产品 trace 确认未进入 frame/tick。
18. 1/100/1000 个 export feature selections 运行 diagnostics/projection，记录 normalized-id allocation 与 wall time，并运行完整 profile feature matrix 保证报告顺序和语义。
19. 对合法/缺 dot/空段/非法字符/相邻 owner prefix 的 feature ids 运行 validation parity，并记录 split/prefix allocation 为 0。
20. 1/100/1000 packages × 1/10/100 features 运行完整 export validation/sanitize/provider/profile pipeline，记录访问/build-count、wall time和 diagnostics/generated-file parity。
21. browser/Android/iOS export host 输入 fixture 在 125/500/1000 Hz 与 1/5/10 pointers 下记录 received/coalesced/ABI-dispatched、queue age 和主线程 wall time，并与 desktop Runtime12 snapshot parity。
22. ZIP 1 KiB/1 MiB/1 GiB native entries 记录 peak RSS/read bytes/throughput，验证流式路径且 archive 内容/顺序/错误 parity。
23. 1/100/1000 package trees 的 materialize→preview→archive 记录 enumerate/stat/manifest parse/write/copy bytes；重复 unchanged run 的实际写入为 0，有界并行输出顺序确定。
24. 1 MiB/100 MiB synthetic generated contents 构建 validate report/JSON/stdout，记录 clone bytes、peak RSS、serialized bytes；compact schema 与 optional content artifact 做 digest/content parity。
25. 对 1/100/1000 个 runtime feature manifests 运行 feature validation，记录 split/prefix allocation、wall time与 diagnostic parity；合法、缺 dot、空/非法 segment 和 `render`/`rendering` owner 边界必须全覆盖。
26. 对 namespace/semver 运行合法、单段、空段、非法字符、少/多 component matrix，记录 segment allocation 为 0 且 diagnostic byte/order parity。
27. 1/100/1000 capability/interface/module/root/contribution/feature rows 运行完整 package validation，记录 membership probes、wall time、allocation 和 diagnostics；总 lookup 必须线性。
28. optional feature + feature extension provider 混合集合验证 borrowed identity 去重，并覆盖 event/system owner dot boundary；成功路径 identity/prefix String allocation 为 0。
29. 1/100/1000 plugin 的 `from_plugins/from_descriptors` 记录 registration report、diagnostics 与 bridge dependency graph build-count；一次批量构造只能 rebuild 一次且报告 byte/order parity。
30. 1/100/1000 plugins × features 的 project completion→dependency report→extension merge，记录 definition/projection build-count、lookup/remove/move 次数与 wall/allocation；目标为 generation 一次 projection 和 O(V+E) feature resolution。

## 路由

- `PERF-MVP-021` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-native-plugin-callback-global-lock.md`。
- `PERF-MVP-022` 已移交 `docs/plans/zircon_plugins/02-sound.md`，记录位于 `docs/plans/zircon_plugins/02/failure-2026-07-17-kira-graph-sync-repeated-compilation.md`。
- `PERF-MVP-034` 已把 host API registry 的 bridge-call context 改为 `Arc` snapshot；两次 lookup 的 focused test 锁定 `Arc::ptr_eq`，原有 disabled/missing/panic/dispatch tests 继续负责 ABI 语义。Rustfmt/静态 guard 已过，Cargo/压力仍 pending。
- `PERF-MVP-035` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-native-plugin-discovery-recursive-rescan.md`。
- `PERF-MVP-036` 已直接消除 candidate/package manifest 深 clone；动态插件加载契约通过前仍不把模块移入 `review.md`。
- `PERF-MVP-037` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-native-registration-replay-per-system-rebuild.md`。
- `PERF-MVP-038` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-native-load-report-repeated-projection.md`。
- `PERF-MVP-039` 已直接消除每 diagnostic 两次临时 needle allocation；Cargo focused tests 通过前仍为 dynamic pending。
- `PERF-MVP-040` 已直接消除 runtime hot update 每 candidate 一次深 clone；Cargo hot-update tests 通过前仍为 dynamic pending。
- `PERF-MVP-041` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-native-systems-conservative-world-writer-serialization.md`；Runtime08/11 共同验收调度 access 与 worker 证据。
- `PERF-MVP-042` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-bridge-import-stable-call-double-mutex.md`。
- `PERF-MVP-043` 已移交 `docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`，记录位于 `docs/plans/zircon_runtime/runtime/08/failure-2026-07-17-plugin-system-shared-callback-mutex.md`；Plugins01/Runtime11 共同验收 native generation 与 worker 行为。
- `PERF-MVP-044` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-native-host-api-global-context-lock.md`。
- `PERF-MVP-045` 已直接移除 ABI call 的完整 diagnostic snapshot 物化；源码守卫/rustfmt 已过，当前 Cargo run 早于本次编辑启动，完成后必须 warm 重跑才算动态证据。
- `PERF-MVP-046` 已直接让 unchanged bridge finalize 复用既有 table，并增加 disabled-state 保持测试；warm Cargo 通过前仍为 dynamic pending。
- `PERF-MVP-047` 已直接消除三条 namespace validator 的 split-segment 临时 Vec；源码守卫/rustfmt 已过，warm Cargo 行为测试与规模分配基准通过前仍为 dynamic pending。
- `PERF-MVP-048` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-runtime-profile-availability-rebuild.md`；先做启动规模 build-count/trace，再决定共享 projection 形态。
- `PERF-MVP-049` 已直接移除 export profile nested feature search 的 normalized String 分配；源码守卫/行为 helper/rustfmt 已过，当前源码 warm Cargo 与完整 export feature matrix 通过前仍为 dynamic pending。
- `PERF-MVP-050` 已直接消除 export project feature namespace/identity 的 segment Vec 与 formatted owner prefix；源码守卫/owner-boundary test/rustfmt 已过，warm Cargo parity 仍 pending。
- `PERF-MVP-051` 已移交 `docs/plans/zircon_plugins/09-export-publishing.md`，记录位于 `docs/plans/zircon_plugins/09/failure-2026-07-17-export-profile-validation-quadratic-scans.md`。
- `PERF-MVP-052` 已移交 `docs/plans/zircon_plugins/09-export-publishing.md`，记录位于 `docs/plans/zircon_plugins/09/failure-2026-07-17-export-host-high-frequency-input-dispatch.md`；Runtime12 共同冻结跨平台输入合并契约。
- `PERF-MVP-053` 已直接把 ZIP native package entry 改为流式压缩；源码守卫/rustfmt 已过，warm Cargo 与大文件 RSS/ZIP parity 仍 pending。
- `PERF-MVP-054` 已移交 `docs/plans/zircon_plugins/09-export-publishing.md`，记录位于 `docs/plans/zircon_plugins/09/failure-2026-07-17-export-materialize-repeated-tree-scan-and-copy.md`。
- `PERF-MVP-055` 已移交 `docs/plans/zircon_plugins/09-export-publishing.md`，记录位于 `docs/plans/zircon_plugins/09/failure-2026-07-17-export-validate-report-full-content-clone.md`。
- `PERF-MVP-056` 已直接消除 runtime feature namespace/owner 验证的 segment Vec 与 formatted prefix；源码守卫、owner-boundary test、rustfmt 与 diff check 已过，warm Cargo/注册规模分配基准仍 pending。
- `PERF-MVP-057` 已直接消除 package namespace 与 semver validator 的 segment Vec；源码守卫、shape/component 诊断行为测试、rustfmt 与 diff check 已过，warm Cargo/分配基准仍 pending。
- `PERF-MVP-058` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-package-validation-quadratic-uniqueness-scans.md`。
- `PERF-MVP-059` 已直接让 provider uniqueness 借用 identity pair，并移除 event/system owner prefix String；源码守卫、duplicate/owner-boundary tests、rustfmt 与 diff check 已过，warm Cargo 仍 pending。
- `PERF-MVP-060` 已直接批量构造 catalog reports、移除 completed manifest 二次 completion、流式 support predicate并以 bool seen 排序；源码守卫/owner-target behavior/rustfmt/diff check 已过，warm Cargo/构造 build-count benchmark 仍 pending。
- `PERF-MVP-061` 已移交 `docs/plans/zircon_plugins/01-plugin-architecture-core.md`，记录位于 `docs/plans/zircon_plugins/01/failure-2026-07-17-runtime-plugin-catalog-derived-projection-rebuild.md`。
- world-extension plan 冷路径继续由 Runtime 06/Plugins 01 在产品 trace 中确认；未取得动态证据前不做提前缓存或容器重写。

本记录仅证明上述范围已完成静态阅读，不满足 `review.md` 的动态验收条件；对应目录继续保留在 `pending.md`。
