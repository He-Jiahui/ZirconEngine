---
title: Runtime Foundation Module、Config、Event、Service、Driver、Manager、Persistence、Lifecycle 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime118
review_date: 2026-08-23
baseline_head: 9fee3ea0435961a81c85aa2502e64f1f357345d7
baseline_epoch: 365
supersedes:
  - docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/framework/foundation
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/foundation
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/settings.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/runtime_services.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/project_access.rs
tests:
  - zircon_runtime/src/foundation/tests.rs
  - zircon_runtime/src/foundation/runtime/config_manager_tests.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_app/src/entry/tests/entry_config_storage.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-18-config-manager-synchronous-full-file-rewrite.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp
  - dev/UnrealEngine/Engine/Source/Editor/EditorConfig/Private/Tests/JsonConfigTests.cpp
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/config/project_settings.cpp
  - dev/godot/core/io/config_file.h
  - dev/godot/core/io/config_file.cpp
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/object/message_queue.cpp
  - dev/godot/tests/core/config/test_project_settings.cpp
  - dev/godot/tests/core/io/test_config_file.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_registry.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_reader.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_writer.rs
  - dev/bevy/crates/bevy_ecs/src/message/update.rs
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/Fyrox/editor/src/settings/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/DebugManagerTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/Debugging/GetItemTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/Debugging/PanelNameAndOrderTests.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99s · Runtime Foundation Current Source Review

## 1. 结论

Foundation已有可保留的局部机制，但还不是工程级配置、事件与模块基础设施。配置写入已脱离调用线程，具备dirty/persisted generation、25ms trailing debounce、单owner worker、atomic stage/commit、backup recovery、显式有界flush、失败指标和迟到writer fence；Core EventBus具备按topic订阅、Lossless/BoundedDropOldest/Latest、per-subscriber queue、断开回收、精确counter和默认每64次采样的时延诊断。Runtime55之后还加入path gate最后owner回收、EventBus采样诊断、最后订阅者零额外`Arc` clone与受管benchmark。这些都应保留为重构输入。

但是Foundation仍在发布不真实的能力合同。`ConfigDriver`和`EventDriver`仍是零字段ZST，却以Immediate driver激活；两个manager的dependency仍为空，既不依赖也不调用driver；`EventManager`在production调用图中仍没有resolver/consumer。Asset和Platform只声明Foundation module依赖，没有精确Foundation service消费边。模块图仍把“名字已注册”误报为“provider已就绪且产品正在使用”。

三个P0均未修复。App仍在module activation前后两次直写Core来模拟配置优先级；dynamic session仍只在activation前写pipelined render profile，因此磁盘旧值可覆盖本次显式session值。worker仍snapshot整个Core ConfigStore，Editor capability、Animation、Physics、window/platform/render profile等Core旁路值会在任一布局保存时搭便车写入全局文件。每个Runtime默认仍注册同一全局path，第二个live manager递增同一epoch并让第一个后续commit变stale。path gate dead-key回收只修复registry泄漏，不修复live owner互相supersede。

当前账本为 **3 P0 Open、55 P1 Open、1 P1 Partial、14 P2 Open、39 Gate Fail、1 Gate Partial**。唯一Partial是`FND-P1-039/FND-G23`：last-owner回收代码、回归和ignored benchmark已存在，但没有本轮动态验证，也没有生产cardinality/health诊断。目标仍是`CompiledFoundationContract + LayeredConfigAuthority + ScopedPersistenceBroker + TypedEventService`，并硬切空driver、Core公开旁路、调用顺序优先级和测试专用service假象。

本轮只做current-source静态review和文档记录，没有修改production、tests、Cargo或ABI；没有运行Cargo、App/Editor、双Runtime、fault、soak或benchmark，因此不能宣称性能或表现达到、超过当前Unreal。按用户范围，本篇不展开tooling优化。

## 2. 审查边界与物理冻结

