---
title: Runtime Foundation Current Source Performance Adoption
date: 2026-08-23
scope:
  - zircon_runtime/src/foundation
status: static_complete_dynamic_pending
source_fingerprint: fbe13d3db998feedef515c1501946f39554c81094c8ec9afa3d6469e87d13afc
canonical_owner:
  - docs/plans/optimize/zircon_runtime/99s-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-current-source-review.md
related_failure:
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-18-config-manager-synchronous-full-file-rewrite.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp
---

# Runtime Foundation Current Source Performance Adoption

## 1. 验收边界

本轮逐文件复读 `zircon_runtime/src/foundation/**` 当前 **14/14 个 Rust 文件、2,138 physical lines / 1,909 non-empty lines、69,268 bytes、23 个 test marker**。目录 clean，当前树相对 Optimize118 的 `baseline_head=9fee3ea0435961a81c85aa2502e64f1f357345d7` 没有源码差异；按“相对路径 + NUL + 原始 bytes + NUL”的有序 SHA-256 为 `fbe13d3db998feedef515c1501946f39554c81094c8ec9afa3d6469e87d13afc`。

Optimize118 已在同一 current working tree 对 Foundation contract、Core config/event owner、App/Editor/plugin/dynamic session 调用链和五引擎参考完成更宽的 E3 审查。本报告只把其结果纳入 performance 执行队列并补充直接源码复验，不复制 P0/P1 账本，不建立第二个 implementation owner。

## 2. 当前结论

Foundation 已有值得保留的局部性能基础：配置写入离开调用线程，25ms trailing debounce 把 burst 合并为一次 attempt；dirty/persisted generation、单 worker、atomic writer、backup recovery、bounded flush 和 late-writer fence 已存在。EventBus provider 也具备 bounded/latest policy、dead subscriber 回收、精确 counter 与采样式时延诊断。

但它仍不是可接受的 MVP 基础设施。当前三个 P0 都是架构阻断：

1. App 依赖 activation 前后两次直接写 Core 模拟配置优先级，dynamic session 只在 activation 前写 render profile；磁盘旧值可以覆盖显式 session 输入。
2. `ConfigPersistenceWorker` 每次 attempt 调用 Core `snapshot`，复制整个 `HashMap<String, Value>`，再 `serde_json::to_vec_pretty` 生成整文件 bytes。任何 durable key 写入都会把 Editor capability、Animation、Physics、Window/Platform/Render profile 等旁路值一起持久化，CPU、RSS 与写放大按全局配置总量增长。
3. 所有 Runtime 默认竞争同一全局 `config.json`。第二个 live manager 注册同 path 会推进 epoch，使第一个 manager 后续 commit stale；dead-key 回收只控制 registry cardinality，不能修复 live owner 互相 supersede。

另有 capability truth 阻断：`ConfigDriver` 与 `EventDriver` 是零字段 ZST，却以 Immediate driver 激活；manager 不依赖也不调用它们。`DefaultEventManager` 只是把 topic 复制成 `String` 后转发给 Core，production resolver/consumer 为 0，Runtime UI 又有另一套 `UiEventManager`。在 authority、consumer 和 schema 未成立前优化一次 topic clone 没有产品价值。

## 3. 算法与复杂度判断

| 路径 | 当前复杂度 | 实际风险 | 处理决定 |
|---|---|---|---|
| `set_value` | Core load 与 store 分两次锁；单 key 平均 O(1) | compare/write 不是 revision transaction，竞态与旁路并存 | M118.3 硬切 typed transaction |
| persistence attempt | O(K + B)，K 为 Core 全部 config key，B 为 pretty JSON bytes | 每次 dirty flush 都 full snapshot + full serialization + full-file replace | M118.3/4 durable projection + scoped broker |
| path fence register/commit | 全局 path map 平均 O(1)，commit path 串行 | 默认地址无 runtime/project/profile identity，多个 live owner 语义错误 | M118.4 typed address + lease/CAS |
| persistence report p95 | 最多复制并排序 64 个样本，O(64 log 64) | 低频 control-plane，可忽略 | 不单独优化 |
| EventManager publish | topic 分配 + Core publish | facade 无真实 consumer、无 typed result/scope/schema | M118.5 建立唯一 authority 或删除 facade |

