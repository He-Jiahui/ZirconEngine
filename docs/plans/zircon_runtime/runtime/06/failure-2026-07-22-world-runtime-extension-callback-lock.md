---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-runtime-extension-callback-lock
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/plugin/extension_registry
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
tests:
  - cargo test -p zircon_runtime --lib scene --locked --jobs 1 -- --nocapture --test-threads=1
  - concurrent world creation with slow and reentrant extension callbacks
  - concurrent plugin scene systems use per-World callback state without shared mutex wait
---

# Runtime06：world runtime extension callback全局锁交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene root/runtime extension/runtime hook 9/9逐Rust文件性能审查，PERF-MVP-451
- 修复责任计划：`docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md`
- 交接原因：Runtime06拥有plugin lifecycle与reload/unload quiescence，Plugins01共同提供extension registry generation；最低根因不能由scene call site局部解锁解决。
- 共同验收：Plugins01负责extension registry generation与插件reload/unload生命周期语义
- 生命周期键：`world-runtime-extension-callback-lock`

## 失败现象与复现证据

`WorldDriver::apply_world_runtime_extensions`持有`runtime_extensions` mutex guard并在锁内遍历、执行全部type-erased `Fn(&mut World)` callback。`level_manager_lifecycle`在每次World创建调用该入口；因此一个慢插件会串行阻塞同一driver的其他World初始化，callback重入install/apply边界可能自锁，callback wall time直接落在主线程F0/F2 bootstrap。

本轮只把plan唯一性校验的key clone改为borrowed set，并把scene hook identity查重从二次扫描改为HashSet；没有用“每次apply深clone完整plan”掩盖callback锁根因。

2026-07-22 plugin control-plane逐文件审查补充确认第二层同根串行：`SystemRegistrationBuilder`与`RuntimeSceneSystemRegistrationBuilder`把注册时FnMut封入`Arc<Mutex<S>>`，随后为每个World构造的system实例都clone同一Arc，并在每次schedule run期间持锁执行plugin callback。即使WorldDriver外层锁移除，多个World/worker仍会在callback state上全局串行；对应PERF-MVP-532。

## 最低共享层根因

WorldDriver只保存`Mutex<WorldRuntimeExtensionPlan>`可变authority，没有可在短锁内取得、锁外执行的immutable generation handle。Plugin registry与scene owner也没有共同冻结install/reload/unload期间旧generation的in-flight lifetime。

同时，registration contract没有per-World factory/state owner，只能通过共享FnMut mutex维持可变callback；这把执行期锁与reload/unload生命周期耦合，不能由schedule call site局部解锁。

## 架构修复验收

- Runtime06发布`Arc`持有的immutable ordered registration generation；WorldDriver在短锁内只snapshot handle，所有callback锁外执行。
- install/merge先构建候选、验证identity/order，再原子发布；失败不改变当前generation，旧generation由in-flight Arc自然延寿。
- reload/unload必须定义quiescence：新World只见新generation，已开始apply的World可安全完成旧snapshot，不持registry/world-driver全局锁等待foreign callback。
- native/runtime scene system由generation factory为每个World创建独立callback state；每次run只借用实例私有`&mut S`，不同World不共享callback mutex，SystemParam访问冲突仍由schedule决定。
- 1/8/64并发World创建 × 0/10/1000ms callback与重入fixture记录mutex wait/hold、callback-in-lock wall、generation/build count和F0/F2 p95；callback-in-lock=0、无死锁、顺序/错误/rollback等价。
- 1/8/64 Worlds运行同一plugin system时记录shared callback mutex acquire/wait/hold=0、worker overlap与reload age；同World顺序、panic/error与state continuity等价。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止每次World创建深clone全部descriptor/String作为最终方案；禁止换成读锁后仍在guard内执行callback。
- 禁止在未声明World访问与thread affinity前并行执行extension callback。
- 禁止同时保留mutex plan与Arc generation两套authoritative registration truth。

## 当前前向修复状态

- `WorldDriver` 现保存 `Mutex<Arc<WorldRuntimeExtensionPlan>>`；apply 仅在短锁内克隆 generation handle，随后在锁外执行 callback。`world_runtime_extension_callback_can_publish_a_new_generation` 与 `world_runtime_extension_callbacks_overlap_across_independent_worlds` 覆盖 reentrant publication 和跨 World 并发。
- plan install 先以 `try_merge` 构建和校验候选，再以新的 `Arc` 一次发布；失败候选不会替换已发布 generation，已开始 apply 的旧 generation 由持有的 `Arc` 自然延寿。
- typed、external 与 runtime scene system builders 现在在每次 build 时克隆 callback template，并将该实例放入新建 system；对应私有状态和跨 World overlap 回归已存在，不再使用 `Arc<Mutex<S>>` 共享可变 callback state。
- 本次只读源码审计确认以上结构和回归存在；受管 Cargo、1/8/64 压力矩阵及 reload age 的终态证据尚未产生。因此 failure 保持 `open`，不得以静态审计替代动态验收。

## 修复结果与回传

Open state: `静态架构修复已集成，等待受管动态并发/生命周期验收`; no dynamic pass is claimed.