| 范围 | 文件 / 行 / 非空行 / bytes / tests / dirty | fingerprint |
|---|---:|---|
| Foundation contract、manager、Core config/event owner | 31 / 2,851 / 2,535 / 92,423 / 7 / 0 | `1f8f652bab3b08bb5a420c2ea53fb9879707db20fb55d7706c34b470160dcd1b` |
| focused direct tests | 15 / 2,963 / 2,693 / 106,498 / 68 / 1 | `5c2d8127043c3c586e77bb04623313df49e336c00006b66d0449cee900487f0d` |
| App、Editor、plugin、dynamic session产品调用链 | 16 / 5,258 / 4,764 / 192,499 / 39 / 7 | `1e9774dba129012df302982bb9a45a0fdb138dd46ba581c7fc6263baf89f7779` |
| 五引擎参考实现与测试 | 36 / 24,033 / 20,787 / 880,031 / 82 / 0 | `410a3b3608b3af3c96e5a7c7738904169bbb5fe655135f12c04712d9159a2f62` |

fingerprint算法：仓库相对路径转`/`并排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾无LF，再计算UTF-8 SHA-256。它冻结本轮实际读取集合，不是config revision、event schema、module generation或release identity。

本轮按当前working tree读取。31个Foundation owner文件均clean；`zircon_app/src/entry/tests/profile_bootstrap.rs`以及7个产品调用文件已有其他会话/用户改动，本文只审查结果，不覆盖或归属这些源文件。基线HEAD为`9fee3ea0435961a81c85aa2502e64f1f357345d7`，coordinator epoch为365。MVP仍为`00 in_progress`，F0-F5按依赖保持blocked；本篇不得将高级Foundation计划写成MVP已完成。

`docs/plans/zircon_runtime/runtime/02/failure-2026-07-18-config-manager-synchronous-full-file-rewrite.md`仍为open：单worker与atomic/fence架构已有静态round-3结果，但最终source-bound Cargo、failure fixed return和里程碑提交未完成。本篇不关闭或改写该failure。

## 3. 当前产品事实链

### 3.1 Boot、磁盘恢复与旁路持久化

```text
BuiltinEngineEntry::bootstrap
  -> CoreRuntime::new
  -> store_entry_config(CoreHandle::store_config*)       [第一次，activation前]
  -> register/activate modules
       -> Foundation ConfigManager factory
          -> recover config.json
          -> for each disk key: CoreHandle::store_config_value
          -> start zr-config-persist worker
  -> store_entry_config(CoreHandle::store_config*)       [第二次，activation后]

Editor layout set_value
  -> dirty generation++
  -> worker snapshot entire Core ConfigStore
  -> pretty JSON whole-file commit
```

第二次App写确实让最终内存中的entry值压过磁盘，但没有形成layer contract：activation期间构造的consumer可看到磁盘值，第二次写又不推进ConfigManager dirty generation。更危险的是持久化projection不是“通过manager修改的durable key”，而是Core整表。因此Editor capability、Animation、Physics、session render profile等旁路值会在下一次任意layout/preset写入时被全量持久化，形成时序依赖和跨启动污染。

当前生产旁路不只在旧报告列出的App、Editor和builtin Animation中。`zircon_plugins/animation`、`zircon_plugins/physics`也直接`load_config/store_config`；Editor的enabled subsystem transaction仍直接写Core。Physics甚至把Core内存写失败包装成`persistence`错误，但该调用本身不通知Foundation worker，合同名称和真实durability继续分离。

### 3.2 Dynamic session与多Runtime反例

```text
RuntimeDynamicSession::build
  -> CoreRuntime::new
  -> store_profile_submission_config(pipelined)          [activation前Core直写]
  -> activate_registered_modules
       -> Foundation从全局config.json覆盖同名key
  -> RuntimeRenderBridge::new
       -> 从Core读取RENDER_PROFILE_CONFIG_KEY
```

现有dynamic test只在尚未激活Foundation的Core中验证profile write，未带磁盘、module activation和render bridge。只要全局文件含旧`zircon.render.profile_bundle`，本次显式pipelined profile仍可被旧值覆盖。

`ConfigCommitFence::register(path)`对同一live gate执行`current.wrapping_add(1)`。第二个manager激活会立即使第一个epoch过期；第一个仍可`set_value`并显示dirty，但commit返回cancelled/superseded。新增Drop回收只在最后`ConfigCommitFence`消失时删除map key，不改变两个live owner的语义。默认path又不含project/profile/principal/runtime/session identity，而dynamic API允许同进程保留多个session，因此P0不是理论夹具问题。

