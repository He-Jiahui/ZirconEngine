---
title: Runtime Foundation Module、Config、Event、Service、Driver、Manager、Persistence、Lifecycle 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime55
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/foundation
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/events
  - zircon_runtime/src/foundation
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
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
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-18-config-manager-synchronous-full-file-rewrite.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/config/project_settings.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_registry.rs
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Fyrox/editor/src/plugin.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 55 · Runtime Foundation Module、Config、Event、Service、Driver、Manager、Persistence、Lifecycle 与 Product Integration 工程化差距

## 1. 结论

Foundation不是全空壳。配置持久化已经有dirty/persisted generation、25ms trailing debounce、单owner worker、atomic staging/commit、进程内path epoch fence、backup recovery、有界显式flush、失败指标，以及用`CoreWeak`避免registry反向保活Runtime的基础。Core EventBus也已有按topic订阅、Lossless/BoundedDropOldest/Latest策略、per-subscriber cursor queue、断开清理和可选诊断。这些实现应保留为重构输入，不能退回同步整文件写或无界临时channel。

但`FoundationModule`当前发布的是不真实的工程合同。`ConfigDriver`和`EventDriver`都只是零字段ZST，却以`StartupMode::Immediate`注册为已激活driver；两个manager既不依赖也不调用它们。`EventManager`在全部production Rust调用图中没有一个解析者，只有Foundation/App测试自造订阅者证明roundtrip。Asset与Platform声明依赖Foundation，却没有任何Foundation service依赖边或消费点。模块图因此把“名字存在、factory返回Arc”误报成了“能力已实现且产品正在使用”。

配置链还有两个可直接破坏产品语义的阻断。第一，App在模块激活前后两次把entry配置直写Core，动态session则只在激活前写pipelined render profile；Foundation激活时会把全局`config.json`逐项无条件写回Core。与此同时，只要Editor布局等任一持久设置变化，worker就snapshot整个Core ConfigStore，把render profile、window、platform和session override一起落盘。这样旧磁盘值可以覆盖本次动态session显式profile，session-only值又会反向污染下一次启动。第二，每个Foundation manager默认使用同一全局路径；第二个仍存活的Runtime激活manager时会递增同一路径epoch，静默使第一个manager的后续commit全部变成stale。动态session API明确允许同进程创建多个`CoreRuntime`，因此这不是理论上的测试夹具问题。

本轮登记 **3项P0、56项P1、14项P2和40项验收门禁**。目标是建立`CompiledFoundationContract + LayeredConfigAuthority + TypedEventService + ScopedPersistenceBroker`，硬切删除空driver、Core公开旁路和测试专用能力假象。Runtime03继续拥有通用config schema/layer，Runtime02拥有EventBus与worker执行/DLL-unload，Runtime25拥有文件原语，Runtime45拥有跨产品Preference backend，Runtime46/50拥有通用module/service/manager kernel；本篇拥有Foundation纵向组合、启动优先级、多Runtime隔离、产品消费和删除旧路径的闭环。

本轮仅做静态review与文档总账，没有修改production、tests、Cargo、ABI或reference source；没有运行Cargo、Editor、动态session、双Runtime持久化、故障注入、soak或benchmark。不能据此宣称性能或表现优于Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| Foundation contract、manager、Core config/event owner | 31 / 2,658 / 85,478 / 5 | SHA-256 `8a20ba6f32d9b4bf1f0dafed551deb4ff73fc86842961c1a1013f3ae7a14d917` |
| App、Editor、dynamic session与依赖模块产品链 | 14 / 4,915 / 183,004 / 31 | SHA-256 `7412444bbd8c7a44bb1f66abc62c22db779a36c6e2f5e6bdb3d08fdf6fd7a27b` |
| focused direct tests | 15 / 3,335 / 120,234 / 80 | SHA-256 `54feed85b074ec5ad5ce41d4c8d25cf9a139c6decb08dda7c70a1a66ac018803` |
| reference corpus | 14 / 9,199 / 348,007 / 25 | SHA-256 `0f09719889061633ed598e52c8518a0608aaf4802b40b410adfc014e592b455e` |

