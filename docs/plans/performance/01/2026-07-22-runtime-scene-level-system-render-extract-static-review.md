---
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/level_system
  - zircon_runtime/src/scene/render_extract
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
tests:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - current-source Windows zircon_runtime level/render extract tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene LevelSystem/render extract逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/{level_system.rs,level_system_render_extract.rs,level_system/**,render_extract/**}`当前源 **5/5** 个Rust文件、**494** 行、**3** 个就地test已逐文件阅读；范围包含World/metadata/lifecycle/runtime state锁、runtime system take/run/restore、animation pose与playback缓存、physics frame结果、script started state、World/Level RenderExtractProducer入口。

## 可直接止损但因共享租约未改

`script_binding_started(entity, &str)`为查询`BTreeSet<(EntityId,String)>`每次调用`binding_key.to_string()`，Fixed/Update按binding调用会稳定分配。内部改为`BTreeMap<EntityId,BTreeSet<String>>`即可让`BTreeSet<String>::contains(&str)`借用查询并缩小到单entity范围，公开API无需变化。本轮申请`level_system.rs`写租约时发生精确冲突，说明另一活动会话正在修改该文件；因此只把script lookup alloc=0写入PERF-MVP-469验收，没有覆盖共享源码。

## 单runtime-state mutex与frame payload复制

`WorldRuntimeState`把physics、animation与script三个独立更新域塞进同一Mutex。render extract先锁runtime state，把全部`AnimationPoseOutput`深clone到Vec，随后持World mutex运行完整scene extract并逐pose查node/skeleton；animation playback getter同时clone三张BTreeMap，physics contacts/triggers getter复制完整Vec，script started查询也争用相同锁。稳定帧、多camera、physics/editor观察者和script tick会彼此串行，锁持有/clone bytes随payload规模增长。

PERF-MVP-469由Runtime07承接：按域revision发布immutable/sealed frame handles，simulation owner原位或clear/swap复用，render短锁取一次handles后锁外过滤；frame seal只汇合generation，不把三域正文重新复制进单一struct。animation/physics/script具体增量仍分别复用PERF-MVP-439/335/442，避免维护第二套consumer cache。

## 其他路径归属

- `World::to_render_frame_extract`与`RenderExtractProducer for World`均为运行mutable RenderExtract systems先clone完整World，继续归PERF-MVP-349；LevelSystem production入口持真实World锁避免该clone，但锁域仍需469/349共同缩短。
- `LevelSystem::snapshot`深clone World用于保存/operation，继续归PERF-MVP-453/467；不能把frame snapshot方案误用于I/O transaction。
- runtime scene system先短锁take system、锁外run、再短锁restore，未发现callback跨World mutex；该边界保留。
- metadata/lifecycle/subsystem clone只在control-plane调用，未确认独立frame热点。

## 参考引擎对照

Bevy render extract以typed ECS query访问匹配component并把app/render world边界显式化，独立资源/组件访问由系统调度声明；它没有用一个“所有子系统frame state”互斥锁序列化physics/animation/script。Zircon应采用按域owner与sealed generation handle原则，同时保留LevelSystem作为组合门面，不复制Bevy的unsafe storage API。

## 动态验收

1. current-source Cargo：poison recovery、world replace generation/reset、runtime system restore、animation pose filtering/skeleton、physics record/reset、World/Level render extract parity。
2. poses/events/bindings 0/1k/100k、cameras/readers/threads 1/8/64、stable/1% change记录runtime/world mutex wait/hold、pose/event/map/String clone bytes、extract builds与p95。
3. PERF-MVP-469要求跨域互斥=0、锁持有不随payload bytes增长、stable payload deep clone=0、script lookup alloc=0；349要求World clone bytes=0且stable scene artifact不重建。

受管Cargo仍被共享CPU预约阻塞，并发/规模counter与F2/F4产品trace未完成，因此本切片继续保留在`pending.md`，不进入`review.md`。