### 3.3 Event facade与Core provider

```text
FoundationModule
  -> Immediate EventDriver (ZST, no methods, no consumer)
  -> Immediate EventManager (no EventDriver dependency)
       -> CoreWeak::upgrade
       -> CoreHandle::publish_event / subscribe_events

production EventManager resolver: 0
Foundation tests: subscribe + publish + recv roundtrip
```

`EventManager::publish`仍返回`()`，Core已销毁时静默成功；`subscribe`则伪造一个zero-state disconnected subscription。Core本身继续公开第二套raw `String + serde_json::Value` authority。默认EventBus现在每64次采样routine timing，并保留exact publish/delivery/drop counters；同topic发布有串行delivery lock，bounded/latest可控，Lossless仍无容量。当前benchmark全部`#[ignore]`，只说明证据入口存在，不是本轮通过的性能门。

## 4. 可保留基础与已确认局部进展

- ConfigStore以`Arc<Value>`保存，typed load直接从共享JSON反序列化，避免一次中间深clone。
- Foundation manager只持`CoreWeak`，registry service不会单独保活Runtime root。
- config worker把snapshot/serialization/I/O移出调用线程，用dirty/persisted/attempted generation区分请求、尝试和成功。
- 同值写不会无条件推进dirty；失败后同值set或显式flush仍可重新请求提交。
- atomic writer复用resource I/O staging，commit fence可阻止已取消或被替代的迟到writer越过新代提交。
- path gate现在在最后owner Drop时删除匹配Weak entry；source test与65,536 path ignored benchmark存在，`FND-P1-039`因此为Partial。
- EventBus会回收dead subscriber和空topic；bounded/latest报告drop；recv/try/timeout/disconnect分态。
- EventBus默认timing sampling interval为64，exact counters不采样；最后subscriber接收原`Arc`，fanout减少一次payload clone。
- Manager handle包含index/generation/name并在resolve时复核，明显stale handle会被拒绝。

这些局部机制不能把空driver、全局文件、整Store snapshot、ignored benchmark或测试roundtrip升级成产品资格。

## 5. P0 阻断项

| ID | Status | 当前证据与后果 | 硬切目标 / owner |
|---|---|---|---|
| `FND-P0-001` | Open | App前后两次Core直写；dynamic session只在activation前写profile；磁盘恢复无条件覆盖；worker又snapshot整Store。启动优先级依赖调用时序，session值可被旧盘覆盖，也可被无关布局保存带入全局文件 | `BootConfigSnapshot + ConfigLayerCompiler`先组成带source/scope的effective revision；只有schema声明durable的key进入backend。Runtime118 + Runtime03/App |
| `FND-P0-002` | Open | 默认manager共享单一用户path；第二个live manager递增epoch并使第一个commit stale；dynamic session可并存多个CoreRuntime | process-scoped persistence broker或显式exclusive/shared scope lease；activation前fail-close，绝不静默supersede live owner。Runtime118 + Runtime03/25/45 |
| `FND-P0-003` | Open | 两个Immediate driver仍为空ZST；manager dependency为空并直调Core；EventManager product consumer为0；Asset/Platform只有module假依赖 | compiled contract只注册有行为、owner、dependency、health和consumer的provider；无独立边界则删除driver。Runtime118 + Runtime42/46/50 |

## 6. P1 工程化差距

### 6.1 Contract、Module、Driver 与 Service Composition

