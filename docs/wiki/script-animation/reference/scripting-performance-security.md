---
related_code:
  - zircon_runtime/src/script/vm/plugin/management_policy/memory.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/garbage_collection.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery/limits.rs
  - zircon_runtime/src/dynamic_api/bounded_json
  - zircon_runtime/src/navigation/repath_budget.rs
implementation_files:
  - zircon_runtime/src/script/vm/plugin/management_policy
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery/limits.rs
  - zircon_runtime/src/dynamic_api/bounded_json
  - zircon_runtime/src/navigation/repath_budget.rs
plan_sources:
  - user: 2026-09-09 扩展脚本、反射、动画与导航公开接口文档
tests:
  - zircon_runtime/src/script/vm/tests
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
doc_type: best-practice
---

# 脚本、动画与导航的性能和安全基线

## 分层预算

引擎同时限制四类成本：插件发现/载入、VM 内存与 GC、动态 API payload、导航 repath。预算不是“建议值”，而是防止恶意或失控脚本拖垮主循环的合同。

| 域 | 默认/公开控制 | 超限行为 |
| --- | --- | --- |
| plugin discovery | depth 16、entries 16384、manifest 1024、manifest 256 KiB | discovery error |
| bytecode cache | entries 1024、bytes 256 MiB | 拒绝或淘汰缓存 |
| VM GC | `DEFAULT_VM_GC_MAX_MICROS_PER_FRAME` | 分步 GC，保留 root |
| bounded JSON | depth/bytes/items/deadline | `BoundedJsonError` |
| nav repath | `NavRepathBudget` 默认 32/frame | 延迟到下帧 |

## 管理策略 API

```rust
let policy = VmPluginManagementPolicy::default()
    .with_memory(VmPluginMemoryPolicy::with_limits(
        Some(64 * 1024 * 1024), Some(128 * 1024 * 1024)))
    .with_garbage_collection(VmPluginGarbageCollectionPolicy::cooperative(Some(4)));
policy.validate()?;
```

`VmPluginHotReloadPolicy`、`VmPluginGarbageCollectionMode` 和 memory limits 必须在加载前 validate。soft limit 用于诊断/回收，hard limit 用于拒绝分配；不要把 hard limit 当作可自动扩容。

## GC root 与句柄

VM 对象通过 `VmObjectRef`/`HostHandle` 引用；跨帧保存引用必须注册 `VmGcRootToken`。每帧 GC 使用 `VmGcBudget`，读取 `VmGcStepReport`/`VmGcDiagnostics` 监测耗时和未回收对象。热重载时先 quiesce callback，再撤销 root，最后执行 GC。

## ABI 防护

动态 API 所有输入先做 UTF-8、长度、深度、deadline 预检。回调 panic 使用 `catch_native_callback_panic` 转为 status。插件发现限制路径、manifest 和 bytecode 大小；不得按用户输入拼接 DLL、WASM 或脚本路径。

## 动画和导航成本

动画 graph evaluation 应缓存 compiled graph、参数 map 和 track path；mask target 去重使用 HashSet。导航只在目标或环境改变时 repath，配合 budget 和 cursor；每帧对所有 agent 强制 find_path 会产生尖峰。脚本更新动画参数应批量写入 World，避免 entity 级 host call 往返。

## 线程模型

只读资产和 graph 可在 worker 线程评估；World、host registry、reflection catalog commit 只能在所有权明确的调度阶段修改。禁止跨线程发送裸 `&mut World`、`wgpu` 对象或未刷新 generation 的 callback handle。

## 安全审计清单

- [ ] package owner、capability、backend selector 全部白名单校验。
- [ ] payload 有字节/深度/时间预算。
- [ ] 脚本错误不会吞掉 host/World 锁 poison 或 panic。
- [ ] 状态 blob 与反射 schema 版本匹配。
- [ ] navigation/animation 预算进入 telemetry。
- [ ] 生产构建关闭任意脚本文件加载和调试导出。

## 负例

```text
不要把 discover_vm_plugin_packages 的 limits 设为 usize::MAX 来“兼容大项目”；
应提升单项上限并同步 telemetry、缓存策略和安全评审。
```

## 源码与测试

- [memory policy](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/plugin/management_policy/memory.rs)
- [GC policy](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/plugin/management_policy/garbage_collection.rs)
- [discovery limits](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery/limits.rs)
- [bounded JSON](https://github.com/He-Jiahui/ZirconEngine/tree/main/zircon_runtime/src/dynamic_api/bounded_json)
- [nav budget](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/navigation/repath_budget.rs)

## 性能预算落地

为每帧建立 budget ledger：`script_us`、`gc_us`、`animation_us`、`navigation_us`、`dynamic_api_bytes`。超过阈值时优先降级非关键脚本系统、延迟 repath、减少 diagnostics，而不是跳过安全校验。

## 代码审查问题

1. 是否在循环内重复 resolve module/function？
2. 是否把字符串路径转换放到每帧热路径？
3. 是否持有 World borrow 调用 VM 或 IO？
4. 是否对脚本浮点、数组、递归深度做边界检查？
5. 是否在 reload 失败时恢复完整 generation？

## 压测场景

| 场景 | 指标 |
| --- | --- |
| 1000 agents | repath p95、blocked count |
| 256 graph nodes | evaluation allocations |
| 64 MiB state blob | migration wall time |
| malformed JSON | reject latency、无 panic |
| 8 concurrent reloads | stale commit rate |

## 发布门禁

- 禁止未签名/未 allowlist 的 plugin package。
- 所有公开 host export 生成 capability 清单。
- 运行 fuzz/负例测试覆盖 bounded JSON、schema、handles。
- 记录 backend、profile、limits 和 feature flags 到 build manifest。