fingerprint算法与Runtime54一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`编码，LF连接且末尾不追加LF，再计算UTF-8 SHA-256。它只冻结本轮读取集合，不是config revision、event schema、module generation或release identity。

Foundation、framework foundation与core manager本轮读取文件为clean；`zircon_editor/src/ui/host/project_access.rs`已有其他会话/用户改动，本文按当前working tree读取且不覆盖。全局索引、coverage和consolidation也属于共享增量文档，因此`source_recheck_required`保持true。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

### 2.2 已读与未宣称范围

- 逐文件读取Foundation contract、module、两个driver、两个manager、config path/state/worker/writer/commit fence及全部focused tests。
- 读取Core ConfigStore、EventBus所有生产文件、manager resolver/service/name以及相关结构/行为测试。
- 追踪BuiltinEngineEntry、dynamic session、dynamic render loop、Editor host/layout、Animation、Asset、Platform和shader viewer调用链。
- 反查production caller：`config_manager_handle`只有Editor host解析；`event_manager_handle`没有production解析；EventManager的唯一产品形态消费在App测试。
- 读取一个open config failure；它已记录静态round-3复核，但managed current-source门和failure return未完成，本文不改变其`open`状态。
- 未把Editor retained hierarchy failure纳入本报告，因为它使用独立Editor/UI消息链，不是Foundation EventManager owner。
- 未重新审查Runtime02 EventBus全部性能结论、Runtime25 atomic I/O正确性、Runtime45 preference lane或Runtime50 manager call lease；这里只核对Foundation如何组合和消费它们。

## 3. 当前真实调用链

### 3.1 配置启动与持久化

```text
BuiltinEngineEntry::bootstrap
  -> CoreRuntime::new
  -> store_entry_config(CoreHandle::store_config*)       [第一次，绕过ConfigManager]
  -> register/activate modules
       -> Foundation ConfigManager factory
          -> recover_and_load_from_disk
          -> for each disk key: CoreHandle::store_config_value
          -> start zr-config-persist worker
  -> store_entry_config(CoreHandle::store_config*)       [第二次，仍绕过ConfigManager]

Editor layout save
  -> resolve ConfigManager
  -> set_value
  -> dirty generation++
  -> worker snapshot entire Core ConfigStore
  -> persist one global JSON map
```

二次写确实让BuiltinEngineEntry最终内存值覆盖磁盘值，但没有形成layer contract：Foundation之后、第二次写之前构造的service可观察到另一套值，而且第二次写不推进持久化generation。更严重的是worker不是只保存通过ConfigManager变更的key，而是保存整个Core store，所以旁路写入会在下一次任意manager提交时被“搭便车”持久化。

### 3.2 动态session反例

```text
RuntimeDynamicSession::build
  -> CoreRuntime::new
  -> store_profile_submission_config(pipelined)          [激活前Core直写]
  -> activate_registered_modules
       -> Foundation从全局config.json覆盖同名key
  -> RuntimeRenderBridge::new
       -> 从Core读取RENDER_PROFILE_CONFIG_KEY
```

动态session没有激活后二次回写。因此，只要全局文件含旧`zircon.render.profile_bundle`，本次显式pipelined profile就能在模块激活时被覆盖。现有测试只断言profile在“尚未激活任何module”的Core中可读，没有带磁盘Foundation激活步骤。

### 3.3 同路径多Runtime反例

`ConfigCommitFence::register(path)`从进程静态`OnceLock<HashMap<PathBuf, Weak<Mutex<PathCommitEpoch>>>>`取得gate，每次成功注册都执行`current.wrapping_add(1)`。这适合“旧manager已关闭、迟到writer必须被新manager压住”的测试，却没有owner lease或active-manager约束。两个manager同时活跃时，第二次注册立即使第一个epoch过期；第一个manager仍可`set_value`并报告dirty，但commit只能收到“cancelled or superseded”。默认路径又不含runtime/session/project identity。

### 3.4 Event facade

```text
FoundationModule
  -> Immediate EventDriver (ZST, no methods, no consumer)
  -> Immediate EventManager (no dependency on EventDriver)
       -> CoreWeak::upgrade
       -> CoreHandle::publish_event / subscribe_events