| ID | Status | 差距 | 目标 / owner |
|---|---|---|---|
| `FND-P1-001` | Open | ConfigManager/EventManager descriptor的service dependency均为空 | compiled dependency绑定真实provider；Runtime46 + Runtime118 |
| `FND-P1-002` | Open | 空driver类型与name仍从`foundation`公开导出 | 删除导出；若有真实边界则提供contract、health和resolver |
| `FND-P1-003` | Open | Kernel Foundation在factory中spawn OS线程和文件I/O，却不声明executor/blocking/filesystem/shutdown capability | descriptor声明资源、affinity、budget和teardown policy；Runtime02/46 |
| `FND-P1-004` | Open | Foundation没有`ModuleLifecycle`，flush/drain/stop只藏在service Drop | module显式quiesce -> flush -> drain -> stop并返回terminal receipt |
| `FND-P1-005` | Open | worker spawn成功即Manager Ready，panic/failed writer没有Degraded/Failed | readiness绑定worker generation、backend health和remediation |
| `FND-P1-006` | Open | Asset声明Foundation module dependency，但没有Foundation service edge/消费点 | 删除假依赖或消费精确contract |
| `FND-P1-007` | Open | Platform声明Foundation module dependency，真实manager只依赖PlatformDriver | module ordering不得替代service dependency |
| `FND-P1-008` | Open | EditorManager虽声明ConfigManager依赖，host仍并存manager与Core直写 | dependency约束实际API并禁止旁路 |
| `FND-P1-009` | Open | manager常量与descriptor靠字符串硬对齐，测试主要比对文字 | compiled contract key；name只作显示 |
| `FND-P1-010` | Open | Foundation manifest不报告scope/provider/durability/delivery/optional | capability manifest报告声明、resolved provider和live health |
| `FND-P1-011` | Open | Foundation无条件进入client/server/editor core candidates | target profile显式选择Config/Event/Persistence能力 |
| `FND-P1-012` | Open | shader viewer手工激活Foundation，只验证调用存在 | 工具profile通过同一compiled plan取得最小能力和load receipt |

### 6.2 Config Contract、Layer、Persistence 与 Lifecycle

| ID | Status | 差距 | 目标 / owner |
|---|---|---|---|
| `FND-P1-013` | Open | public contract是任意`&str -> Value` | typed `ConfigKey<T>`、owner namespace、schema id/version |
| `FND-P1-014` | Open | `get_value: Option`混合missing和RuntimeUnavailable | typed `Result<ConfigRead<T>, ConfigError>` |
| `FND-P1-015` | Open | 没有remove/reset-to-default/clear-override | typed mutation返回revision |
| `FND-P1-016` | Open | 没有key registry、default、validator、restart/live-apply、secret policy | Runtime03 ConfigRegistry作为唯一声明源 |
| `FND-P1-017` | Open | engine/project/user/profile/session/CLI/env挤进同一HashMap | 显式layer/scope precedence和source explanation |
| `FND-P1-018` | Open | 多key更新无prepare/validate/commit、CAS或revision | 原子transaction发布单一revision/delta |
| `FND-P1-019` | Open | 磁盘恢复逐key写Core，consumer可观察部分状态 | activation前构造不可变snapshot并一次发布generation |
| `FND-P1-020` | Open | 文件无format/key version、migration、unknown-key report | versioned envelope、migration、quarantine、forward preserve |
| `FND-P1-021` | Open | `ZIRCON_CONFIG_PATH`是进程全局字符串 | host注入typed persistence address |
| `FND-P1-022` | Open | 平台目录缺失时回退CWD `.zircon-config.json` | host明确决定path；不能确定则禁durability并报告 |
| `FND-P1-023` | Open | 默认path不含project/profile/principal/runtime/session identity | address包含scope identity、tenant、backend |
| `FND-P1-024` | Open | worker snapshot整个Core Store | durable projection只含registry授权key和revision |
| `FND-P1-025` | Open | `CoreHandle/CoreRuntime::store_config*`公开旁路且不推进dirty | 迁移调用点后物理删除，无compat alias |
| `FND-P1-026` | Open | BuiltinEngineEntry两次旁路写模拟precedence | host提交一次source-tagged boot snapshot |
| `FND-P1-027` | Open | Editor布局走manager，subsystem/sandbox走Core直写 | Editor统一typed scope transaction |
| `FND-P1-028` | Open | builtin/plugin Animation与plugin Physics继续Core直写 | hard cut到canonical authority并返回真实durability receipt |
| `FND-P1-029` | Open | dynamic session profile是pre-activation raw store | immutable session bootstrap layer |
| `FND-P1-030` | Open | Config commit没有typed observer/delta | authority commit后锁外发布revision delta |
| `FND-P1-031` | Open | production没有`ConfigManager::flush`调用，退出依赖Drop | host/module shutdown显式fence并收集receipt |
| `FND-P1-032` | Open | flush只表示dirty generation，无durability level/backend分项 | written/data-synced/metadata-synced逐backend结果 |
| `FND-P1-033` | Open | report没有path/scope/runtime/backend/observed_at | health snapshot带identity和window |
| `FND-P1-034` | Open | pending/peak flush实际只有0/1 | 改为pending generation或真实queue/backlog统计 |
| `FND-P1-035` | Open | I/O/parse/timeout/panic折叠为`ConfigParse + String` | typed operation/path/source/retryability/disposition |
| `FND-P1-036` | Open | 写失败没有timer/backoff retry，只靠set/flush触发 | bounded retry、jitter、health transition和terminal failure |
| `FND-P1-037` | Open | dirty generation用`saturating_add`并静默耗尽 | checked exhaustion fail-close；Runtime24 |
| `FND-P1-038` | Open | path epoch用`wrapping_add`并可能复用身份 | checked/non-reusing identity；Runtime24/25 |
| `FND-P1-039` | Partial | last-owner Drop已删除匹配Weak entry，source test和ignored benchmark存在；未暴露cardinality/health，本轮未运行 | bounded registry生产诊断 + current source-bound验证 |
| `FND-P1-040` | Open | `load-if-equal`与`store`分两次锁，无transaction revision | authority锁内/versioned compare-and-commit |
| `FND-P1-041` | Open | worker snapshot closure强持ConfigStore，Runtime root消失后仍可访问旧store | worker绑定module generation并封存final snapshot |
| `FND-P1-042` | Open | Drop超时只tracing并丢JoinHandle，无调用方receipt | canonical execution domain与产品terminal result；Runtime02/118 |

