---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/settings
  - zircon_editor/src/core/recovery
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
tests:
  - tools/tests/test_editor14_interactive_save_job_adapter_contract.py
  - tools/tests/test_editor17_recovery_test_ownership_contract.py
  - tools/tests/test_editor17_settings_owner_modules_contract.py
  - tools/tests/test_editor12_settings_page_contribution_contract.py
  - tools/tests/test_runtime_job_system_audit.py
doc_type: implementation-evidence
status: static_current_dynamic_blocked
---

# Editor Jobs / Settings / Recovery currentness复验（2026-08-23）

## 当前清单与结论

| 模块 | current Rust | 行 / bytes / tests | path+raw SHA256 | currentness结论 |
|---|---:|---:|---|---|
| `core/jobs/**` | 47/47 | 9,083 / 297,549 / 108 | `8df93fad07c7777f5de0a916561825c028c7da0f5c8269f50fc31db2aa158ac1` | 8月15/16架构结论不变 |
| `core/settings/**` | 16/16 | 3,937 / 135,386 / 34 | `ec68baa6344bdc4dc4b9a58b782b4f2513863e94acf6d75a267a26d2a1d41180` | 8月15架构结论不变 |
| `core/recovery/**` | 20/20 | 4,890 / 166,711 / 54 | `86f33fc3baed7534fe6c8d91fba0e561343877763db172eef869d53d54a920fc` | 8月15架构结论不变；测试owner门新增失败 |

复验方法是以前次逐文件全文审查为基线，核对8月15日后相关提交的完整diff，再逐行复读所有发生生产语义漂移的文件。三棵树当前无工作区diff；本轮未修改其Rust源码。

## 漂移逐文件核对

### Jobs

`08094b9b9`触及15个Jobs文件，其中生产文件的current差异如下：

| 文件 | 复验结果 |
|---|---|
| `mod.rs` | re-export排序，无运行时变化。 |
| `system/admission_reservation.rs` | 测试import排序，无生产变化。 |
| `system/lifecycle.rs` | 新增`cfg(test)` category admission snapshot并放宽测试helper可见性；生产pump/cancel/shutdown/join不变。 |
| `system/pending.rs` | import排序，无算法变化。 |
| `system/progress_observer.rs` | import排序；observer仍由completion侧同步排队/交付，旧worker callback风险未关闭。 |
| `system/state.rs` | import和测试格式变化；terminal retention、dependency/promotion状态不变。 |
| `progress/primary_generation_tests.rs`及9个外部测试文件 | 测试可见性、断言/格式与路径收束；未增加product generation consumer。 |

因此既有P0继续成立：Editor仍在Runtime11之上维护第二套dependency/quota/promotion scheduler；accepted lifecycle/event bytes没有总 admission；completion仍可能让Runtime worker执行observer/promotion工作。P1也不变：retained status没有消费`primary_snapshot_if_changed`，stable tick仍clone/format；公共`wait`/`join`没有named-thread拒绝。

### Settings

8月15日后只有`tests/registry.rs`变化：修正`SettingChange` owner路径及`Option<SettingChange>`断言。`authority.rs`、`registry.rs`、`persistence.rs`、`io.rs`等生产文件哈希漂移来自8月16日提交对当时已审workspace的收束，而当前算法与原报告一致。

开放P0保持：同一物理settings文件仍按key提交多个持久化任务；完整encode仍在authority/project锁内；receipt只命名key revision而非file generation；stable retained projection仍每帧获取settings/锁并可能重建派生状态。目标仍是每文件一个latest-generation durable lane、锁外encode、affected-mask immutable publication和stable `O(0)`消费。

### Recovery

生产漂移只有`autosave_adapter.rs`把`RuntimeForeignOutputBudget`导入统一到`zircon_runtime_host::foreign_output`，没有改变admission或payload行为；`tests/autosave_adapter.rs`的大量变化为rustfmt。原P0全部保持：autosave/restore/heartbeat产品链未接通；due时全dirty/toolkit/path projection与`O(W*D)`查找；请求struct估算不覆盖真实payload；每snapshot多次目录扫描；project generation fence缺失。

本次源码合同暴露结构验证债务：`recovery/tests.rs`为802行、`recovery/tests/autosave_adapter.rs`为998行，超过800行owner门。下一次实现需按feature owner拆到folder-backed tests，不能继续向两个聚合文件叠加；这是测试维护阻断，不等同于产品性能已退化。

## 参考引擎约束未改变

- Unreal `TaskGraphInterfaces.h`以单一queue interface、named thread和completion trigger表达任务所有权；Jobs目标仍是Runtime11单一TaskGraph加Editor typed facade，而不是继续嵌套两套scheduler。
- Unreal `ConfigCacheIni.cpp:2871-2910,3220-3235`在文件dirty且生成内容变化时才写入并在成功后清dirty；Settings应以file generation合并burst并跳过unchanged durable bytes。
- Unreal `PackageAutoSaver.cpp:1175-1218,1288-1300`通过dirty callbacks维护增量集合，并以bounded backup slot推进；Recovery应消费Editor03 dirty delta和O(1) manifest/slot，而不是due时重建全量状态。

这些源码建立ownership和复杂度基线，不提供Zircon动态耗时或功耗值。

## 验证结果与动态门

- `rustfmt --edition 2021 --check`：Jobs 47/47、Settings 16/16、Recovery 20/20通过；scoped `git diff --check`通过。
- 五个Python模块合计执行13 tests，出现3条failure记录：Recovery两个超长test owner；Runtime11全局audit发现`mesh_sdf_cook/cook.rs`及Graphics两个parallel encoder/builder文件进入bare-thread清单，超出本切片owner。其余断言未报告失败。
- 未运行Rust/Cargo。当前managed validator session已归档，不能以raw Cargo或伪造identity绕过；Rust tests中的`std::env::temp_dir()`也可能落C盘，因此本轮没有执行。
- WPR/xperf/allocator/RSS/package power与F0/F4均未测；没有current-source可执行文件，所以RenderDoc不适用于这三个CPU/I/O模块。

三个模块继续留在`pending`。动态接受仍要求：Jobs storm/observer stall和single TaskGraph计数；Settings key burst到file generation、lock/I/O counters；Recovery dirty 0/1/10K、payload 1 KiB/64 MiB/1 GiB、目录3/1K/100K、project switch/crash矩阵；最后在同机同场景下记录CPU、RSS、wakeups、file I/O、package power及p50/p95。