当前 full snapshot 已在后台线程执行，因此它不再是“同步主线程整文件重写”；旧 failure 的静态止损已有进展。但后台化不等于算法完成：大配置下 worker CPU/RSS/I/O 仍线性放大，shutdown/flush 仍可能把延迟转移回调用方。应以 durable key projection、delta/revision 和 scoped backend 降低工作规模，而不是再加线程或缩短 debounce。

## 4. Unreal 源码依据

Unreal `FConfigCacheIni/FConfigContext` 明确区分 config hierarchy、static/dynamic layer、saved/runtime change、command-line override 与 disk-backed/temporary cache；`FConfigFile::UpdateSections` 能对 hierarchy 做差异化保存。该证据支持 Zircon 建立 typed layer/source/durable projection，不支持继续把所有 Core 值写进一个全局 JSON。

Unreal `FModuleManager` 把 load、`StartupModule` 完成、module-changed notification、pre-unload、shutdown 与 unload 分成真实生命周期；注册名字或构造 ZST 不等于 capability Ready。Zircon 必须删除空 driver，或让真实 provider/health 成为 descriptor 的 admission 条件。

Unreal Messaging 的 context 带 message type、sender、recipients、scope、send/expiration time、forwarding context，bus 有 authorizer、interceptor、router、tracer 与 shutdown。Zircon 不必照搬 C++ bus，但必须吸收 typed catalog、scope/generation、publication result、cursor/gap 和显式 quiesce；当前 raw `String + JSON` facade 不能作为最终性能架构。

## 5. 计划采纳

### M118.0：Truth Freeze 与 RED

先建立四个产品语义 RED：disk 覆盖 session profile、whole-store leakage、两个 live Runtime 同 path、empty driver 伪 Ready。测试必须走 App/dynamic session/Editor 产品调用链，不接受只构造 `CoreRuntime::new` 的替代证据。

### M118.1-2：Descriptor 与 Boot Hard Cut

删除空 driver/public name，或接入真实 provider/health；Host 在任何 consumer activation 前提交一次 immutable `BootConfigSnapshot`，按 source/scope/profile/project/principal 编译 precedence。删除 App 双写与 dynamic session raw store。

### M118.3-4：Typed Config Authority 与 Persistence Broker

建立 typed key registry、layer、transaction、revision/delta、migration；worker 只 materialize durable projection。backend address 绑定 scope/project/profile/principal/runtime policy，owner lease/CAS 明确共享或隔离。复用现有 debounce、generation、atomic writer、recovery 与 fence，不再让它们围绕全局 Core snapshot 工作。

### M118.5-6：Typed Event 与 Shutdown

Core EventBus 收进唯一 provider；至少接通一个真实 Runtime/App/Editor consumer，并删除测试专用 facade或旧 Core 入口。module shutdown 显式 quiesce、拒绝新调用、flush/drain 并返回 provider receipt，不能只在 `Drop` 中等待 2 秒后记录日志。

## 6. 动态资格

1. Config scale：`10/1k/100k` registered keys，durable ratio `1/10/100%`，value `16 B/4 KiB/1 MiB`，burst `1/100/10k` writes；记录 caller p50/p95/max、snapshot/copied/serialized/written bytes、write amplification、worker CPU、peak RSS、debounce coalescing 和 flush latency。
2. Multi-runtime：Editor + PIE + tool + 2 dynamic sessions 同进程；覆盖 shared/isolated address、simultaneous commit、crash/late writer/restart、last-good 与 cross-process lock/CAS。
3. Event：topics `1/1k/100k`、subscribers `0/1/100`、policies lossless/bounded/latest、publish rate 与 slow consumer；记录 allocation、queue depth、drop/gap、delivery p95、shutdown drain 和 stale generation rejection。
4. WPR/ETW 必须绑定 current-source artifact，观察主线程、`zr-config-persist`、file I/O、context switch、RSS 与 power。Foundation 无 GPU 路径，RenderDoc 不适用。
5. 当前会话没有受管 Windows Cargo validator，因此 Rust tests、App/Editor、WPR/ETW 均为 0；只可记 static complete，不能进入最终验收。

## 7. 本轮结果

- 14/14 Foundation Rust 文件逐文件复读；当前 tree clean，baseline 后源码 diff 为 0。
- Optimize118 current-source owner 结论被采纳；3 P0 保持 open，不重复登记。
- 生产代码、测试、Cargo、ABI 改动为 0；避免在错误 authority 上进行 topic clone、percentile 或 debounce 微调。