### 6.3 Event Contract、Authority 与 Product Use

| ID | Status | 差距 | 目标 / owner |
|---|---|---|---|
| `FND-P1-043` | Open | EventManager在production没有resolver/consumer | 至少一个真实产品链消费，否则删除service |
| `FND-P1-044` | Open | CoreHandle/CoreRuntime仍公开raw event API，EventManager只是转发 | 唯一event authority并删除公开旁路 |
| `FND-P1-045` | Open | topic任意String、payload任意JSON | typed event id、schema digest/version、owner、codec |
| `FND-P1-046` | Open | publish返回`()`，无sequence/delivery/drop/subscriber/receipt | typed publication result；可靠流使用ack/cursor |
| `FND-P1-047` | Open | Core不可用时publish静默成功 | `Unavailable/Quiescing/Closed` fail-close |
| `FND-P1-048` | Open | Core不可用时subscribe制造zero-state对象 | admission直接返回typed unavailable |
| `FND-P1-049` | Open | 无event generation、producer、clock、ordering、compat window；subscriber ID仍可wrap | envelope声明scope/generation/sequence/clock/schema |
| `FND-P1-050` | Open | 无runtime/world/session/player/view scope或principal/capability | scoped catalog与access policy |
| `FND-P1-051` | Open | Config commit、module lifecycle、service health不发布typed Foundation event | authority commit后发布一致revision |
| `FND-P1-052` | Open | EventManager contract不暴露Core diagnostics/health；新增sampling只在provider | bounded service status、drop/gap/remediation |
| `FND-P1-053` | Open | public policy含无容量Lossless | 有预算reliable/ephemeral class和overflow |
| `FND-P1-054` | Open | UI/Asset/Resource/Scene/Foundation各自事件体系无catalog/bridge边界 | catalog声明owner、scope、bridge和禁止路径 |

### 6.4 Tests 与 Qualification

| ID | Status | 差距 | 目标 / owner |
|---|---|---|---|
| `FND-P1-055` | Open | Foundation tests主要证明roundtrip/name/root结构；唯一EventManager consumer仍是测试；EventBus perf tests为ignored | contract/compile-fail/product tests证明consumer、旧路径删除和managed evidence |
| `FND-P1-056` | Open | reload test在第一manager flush后才建第二Runtime；没有两个live manager、boot layer、dynamic profile、whole-store泄漏矩阵 | simultaneous multi-runtime + precedence + leakage + restart RED/GREEN |

## 7. P2 完整性与可维护性差距

