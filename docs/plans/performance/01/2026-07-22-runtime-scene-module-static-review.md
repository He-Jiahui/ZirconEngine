---
related_code:
  - zircon_runtime/src/scene/module
  - zircon_runtime/src/scene/tests/mod.rs
  - zircon_runtime/src/script/vm/reflection/catalog.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
reference_sources:
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/godot/core/io/resource_loader.cpp
tests:
  - zircon_runtime/src/scene/module/level_display_name.rs
  - zircon_runtime/src/scene/tests/mod.rs
  - current-source Windows zircon_runtime scene tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene module逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/module/**`当前源 **8/8** 个Rust文件、**596** 行、**2** 个inline tests已逐文件阅读，并完整读取118行root `scene/tests/mod.rs`以增加锁域源码守卫。范围覆盖SceneModule descriptor/facade、DefaultLevelManager registry、World创建/遍历/VM type同步、project load/save与WorldDriver tick/hook/extension装配。`module/mod.rs`已有共享工作区的`SCENE_MODULE_NAME` owner迁移，本切片只读保护，没有覆盖。

## 已直接修复

- `LevelManager::level_exists`原先调用`self.level(handle)`，仅为bool却clone一次含多个Arc的`LevelSystem`。现直接在短registry锁内`contains_key`。
- `try_create_level`原先在driver resolve及全部runtime extension callback之前取得`levels` mutex，并一直持有到World构造和insert；慢/重入plugin callback会同时阻塞level exists/summary/query/create。现先完整prepare World，最后只在publish insert时取得registry锁；outer level-registry callback-in-lock已清零。WorldHandle的AtomicU64仅承担唯一id分配，ordering由`SeqCst`收敛为`Relaxed`，不再制造无消费者的全序栅栏。
- `try_for_each_world`原先持有registry mutex逐个取得World锁并执行caller operation。现短锁snapshot cheap-Clone `LevelSystem` Arc handles，释放registry后再进入World与callback，避免reflection prepare把level registry锁域放大到全部World验证。
- 源码守卫先在旧源观察RED，修复后GREEN；scoped rustfmt与diff check通过。current-source受管Cargo取得测试lane时被`plugins01-native-callback-stable-owner-r1-20260722`精确预约，本轮没有启动Rust测试。`sync_vm_types_atomically`仍有意持有registry并锁定全部目标World：当前它依赖冻结level集合才能保证catalog commit与rollback原子性，不能在PERF-MVP-446的generation protocol落地前局部释放。

## 仍需架构修复

`LevelManagerContract::{load_level_asset,save_level_asset}`每次都从`project_root: &str`重新`ProjectManager::open`并同步`scan_and_import`完整工程；save即使只写一个scene也先全量扫描。`save_world/save_level`为避免持World锁跨I/O而深clone完整World，再在caller线程同步serialize/create_dir/write。大工程或大scene的频繁save/load会把目录遍历、import、World clone和磁盘I/O直接落到F0/F4主线程。

此项登记PERF-MVP-453并复用Runtime04既有`project-source-index-targeted-import` handoff：LevelManager合同须消费manager-owned prepared project generation/targeted source transaction，save发布immutable scene snapshot/artifact ticket并由bounded I/O lane原子写入；不能在scene facade维护第二份ProjectManager cache。World VM type prepare/commit的全World验证、全锁与rollback深clone继续归PERF-MVP-446。

WorldDriver内部runtime-extension mutex跨callback问题已单独登记PERF-MVP-451。本轮移除的DefaultLevelManager outer registry锁只是局部止损，不能据此关闭该handoff。

## 参考引擎对照

Bevy `AssetServer`保存跨请求复用的asset info/loader状态并以AssetPath加载，不要求consumer每次重建整个工程扫描；Godot `ResourceLoader`提供`CACHE_MODE_REUSE`、ResourceCache与threaded request token复用。可迁移原则是“project/resource generation由资产owner发布，scene consumer持handle”，而不是在LevelManager里新增隐式全局cache。

## 动态验收

1. current-source scene focused Cargo：level create/exists、extension failure、World iteration、VM type atomic rollback、load/save与完整scene library tests。
2. 1/8/64并发create/exists/summary，extension callback为0/10/1000ms：记录level-registry wait/hold、atomic contention与F0/F2 p95；本轮outer registry callback-in-lock必须为0，handle唯一且发布后可见。
3. 1/100/10k Worlds运行reflection prepare/commit：记录registry/world lock hold、World clone bytes/rollback RSS；PERF-MVP-446完成前不得宣称atomic sync可扩展。
4. 1/10k/100k project files、scene 1/64MiB的load/save、cold/warm/unchanged：记录project opens/scans/imports、World clone bytes、main/worker time、writes与F4 p95；PERF-MVP-453完成后warm prepared generation open/scan=0、单scene save不全量import、主线程I/O=0且内容/错误/atomic publish等价。

动态验收未完成，因此该目录继续保留在`pending.md`，不进入`review.md`。