production callers resolving EventManager: 0
App/Foundation tests: subscribe + publish + recv roundtrip
```

`EventManager::publish`返回`()`；Core已销毁时直接静默返回。`subscribe`在相同情况下返回一个手写zero-state disconnected subscription。ConfigManager的写入至少返回`RuntimeUnavailable`，EventManager却把同一生命周期失败吞掉，调用方无法区分“发布成功但无订阅者”和“runtime已经消失”。

## 4. 可保留基础

- ConfigStore内部使用`Arc<Value>`，typed load直接从共享JSON反序列化，避免先深clone一次再解析。
- ConfigManager只持`CoreWeak`，manager registry不会单独保活Runtime root。
- worker把序列化与文件I/O移出调用线程，并用dirty/persisted/attempted generation区分请求和成功。
- 同值写不会无条件推进dirty；失败后同值写或显式flush仍能重新请求提交。
- atomic writer复用共享resource I/O staging，commit fence阻止已被替代的迟到writer覆盖新文件。
- shutdown有2秒默认上限，显式flush可由调用方给出timeout，错误和基本latency/byte计数可查询。
- EventBus订阅Drop能反注册并回收空topic；有界策略能报告drop，receiver阻塞/超时/断开语义已分开。
- Manager handle包含index/generation/name并在解析时复核，明显stale handle会被拒绝。

这些基础只能说明局部机制存在，不能把空driver、全局文件、全Store snapshot、测试roundtrip或service name当作产品资格。

## 5. P0 阻断项

| ID | 当前证据 | 工程后果 | 硬切目标 / owner |
|---|---|---|---|
| FND-P0-001 | App前后两次直写Core；dynamic session只在Foundation激活前写profile；磁盘恢复无条件覆盖已有Core值；任一持久变更又snapshot整个Store | 启动优先级取决于调用时序；session/CLI/profile值可被旧磁盘覆盖，也可被无关Editor布局保存带入全局文件，重启后形成自强化污染 | `BootConfigSnapshot + ConfigLayerCompiler`先组成带source/scope的effective revision，再激活consumer；只有schema声明为durable的key进入指定backend。Runtime55拥有产品硬切，Runtime03拥有通用layer/schema |
| FND-P0-002 | 默认所有manager共享单一用户路径；第二个live manager注册同path会递增epoch并使第一个commit stale；dynamic session可并存多个CoreRuntime | 多Runtime、Editor+PIE、并行工具或多session中只有最后激活者能可靠持久化，较早实例仍显示active却持续失败 | 建立process-scoped persistence broker或显式独占/共享scope lease；runtime/session/project地址参与identity，冲突在activation前fail-close，绝不静默supersede live owner。Runtime55 + Runtime03/25 |
| FND-P0-003 | 两个Immediate driver均为空ZST且无人解析；manager依赖为空并直接调用Core；EventManager production consumer为0；Asset/Platform只声明module依赖 | module/capability/load report会把占位名字计为真实driver与foundation服务，产品、诊断和依赖图均可伪绿 | 编译Foundation contract时只允许注册有行为、有owner、有依赖、有health的provider；无外部边界就删除driver，有边界则把实际I/O/transport移入driver。Event/Config必须有真实产品consumer和admission证据后才可标Ready。Runtime55 + Runtime42/46/50 |

## 6. P1 工程化差距

### 6.1 Contract、Module、Driver 与 Service Composition

| ID | 差距 | 目标 / owner |
|---|---|---|
| FND-P1-001 | ConfigManager/EventManager descriptor的service dependency均为空，无法证明backend先于facade ready | compiled dependency绑定真实provider；Runtime46 + Runtime55 |
| FND-P1-002 | 空driver类型与name仍从`foundation`公开导出，形成无行为公共表面 | 删除导出；若保留则提供最小稳定contract、health和真实resolver |
| FND-P1-003 | Kernel级Foundation在factory中启动OS线程和文件I/O，却不声明executor、blocking、filesystem或shutdown capability | descriptor声明资源/线程/affinity/teardown policy；Runtime02/46 |
| FND-P1-004 | Foundation没有`ModuleLifecycle`，最终flush、drain和stop只藏在service Drop | module显式quiesce -> flush -> drain -> stop并返回terminal receipt |
| FND-P1-005 | worker线程spawn成功即等于ConfigManager Ready，后续panic/failed writer没有Degraded/Failed状态 | readiness绑定worker generation、backend health与remediation |
| FND-P1-006 | Asset声明Foundation module依赖，但三个manager/driver没有Foundation service edge或消费点 | 删除假依赖，或声明并消费精确contract |
| FND-P1-007 | Platform声明Foundation module依赖，但真实manager只依赖自身PlatformDriver | 同上；module ordering不得替代service依赖 |
| FND-P1-008 | EditorManager虽声明ConfigManager依赖，Editor host仍并存ConfigManager与Core直写路径 | dependency必须约束实际访问API并禁止旁路 |
| FND-P1-009 | manager常量与descriptor靠字符串拼装/硬编码对齐，测试只比对文字 | 使用compiled contract key，name只做诊断显示 |
| FND-P1-010 | `Built-in runtime foundation services`没有列出scope、provider、durability、delivery或可选性 | capability manifest报告声明值、resolved provider和真实health |
| FND-P1-011 | Foundation无条件进入client/server/editor核心module候选，未说明各target需要哪些能力 | target profile显式选择Config/Event/Persistence能力，未选择项不注册 |
| FND-P1-012 | shader viewer手工激活Foundation只验证调用存在，没有消费或readiness证据 | 工具profile通过同一compiled module plan取得最小能力和load receipt |

### 6.2 Config Contract、Layer、Persistence 与 Lifecycle

| ID | 差距 | 目标 / owner |
|---|---|---|
| FND-P1-013 | public contract是任意`&str -> serde_json::Value` | typed `ConfigKey<T>`、owner namespace、schema id/version |
| FND-P1-014 | `get_value`用`Option`同时表达missing和CoreWeak升级失败 | `Result<ConfigRead<T>, ConfigError>`区分missing/unavailable/invalid |
| FND-P1-015 | contract没有remove/reset-to-default/clear-override | typed mutation覆盖set/remove/reset并返回revision |
| FND-P1-016 | 没有key registry、default、validator、restart/live-apply或secret policy | Runtime03 ConfigRegistry作为唯一声明源 |
| FND-P1-017 | engine/project/user/profile/session/CLI/env全挤进同一HashMap | 显式layer与scope precedence，effective view可解释source |
| FND-P1-018 | 多key更新没有prepare/validate/commit、CAS或revision | 原子transaction发布单一revision和typed delta |
| FND-P1-019 | 磁盘恢复逐key写Core，consumer可观察部分加载状态 | activation前构造不可变snapshot并一次发布generation |
| FND-P1-020 | 文件没有format/key version、migration或unknown-key report | versioned envelope、migration plan、quarantine与forward preserve |
| FND-P1-021 | `ZIRCON_CONFIG_PATH`是进程全局字符串覆盖 | 由host注入typed persistence address，不读隐式全局状态 |
| FND-P1-022 | 无平台目录时回落当前工作目录`.zircon-config.json` | product启动明确决定路径；不可确定路径则禁用durability并报告原因 |
| FND-P1-023 | 默认路径不含project、profile、principal、runtime或session identity | address包含scope identity、tenant和backend |
| FND-P1-024 | worker snapshot整个Core ConfigStore，而非本次transaction的durable key集 | durable projection只含registry授权key与对应revision |
| FND-P1-025 | `CoreHandle::store_config*`是公开持久配置旁路且不推进dirty | 完成调用点迁移后删除，不留compat alias |
| FND-P1-026 | BuiltinEngineEntry用前后两次旁路写模拟precedence | host提交一次source-tagged boot snapshot |
| FND-P1-027 | Editor布局走manager，subsystem/sandbox设置走Core直写 | Editor统一使用typed scope transaction |
| FND-P1-028 | Animation设置继续走Core直写，进程内成功但不保证落盘 | Runtime03 hard cut到canonical authority |
| FND-P1-029 | dynamic session profile通过Core直写，未声明session-only与durability | profile进入immutable session bootstrap layer |
| FND-P1-030 | Config变更没有连接EventManager或独立typed observer | commit后锁外发布revision delta，observer有lease/backpressure |
| FND-P1-031 | production没有`ConfigManager::flush`调用，正常退出只依赖Drop | host/module shutdown显式请求fence并收集terminal receipt |
| FND-P1-032 | `flush(timeout)`只表示当前dirty generation，无durability level或backend分项 | 返回written/data-synced/metadata-synced能力与逐backend结果 |
| FND-P1-033 | persistence report没有path/scope/runtime/backend/generation时间戳 | health snapshot带identity、observed_at、window和owner |
| FND-P1-034 | `pending_flushes`实际只有0/1，`peak_pending_flushes`只会被提升到1 | 改名为pending_generation或真实统计coalesced queue/backlog |
| FND-P1-035 | I/O、parse、timeout、worker panic都折叠为`CoreError::ConfigParse`和String | typed error保留operation、path、source、retryability和disposition |
| FND-P1-036 | 写失败后没有定时/backoff重试，只能靠同值set或flush再次触发 | owner控制bounded retry、jitter、health transition和terminal failure |
| FND-P1-037 | dirty generation用`saturating_add`，耗尽后继续伪装同一generation | checked exhaustion并fail-close；Runtime24 |
| FND-P1-038 | path epoch用`wrapping_add`，可把极旧writer重新变成当前代 | checked/non-reusing identity；Runtime24/25 |
| FND-P1-039 | 进程静态path gate map从不删除dead Weak entry | bounded registry按last-owner回收并报告cardinality |
| FND-P1-040 | `load-if-equal`与`store`分两次锁，没有transaction revision | authority锁内/版本化compare-and-commit，通知在锁外 |
| FND-P1-041 | worker snapshot closure强持有ConfigStore，可在Runtime root消失后继续访问旧store | worker绑定module generation，quiesce后封存不可变final snapshot |
| FND-P1-042 | Drop超时只写tracing并丢弃未退出JoinHandle，没有调用方receipt | Runtime02拥有可取消/可遗弃执行域；Runtime55要求产品shutdown收集结果 |

### 6.3 Event Contract、Authority 与 Product Use

| ID | 差距 | 目标 / owner |
|---|---|---|
| FND-P1-043 | EventManager在production没有解析者，真实消费证据为0 | 至少一个App/Runtime/Editor产品链按contract消费，否则删除service |
| FND-P1-044 | CoreHandle仍公开publish/subscribe，EventManager只是第二层转发 | 确定唯一event authority并删除公开旁路 |
| FND-P1-045 | topic是任意String、payload是任意JSON | typed event id、schema digest/version、owner与bounded codec |
| FND-P1-046 | publish返回`()`，没有sequence、delivery、drop、subscriber或receipt | 返回typed publication result；需要可靠语义的流使用ack/cursor |
| FND-P1-047 | Core不可用时publish静默成功 | 明确`RuntimeUnavailable/Quiescing/Closed`，禁止silent no-op |
| FND-P1-048 | Core不可用时subscribe返回伪造zero-state对象 | resolver/admission直接返回typed unavailable，不制造假订阅 |
| FND-P1-049 | contract没有event generation、producer identity、time、ordering或compat window | envelope声明scope/generation/sequence/clock/schema |
| FND-P1-050 | 没有runtime/world/session/player/view scope或principal/capability | scoped topic registry与访问策略 |
| FND-P1-051 | Config commit、module lifecycle和service health都不发布Foundation typed event | 只在authority commit后发布一致revision事件 |
| FND-P1-052 | EventManager contract不暴露Core已有diagnostics或health | service status提供bounded snapshot、drop/gap和remediation |
| FND-P1-053 | public delivery policy包含无容量的Lossless | Runtime02改为有预算的reliable/ephemeral class并显式overflow |
| FND-P1-054 | UI、Asset、Resource、Scene mirror和Foundation各有独立事件体系，边界/桥接无catalog | event catalog说明每类owner、scope、bridge与禁止跨域路径 |

### 6.4 Tests 与 Qualification

| ID | 差距 | 目标 / owner |
|---|---|---|
| FND-P1-055 | 测试主要证明roundtrip、name文字和root文件结构；App测试自造唯一EventManager consumer | contract/compile-fail/product tests证明真实consumer与旧路径删除 |
| FND-P1-056 | 第二Runtime测试只在第一manager已flush后验证reload；没有两个live manager、boot layer或动态profile回归 | 加入simultaneous multi-runtime、disk-vs-session precedence、whole-store泄漏和restart矩阵 |

## 7. P2 完整性与可维护性差距

| ID | 差距 | 收敛方向 |
|---|---|---|
| FND-P2-001 | `contains_key`通过`get_value`深clone JSON | 提供不复制的presence/read guard或typed snapshot |
| FND-P2-002 | 每次`get_value`深clone完整Value，大布局读取成本随文档增长 | immutable Arc snapshot或typed cached decode |
| FND-P2-003 | worker固定`to_vec_pretty`，增加CPU与写入字节 | canonical compact codec，调试导出另行pretty-print |
| FND-P2-004 | HashMap序列化顺序不稳定，同一逻辑配置可产生不同文件bytes | canonical key order与deterministic digest |
| FND-P2-005 | 每次report都复制并排序最多64个latency样本 | 在线histogram/sketch或预聚合窗口 |
| FND-P2-006 | p95/max没有sample count、window起止和reset generation | 完整metric window metadata |
| FND-P2-007 | `serialized_bytes`包含失败attempt，名称易被误读为durable bytes | 分开attempted/written/committed bytes |
| FND-P2-008 | success/failure等指标用saturating counter且无exhausted标记 | checked或exhausted health bit |
| FND-P2-009 | 每次manager构造重新读取环境/CWD，进程中不同激活可选到不同路径 | host冻结一次typed address plan |
| FND-P2-010 | public `CONFIG_DRIVER_NAME/EVENT_DRIVER_NAME`扩大无用稳定表面 | 删除占位public constants，诊断名从compiled contract生成 |
| FND-P2-011 | EventManager每次publish都把`&str`复制为String | 注册topic id，热路径使用dense id |
| FND-P2-012 | subscribe总是分配`Box<dyn EngineEventSubscription>` | typed/generation cursor handle或pool-backed lease |
| FND-P2-013 | disconnected subscription在Foundation重复手写Core EventBus状态语义 | availability在resolver边界统一表达，不复制实现 |
| FND-P2-014 | source-string结构测试易被格式/重排影响，且不证明行为 | AST/compile contract与运行时状态测试 |

## 8. 参考引擎对照与适用边界

| 参考 | 已核对机制 | Zircon应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal | `FConfigBranch`明确static/dynamic/saved/runtime-change layers、hierarchy、command-line override、async load completion与Flush；`IModuleInterface`区分Startup/PreUnload/Shutdown和动态卸载支持 | layer/source、runtime change、load readiness、显式module teardown顺序 | 不复制全局`GConfig`、INI语法或C++单例/DLL模型 |
| Godot | ProjectSettings记录initial/current、persist/basic/internal/restart-if-changed、order、version、changed set、feature override和save/setup | key metadata、change revision、restart policy、feature/layer override、可解释保存 | 不复制Object singleton和Variant全局表作为Zircon最终类型系统 |
| Bevy | Plugin具有build/ready/finish/cleanup；Message按Rust类型注册，双buffer保留窗口，reader持独立cursor且可报告missed messages | typed registration、ready/finish边界、per-consumer cursor、明确retention/missed | Bevy message的两帧保留不是跨线程可靠事件流，也不替代持久化broker |
| Fyrox | ResourceEvent是typed enum，subscriber返回generation handle，可显式remove并自动清理dead sender；EditorPlugin有on_start/on_exit | typed domain event、对称订阅生命周期、dead subscriber回收、明确插件退出 | std mpsc广播仍缺少Zircon所需预算、ack、多Runtime与原生卸载证明 |
| Unity Graphics | DebugWindow在OnEnable订阅dirty callback，在OnDestroy对称撤销；DebugManager RegisterData/UnregisterData成对 | consumer生命周期必须对称、可观察并绑定真实窗口/owner | DebugManager singleton和Editor callback不作为通用Runtime event/config权威 |

## 9. 目标架构

```text
Host / Dynamic Session
  -> BootConfigSnapshot
       { source, scope, profile, project, principal, typed values }
  -> FoundationContractCompiler
       -> ConfigRegistry + layer precedence + validation/migration
       -> EventCatalog + schema/scope/delivery class
       -> resolved provider capabilities
  -> CompiledFoundationContract
       -> ConfigAuthority
            immutable effective revision
            transactional mutation + typed delta
            durable projection only
       -> ScopedPersistenceBroker
            address/lease/CAS/recovery/durability receipt
       -> TypedEventService
            topic id + generation + cursor/ack/drop state
       -> lifecycle state
            activate -> ready -> quiesce -> flush/drain -> stopped