| ID | Status | 差距 | 收敛方向 |
|---|---|---|---|
| `FND-P2-001` | Open | `contains_key`通过`get_value`深clone JSON | no-copy presence/read guard或typed snapshot |
| `FND-P2-002` | Open | 每次`get_value`深clone完整Value | immutable snapshot或typed decode cache |
| `FND-P2-003` | Open | worker固定`to_vec_pretty` | canonical compact codec；调试导出另行pretty |
| `FND-P2-004` | Open | HashMap序列化顺序不稳定 | canonical key order与deterministic digest |
| `FND-P2-005` | Open | report复制并排序最多64个latency samples | online histogram/sketch或预聚合窗口 |
| `FND-P2-006` | Open | p95/max无sample count/window/reset generation | 完整metric window metadata |
| `FND-P2-007` | Open | `serialized_bytes`包含失败attempt | attempted/written/committed bytes分开 |
| `FND-P2-008` | Open | metrics用saturating counter且无exhausted bit | checked或显式exhausted health |
| `FND-P2-009` | Open | 每次manager构造重读env/CWD | host冻结一次typed address plan |
| `FND-P2-010` | Open | public driver name扩大无用稳定表面 | 删除占位constant |
| `FND-P2-011` | Open | EventManager每次publish复制topic String | registered dense topic id |
| `FND-P2-012` | Open | subscribe总分配`Box<dyn Subscription>` | typed/generation cursor lease或pool |
| `FND-P2-013` | Open | Foundation重复手写disconnected subscription语义 | availability统一在resolver/admission表达 |
| `FND-P2-014` | Open | source-string结构测试易受格式影响且不证明行为 | AST/compile contract + state tests |

## 8. 参考引擎对照与适用边界

| 参考 | 已核对机制 | Zircon应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal | `FConfigCacheIni/FConfigContext`区分hierarchy、static/dynamic/saved/runtime change、command-line override与load/flush；ModuleManager只在`StartupModule`完成后置Ready，记录load order，shutdown先统一PreUnload再逆序Shutdown；Messaging context携带type/sender/recipients/scope/time/expiration并有authorizer/tracer/shutdown | source/layer、load readiness、显式teardown、typed message context、可观察lifecycle | 不复制`GConfig`全局单例、INI语法或C++ DLL细节 |
| Godot | ProjectSettings记录initial/current、persist/basic/internal/restart/order/version/changed set/feature override；config version拒绝future file；ConfigFile与MessageQueue各有明确load/save/error和bounded queue配置 | key metadata、revision、restart policy、override解释、format version、bounded deferred work | 不把Object singleton/Variant全局表当Zircon最终类型系统 |
| Bevy | Plugin具备build/ready/finish/cleanup状态；Message按Rust类型注册，返回单调`MessageId`，reader持cursor并报告missed，双buffer明确两次update保留窗口 | typed registration、ready/finish、per-consumer cursor、retention/missed contract | 两帧message不是跨线程可靠broker，也不替代持久化 |
| Fyrox | Plugin区分registration/init/on_loaded/on_deinit和受控context；ResourceEvent是typed enum，subscriber有generation handle并自动回收dead sender；Editor settings用dirty flag、subscriber和显式save | typed domain event、对称生命周期、generation handle、真实consumer通知 | std mpsc与简单RON save仍不足以证明预算、多Runtime、ack和durability |
| Unity Graphics | DebugManager有真实panel/data consumer、Register/Unregister和dirty/reset callback；Editor window/tests对称注册、移除panel并验证排序/状态 | service必须有真实产品consumer、对称注册撤销和可验证状态 | 仅为Graphics package局部证据；singleton/editor callback不是通用Runtime authority |

## 9. 目标架构与硬切边界

```text
Host / Dynamic Session
  -> BootConfigSnapshot
       { source, scope, profile, project, principal, typed values }
  -> FoundationContractCompiler
       -> ConfigRegistry + layer precedence + migration/validation
       -> EventCatalog + schema/scope/delivery class
       -> resolved provider capability + health
  -> CompiledFoundationContract
       -> ConfigAuthority
            immutable effective revision
            transaction + typed delta
            durable projection only
       -> ScopedPersistenceBroker
            address/lease/CAS/recovery/durability receipt
       -> TypedEventService
            type id + generation + cursor/ack/drop/gap
       -> Lifecycle
            activate -> ready -> quiesce -> flush/drain -> stopped
```

硬切规则：

