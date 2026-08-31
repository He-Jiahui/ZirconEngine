---
related_code:
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_dispatch.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_diagnostics.rs
base_reports:
  - docs/plans/performance/01/2026-08-15-runtime-asset-project-registry-pipeline-current-architecture-review.md
  - docs/plans/optimize/zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
owner_plans:
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Public/IDirectoryWatcher.h
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Private/Windows/DirectoryWatchRequestWindows.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Public/FileCache.h
  - dev/UnrealEngine/Engine/Source/Developer/DirectoryWatcher/Private/FileCache.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/AutoReimport/ContentDirectoryMonitor.cpp
tests:
  - zircon_runtime/src/asset/tests/watcher.rs
  - tools/tests/test_runtime_asset_watch_error_reconciliation_m0_performance_contract.py
doc_type: currentness-revalidation-and-m0
status: static_current_revalidated_simple_m0_landed_dynamic_and_structural_pending
---

# Runtime asset watch error reconciliation当前性重验与M0（2026-08-23）

## 当前冻结与审查边界

| scope | Rust文件 | physical lines | bytes | tests | fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_runtime/src/asset/watch/**` M0前 | 19/19 | 817 | 26,537 | 0 | `0bc1fe0815de2a265fb6cf24e6fb0f888a0b6f07cd91982f5af8e8c321404626` |
| `zircon_runtime/src/asset/tests/watcher.rs` M0前 | 1/1 | 289 | 9,748 | 9 | raw SHA256 `9bd23dc2fa8f5ad2fcd240ddb0e57844dacfc8de9f62e42c9713f4acac444322` |
| `zircon_runtime/src/asset/watch/**` M0后 | 19/19 | 825 | 26,889 | 0 | `3b37bad0509fcc0b1fd7cd17c40f86ef5fd8662d5de9a277f93e9089d98d089b` |
| `zircon_runtime/src/asset/tests/watcher.rs` M0后 | 1/1 | 291 | 9,847 | 9 | raw SHA256 `a896c66ad5401da0aa3f7e47fef9826c7ab6b9346aaec89d01b860d20df6b0b9` |

生产目录19/19文件和watcher测试1/1文件已逐行复读。生产调用继续沿
`AssetWatcher::spawn -> watch_loop_inner -> ProjectWatcherActivation ->
process_watch_batch_in_generation -> ProjectManager candidate -> resource commit -> project generation publish`
读完。`runtime.rs`、`watch_diagnostics.rs`及相邻asset-resource文件当前有其他Session改动，本轮只读并
保留，不把watcher M0扩张到这些owner。

生产链已有的正确底座继续保留：ingress与folded pending同时受entries/approximate bytes约束；
debounce与max latency分离；overflow发布`requires_reconciliation`；activation是single-flight worker；
project切换有Pending/Draining/Active/Retired generation；文件、resource与ProjectManager candidate在
generation gate下提交。当前问题不是缺少线程，而是错误、source truth、committed generation和consumer
cursor没有形成一条统一事务。

## 当前结构瓶颈

### Provider error只进入日志/通知，不进入source truth恢复

`watch_loop_inner`收到`notify::Result::Err`时只调用`on_error`。普通单个错误进入容量64的error queue并
广播；只有error queue发生驱逐时才设置reconciliation。若OS watcher已经漏掉事件，catalog、artifact和
resource可无限保留旧事实。

这是Optimize88 `WATCH88-P0-001`的current-source确认，不是基于名称的猜测。当前错误路径静态行为为：

| operation per provider error burst | M0前 |
|---|---:|
| observable error callback | 每个error 1次 |
| source reconciliation request | 0 |
| dirty-until-success latch | 0 |

### Reconciliation提交与consumer publication不是同一事实

`requires_reconciliation`会执行full generation和resource reconciliation，但成功后仍把输入`changes`
原样发布。overflow已清空输入时，Runtime可提交新generation而AssetChange consumer收不到任何delta。
该问题必须由`CommittedAssetGeneration`或`SnapshotRequired`硬切解决，不能制造伪`Modified(res://)`；
本轮M0不修改该合同。

### Compound source owner仍由事件形状猜测

目录Compound成员的单个Modify仍会以成员URI进入single-source targeted path。正确目标是由Runtime85
建立immutable source-owner/reverse-dependency generation；不得用向上搜索`.zmeta`的局部修补替代。
本轮M0不触碰该owner。

## 已落地的受限M0

watcher ingress的provider error分支现在同时完成两件事：

1. 保留现有`AssetWatchError`回调和错误可观测性；
2. 把当前batch标记为`requires_reconciliation`，并启动/刷新与普通事件相同的debounce/max-latency窗口。

同一窗口内的error storm只形成一个reconciliation batch，不为每个error直接启动full scan。若窗口中还有
partial changes，activation看到reconciliation token后按现有合同丢弃partial event guess并走truth scan。
静态变化为：

| operation per provider error burst | M0前 | M0后 |
|---|---:|---:|
| observable error callback | 每个error 1次 | 每个error 1次 |
| reconciliation batch | 0 | 每个debounce/max-latency窗口<=1次 |
| immediate full scan in notify callback | 0 | 0 |
| dirty-until-success latch | 0 | 0，仍开放 |

因此M0只关闭“普通error永不请求恢复”的确定性缺口，不关闭WATCH88-P0-001整体。scan/import/commit失败后
保持dirty、typed failure分类、有界退避、terminal recovery receipt仍由Runtime04/11和Optimize88后续
里程碑实现。

## Unreal源码依据与适配边界

- `IDirectoryWatcher.h:8-39`把`FCA_RescanRequired`定义为正式change action，并允许携带目录和事件流
  失真之前的时间戳；这证明rescan是控制面事实，不是日志字符串。
- `DirectoryWatchRequestWindows.cpp:216-258`区分正常通知、目录不可访问、
  `ERROR_NOTIFY_ENUM_DIR`和其他失败；其中event enumeration失真会设置`bIsRescan`。
- `DirectoryWatchRequestWindows.cpp:290-346`在rescan分支不解析不完整buffer，而是发布目录级
  `FCA_RescanRequired`。
- `FileCache.cpp:1171+`先收割hash结果，再把目录变化合入可持续file-state cache；
  `ContentDirectoryMonitor.cpp:126-198`tick cache并从outstanding transaction按阈值取工作。这支持未来
  dirty-until-success与snapshot delta，不支持为每个error同步全扫。

Zircon不复制Unreal的C++ singleton、Editor tick或完整FileCache层级。当前M0只吸收“事件流失真必须进入
truth reconciliation”的最小合同；最终仍使用Zircon唯一project/source generation和shared task runtime。

## 测试先行与验收

1. 现有watcher行为测试先修改为单个`notify::Error`必须同时收到原错误和一个空changes、
   `requires_reconciliation=true`的batch；M0前静态合同得到1项RED。
2. `tools/tests/test_runtime_asset_watch_error_reconciliation_m0_performance_contract.py`实施后
   2/2 GREEN；测试38行、1,545 bytes、raw SHA256
   `771cc0bea7727eecfde692b8eae9d0bc42fc87ed579ee0be2c6e56feb8e95c2d`。
3. 连同Editor asset workspace/world-sync/ZUI watcher邻近合同运行19/19 GREEN；focused
   `rustfmt +1.94.1 --edition 2021 --check`和scoped diff check通过。
4. managed Cargo恢复后执行watcher unit/integration tests，并注入1/64/1,024 errors、mixed event/error、
   scan failure和shutdown race。
5. current-source F4项目运行时记录error-to-reconcile request/commit latency、batch count、scan count、
   queue age、CPU/RSS与功耗；没有同机动态数据前不声称性能接近Unreal。

RenderDoc不适用于watcher CPU/IO控制面。只有asset reconciliation改变可见纹理/mesh后，才用RenderDoc
验证资源generation与最终像素/draw一致性；它不能替代WPR/xperf、IO、queue和generation counter。

## 剩余硬切目标

- `ObservedSourceChange -> AssetInvalidationPlan -> CommittedAssetGeneration -> consumer cursor`
  单链，错误与overflow携带typed dirty scope和terminal receipt。
- scan/import/resource/durability任一失败保持last-known-good和dirty-until-success，使用shared bounded
  operation lane、deadline、cancel与backoff，不建asset私有pool。
- committed delta由candidate与previous generation计算；consumer gap必须收到`SnapshotRequired`。
- Compound、auxiliary input、metadata和package source都通过generation-built owner/reverse-dependency
  index解析，不再用event count/shape决定targeted correctness。

本报告及M0完成后仍保持`dynamic_and_structural_pending`，不得进入`review.md`。

当前managed Cargo执行身份已经归档且不可生成新命令，因此升级后的9个Rust行为测试执行数仍为0；本报告
不以Python源码合同、rustfmt或静态operation count冒充Rust行为、真实filesystem fault、wall time或功耗
验收。