```

关键硬切规则：

1. `core::framework::foundation`只保留纯合同、typed DTO与identity，不放具体worker、文件路径或EventBus实现。
2. 没有独立外部边界的driver直接删除；确有文件/transport provider时，manager必须通过descriptor依赖并调用它。
3. boot/session override与durable user/project配置使用不同类型和地址，不能共享一个raw HashMap后靠调用顺序覆盖。
4. `ConfigAuthority`是唯一持久配置写入口；`CoreHandle::store_config*`完成迁移后删除。
5. `TypedEventService`是唯一Foundation事件入口；Core EventBus可作为内部provider，但不再暴露第二套public authority。
6. 多Runtime要么共享一个process broker及独立scope view，要么以显式exclusive lease在activation前拒绝；不得用epoch暗中杀死live owner。
7. manager调用期lease、native unload和worker执行域仍由Runtime01/02/50收敛，本篇不另建平行kernel。

## 10. 分层重构里程碑

### M0 · Contract Freeze 与 RED Gates

- 写compile-fail/API测试冻结typed key/event、scope、source、revision、provider capability和terminal receipt。
- 写四个必失败模型：disk覆盖session profile、whole-store泄漏、两个live runtime同path、empty driver伪Ready。

### M1 · Foundation Descriptor Truth

- 编译Foundation contract并删除空ConfigDriver/EventDriver及public name。
- Asset/Platform/Editor依赖改为精确service capability；无消费边删除module dependency。

### M2 · Boot Config Hard Cut

- App和dynamic session构造一次`BootConfigSnapshot`，在任何consumer activation前完成layer compose。
- 删除BuiltinEngineEntry两次Core直写与dynamic session pre-activation raw store。

### M3 · Typed Config Authority

- 落地registry、scope/layer、typed read、transaction、revision、delta、migration和effective-source diagnostics。
- 迁移Editor layout/subsystem、Animation、Platform/Window/Render profile，删除Core config旁路。

### M4 · Scoped Persistence Broker

- 引入typed address、owner lease、durable projection、CAS/lock/recovery与backend capability。
- 支持多个Runtime/PIE/session并存，显式共享或隔离，不发生silent supersession。

### M5 · Typed Event Service 与 Product Consumer

- 将Core EventBus收进内部provider，建立typed event catalog、publication result、cursor/gap和health。
- 至少接通一个真实App/Runtime/Editor纵向消费者；删除测试专用EventManager facade或旧Core入口。

### M6 · Lifecycle 与 Shutdown Receipt

- Foundation module显式quiesce Config/Event、阻止新调用、flush/drain worker和subscriber、返回逐provider terminal receipt。
- worker timeout、panic、stale manager和native unload按Runtime02/50统一执行域处理。

### M7 · Product、Failure 与 Performance Qualification

- 运行Editor+PIE、多dynamic session、client/server、project/profile/principal、restart/migration/fault矩阵。
- 在正确性门通过后测1/8/64 Runtime、1K/100K keys、1/1K topics、subscriber storm、real disk、RSS和p95/p99；与参考引擎只做同场景同语义比较。

## 11. 验收门禁（40项）

| Gate | 验收内容 |
|---|---|
| FND-G01 | compiled Foundation contract拒绝duplicate/missing/wrong-contract provider |
| FND-G02 | descriptor中不存在无行为driver或无消费capability |
| FND-G03 | Asset/Platform/Editor每条Foundation依赖都能追踪到精确service调用 |
| FND-G04 | target profile未选择的Foundation capability不注册、不报Ready |
| FND-G05 | boot config在任何module consumer前形成单一effective revision |
| FND-G06 | disk/user layer不能覆盖更高优先级session/CLI显式值 |
| FND-G07 | session-only render/window/platform值永不进入durable projection |
| FND-G08 | Config key有type/schema/version/default/validator/scope/source |
| FND-G09 | 多keytransaction要么全提交一个revision，要么无变化 |
| FND-G10 | remove/reset/migration/unknown/invalid均有typed result |
| FND-G11 | config change observer只收到已commit revision且在锁外运行 |
| FND-G12 | CoreHandle公开config写旁路物理删除，无alias/shim |
| FND-G13 | Editor、Animation、App、dynamic session全部迁移canonical authority |
| FND-G14 | 两个live Runtime共享路径时按声明策略共享或activation fail-close |
| FND-G15 | 第二Runtime不能使第一Runtime后续commit静默stale |
| FND-G16 | project/profile/principal/session scope地址互不污染 |
| FND-G17 | same-process、cross-process和symlink/case alias冲突矩阵通过 |
| FND-G18 | dirty/revision/epoch/ticket exhaustion显式失败且不复用身份 |
| FND-G19 | worker panic/timeout/backend failure进入Degraded/Failed health |
| FND-G20 | flush返回声明的durability level与逐backendterminal receipt |
| FND-G21 | module shutdown显式quiesce、flush、drain、stop，不只依赖Drop |
| FND-G22 | shutdown deadline后没有可越过module generation的迟到commit |
| FND-G23 | path gate/registry最后owner退出后回收，无无界Weak key增长 |
| FND-G24 | malformed/old/future config产生migration/quarantine/diagnostic证据 |
| FND-G25 | Event type在catalog注册schema/scope/owner/version/delivery class |
| FND-G26 | publish返回sequence/result，runtime unavailable不静默成功 |
| FND-G27 | subscriber持generation cursor并能报告missed/drop/gap |
| FND-G28 | reliable类有明确容量、ack、overflow和resync策略 |
| FND-G29 | Event service diagnostics暴露topic/subscriber/queue/drop/health |
| FND-G30 | CoreHandle公开event旁路物理删除或降为不可外用内部provider |
| FND-G31 | 至少一个真实产品消费者通过Foundation event contract闭环 |
| FND-G32 | window/module/session consumer注册与撤销完全对称 |
| FND-G33 | Config commit event携带相同revision，不会先发后写 |
| FND-G34 | manager call guard与provider lease覆盖quiesce/unload竞态 |
| FND-G35 | App、Editor、dynamic session、server profile cold/warm restart通过 |
| FND-G36 | simultaneous Editor+PIE+tool Runtime无配置或事件串扰 |
| FND-G37 | 1K/100K key与1/1K topic基准报告CPU/allocation/lock/RSS/p95/p99 |
| FND-G38 | slow disk、full disk、permission、panic、hung writer、drop storm故障注入通过 |
| FND-G39 | 同硬件同语义比较同时满足正确性、失败和性能门，不只报平均值 |
| FND-G40 | frontmatter路径、链接、索引、coverage、计数、fingerprint与`git diff --check`复核通过 |

## 12. Owner 去重与状态

| Owner | 本报告回写 | 不得改写为 |
|---|---|---|
| Runtime01/02 | Foundation shutdown必须使用canonical lifecycle/execution domain；detached worker和EventBus内部调度由其修复 | Runtime55另造线程池或重复登记DLL-unload P0 |
| Runtime03 | typed ConfigRegistry、layer/schema、Core direct-write hard cut | 现有dirty worker已经等于完整Config系统 |
| Runtime24/25 | checked identity、path/VFS、atomic recovery、跨进程lock/CAS | lowercase path或atomic rename已解决identity/durability |
| Runtime42/46 | target module catalog、compiled descriptor、capability truth和module lifecycle | driver/manager count大于0即能力Ready |
| Runtime45 | Preference backend、scope overlay、multi-process durability与Editor settings owner | Foundation raw config替代完整Preference服务 |
| Runtime50 | typed service directory、call guard、provider lease、stale generation | resolver返回Arc后仍可无guard长期调用 |
| Runtime43/App01 | dynamic session与product host创建/关闭多个Runtime并呈现terminal result | 单元测试new CoreRuntime等于产品闭环 |
| Runtime55 | Foundation纵向组合、boot precedence、durable projection、多Runtime path ownership、真实Config/Event consumer与旧入口删除 | 再包装一层facade、保留空driver或靠调用顺序覆盖 |

| 项目 | 状态 | 说明 |
|---|---|---|
| Runtime55 review | `review_complete` | 3 P0 / 56 P1 / 14 P2 / 40 gates |
| Production重构 | `pending` | 未修改源码、测试、Cargo、ABI或产品行为 |
| Open config failure | `open / validation_pending` | 保留既有静态修复结果，不提前关闭 |
| Dynamic/performance validation | `not_run` | 未运行Cargo、双Runtime、Editor、DLL、fault、soak或benchmark |