1. `core::framework::foundation`只保留纯合同、typed DTO和identity，不放worker/path/EventBus实现。
2. 没有独立外部边界的driver物理删除；有provider时manager必须通过descriptor依赖并调用。
3. boot/session override与durable user/project配置使用不同类型、scope和address，不再靠写入顺序覆盖。
4. `ConfigAuthority`是唯一持久配置写入口；迁移完成后删除全部Core config写旁路。
5. `TypedEventService`是唯一Foundation事件入口；Core EventBus降为不可外用provider。
6. 多Runtime通过process broker共享明确scope，或activation前取得exclusive lease；不得暗中杀死live owner。
7. manager call lease、native unload和worker execution domain继续由Runtime01/02/50拥有，本篇不另造kernel。

## 10. 重构里程碑

### M118.0 · Truth Freeze 与 RED Repro

- 冻结typed key/event、scope/source/revision/provider capability和terminal receipt合同。
- 建立disk覆盖session profile、whole-store泄漏、两个live runtime同path、empty driver伪Ready四个RED。

### M118.1 · Foundation Descriptor Truth

- 删除空ConfigDriver/EventDriver及public names，或接入真实provider与health。
- Asset/Platform/Editor改为精确service dependency；无消费边删除module dependency。

### M118.2 · Boot Config Hard Cut

- App和dynamic session在任何consumer activation前提交一次`BootConfigSnapshot`。
- 删除App双写和dynamic session pre-activation raw store。

### M118.3 · Typed Config Authority

- 落地registry、scope/layer、typed read、transaction、revision、delta、migration和source diagnostics。
- 迁移Editor、Animation、Physics、Window/Platform/Render profile并删除Core旁路。

### M118.4 · Scoped Persistence Broker

- 引入typed address、owner lease、durable projection、CAS/lock/recovery和backend capability。
- Editor+PIE+tool+multi-session显式共享或隔离，不能silent supersede。

### M118.5 · Typed Event Service 与 Product Consumer

- Core EventBus收进内部provider；建立typed catalog、publication result、cursor/gap、health。
- 接通至少一个真实App/Runtime/Editor纵向consumer，并删除测试专用facade或旧Core入口。

### M118.6 · Lifecycle 与 Shutdown Receipt

- Foundation module显式quiesce Config/Event、拒绝新调用、flush/drain worker/subscriber并返回逐provider receipt。
- timeout、panic、stale manager、native unload统一接入Runtime02/50执行域。

### M118.7 · Product、Fault 与 Performance Qualification

- 运行App/Editor/PIE/dynamic/server、project/profile/principal、restart/migration/fault矩阵。
- 正确性通过后测1/8/64 Runtime、1K/100K key、1/1K topic、subscriber storm、real disk、RSS和p95/p99；只做同语义、同硬件比较。

## 11. 验收矩阵

| Gate | Status | 验收内容 |
|---|---|---|
| `FND-G01` | Fail | compiled contract拒绝duplicate/missing/wrong-contract provider |
| `FND-G02` | Fail | descriptor不存在无行为driver或无消费capability |
| `FND-G03` | Fail | Asset/Platform/Editor依赖追踪到精确service call |
| `FND-G04` | Fail | target未选择的capability不注册、不Ready |
| `FND-G05` | Fail | boot config在consumer前形成单一effective revision |
| `FND-G06` | Fail | disk/user不能覆盖高优先级session/CLI |
| `FND-G07` | Fail | session-only render/window/platform永不durable |
| `FND-G08` | Fail | key有type/schema/version/default/validator/scope/source |
| `FND-G09` | Fail | 多keytransaction全成或全不成一个revision |
| `FND-G10` | Fail | remove/reset/migration/unknown/invalid有typed result |
| `FND-G11` | Fail | observer只收commit revision且锁外执行 |
| `FND-G12` | Fail | Core config写旁路物理删除 |
| `FND-G13` | Fail | App/Editor/Animation/Physics/dynamic全部迁移 |
| `FND-G14` | Fail | 两个live Runtime同path按声明共享或activation fail-close |
| `FND-G15` | Fail | 第二Runtime不能让第一Runtime commit静默stale |
| `FND-G16` | Fail | project/profile/principal/session scope不串扰 |
| `FND-G17` | Fail | same/cross-process与symlink/case alias冲突矩阵通过 |
| `FND-G18` | Fail | revision/epoch/ticket耗尽显式失败且不复用 |
| `FND-G19` | Fail | worker panic/timeout/backend failure进入Degraded/Failed |
| `FND-G20` | Fail | flush返回durability level和逐backend receipt |
| `FND-G21` | Fail | module显式quiesce/flush/drain/stop，不依赖Drop |
| `FND-G22` | Fail | deadline后无迟到commit越过module generation |
| `FND-G23` | Partial | last-owner回收源码与测试存在；缺生产cardinality及本轮动态验证 |
| `FND-G24` | Fail | malformed/old/future config有migration/quarantine证据 |
| `FND-G25` | Fail | Event catalog注册schema/scope/owner/version/class |
| `FND-G26` | Fail | publish返回sequence/result，Unavailable不静默 |
| `FND-G27` | Fail | subscriber generation cursor报告missed/drop/gap |
| `FND-G28` | Fail | reliable类有capacity/ack/overflow/resync |
| `FND-G29` | Fail | Event service暴露topic/subscriber/queue/drop/health |
| `FND-G30` | Fail | Core event旁路删除或降为internal |
| `FND-G31` | Fail | 至少一个真实产品consumer闭环 |
| `FND-G32` | Fail | window/module/session consumer注册撤销对称 |
| `FND-G33` | Fail | Config commit event携带相同revision |
| `FND-G34` | Fail | call guard/provider lease覆盖quiesce/unload |
| `FND-G35` | Fail | App/Editor/dynamic/server cold/warm restart通过 |
| `FND-G36` | Fail | simultaneous Editor+PIE+tool无串扰 |
| `FND-G37` | Fail | key/topic benchmark报告CPU/allocation/lock/RSS/p95/p99 |
| `FND-G38` | Fail | disk/permission/panic/hung writer/drop storm fault通过 |
| `FND-G39` | Fail | 同硬件同语义比较同时过正确性、失败和性能门 |
| `FND-G40` | Fail | frontmatter/path/link/index/count/fingerprint/diff-check全复核 |

## 12. Owner 去重与状态

| Owner | 本报告回写 | 不得改写为 |
|---|---|---|
| Runtime01/02 | canonical lifecycle/execution domain、detached worker、EventBus内部调度 | Runtime118另造线程池或重复DLL-unload P0 |
| Runtime03 | ConfigRegistry、layer/schema、Core direct-write hard cut | dirty worker已等于完整Config系统 |
| Runtime24/25 | checked identity、path/VFS、atomic recovery、cross-process lock/CAS | lowercase path或atomic rename已解决identity/durability |
| Runtime42/46 | target catalog、compiled descriptor、capability truth、module lifecycle | service count大于0即Ready |
| Runtime45 | Preference backend、scope overlay、multi-process durability、Editor settings | raw Foundation config替代Preference系统 |
| Runtime50 | service directory、call guard、provider lease、stale generation | resolver返回Arc后可无guard长期调用 |
| Runtime43/App01 | dynamic session/product host创建和关闭多个Runtime并呈现terminal result | 单元测试`CoreRuntime::new`等于产品闭环 |
| Runtime118 | Foundation纵向组合、boot precedence、durable projection、多Runtime path owner、真实Config/Event consumer和旧入口删除 | 再包装facade、保留空driver或靠调用顺序覆盖 |

| 项目 | 状态 | 说明 |
|---|---|---|
| Runtime118 review | `review_complete` | 3 P0 Open / 55 P1 Open / 1 P1 Partial / 14 P2 Open |
| Production重构 | `pending` | 未修改源码、测试、Cargo、ABI或产品行为 |
| Runtime02 config failure | `open / validation_pending` | 保留既有静态成果，不提前fixed return |
| Gate | `39 Fail / 1 Partial` | G23只有静态局部进展 |
| Dynamic/performance validation | `not_run` | 未运行Cargo、双Runtime、Editor、DLL、fault、soak或benchmark |

首个实现切片必须是M118.0：先建立四个产品语义RED和capability truth合同，再删除空driver与Core旁路。不得先在现有raw HashMap、全局path或String/JSON event facade上继续堆功能。
