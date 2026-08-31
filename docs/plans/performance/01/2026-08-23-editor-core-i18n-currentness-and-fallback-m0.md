---
related_code:
  - zircon_editor/src/core/i18n
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/notifications/presentation.rs
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-core-i18n-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-i18n-protected-plan-routing.md
owner_plans:
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/TextLocalizationManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Internationalization/TextLocalizationManager.cpp
tests:
  - tools.tests.test_editor17_i18n_english_fallback_m0_performance_contract
  - tools.tests.test_editor17_decision_notification_center_contract
doc_type: currentness-revalidation-and-m0
status: static_current_revalidated_simple_m0_landed_dynamic_and_structural_pending
---

# Editor core i18n当前性重验与fallback M0（2026-08-23）

## 当前冻结与产品可达性

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| M0前current index | 7/7 | 1,085 | 36,468 | 10 | `65a8f051c3eeffe79ef205b50d64b4e8c8e1982efa8fd482d71781ae962f0015` |
| M0后worktree | 7/7 | 1,098 | 36,717 | 10 | `d2941850bc5cb8ce79b289039b5f39da03b83637377de4a6654e19edc96f5163` |

7/7文件和10个Rust测试已完整复读。两个embedded TOML bundle仍为4,093/4,051 bytes，各54个
translation keys。生产构造、settings hot apply、editor-message publication以及notification调用链已重查：
实际非测试本地化集中在Decision/Toast/Progress presentation；`EditorTopic::i18n()`仍只有production
publisher，subscriber注册均位于builder测试。

当前两locale/54-key的BTreeMap查找不是已证明的P0算法瓶颈。bundle只在service构造时parse一次，
翻译值以`Arc<str>`共享；settings generation可拒绝stale/no-op transition；event queue受32 entries/
64 logical bytes约束并在overflow时合并为latest-locale resync。这些边界保留。

## 已落地的简单M0

旧fallback表达式每次调用`EditorLocale::english()`，即每次非英文active bundle缺key时执行一次
`Arc::from("en")` locale owner堆分配，只为了在BTreeMap中探测English bundle。

本轮把English tag集中为唯一静态常量，为`EditorLocale`实现与其排序一致的`Borrow<str>`，并用借用的
静态tag查询bundle。静态热路径操作数变化为：

| operation per non-English fallback lookup | before | after |
|---|---:|---:|
| English locale owner heap allocation | 1 | 0 |
| translation body copy | 0 | 0 |
| active/English BTreeMap probes | <=2 | <=2 |

缺失于所有bundle的raw key仍会`Arc::from(key)`；本轮没有添加无界missing-key cache。该成本应在
text revision拥有的changed projection中按entries和bytes约束，而不是独立扩张全局intern table。
以上是代码路径计数，不冒充allocator实测；动态allocation/RSS仍待current-source binary验证。

## 剩余结构性P0

### 缺少visible text revision

locale值和settings generation不是localized display generation。retained consumer无法O(1)证明已缓存
文本仍current，只能在上层重建时重复translate。active Workbench notification同步当前每tick重新抓取
Decision/Toast/Progress并本地化，即使locale和source rows都没有变化；fallback M0只降低单次重建成本，
没有消除稳定帧工作。

目标必须由`EditorI18nService`发布非零monotonic `TextRevision`和typed cause：至少区分
`LocaleChanged`与未来`BundleRevisionChanged`。accepted visible-text change只增一次；stale/no-op settings
不增。Notification统一projection消费该token，不建立自己的locale generation。

### 无产品consumer的序列化event路径

每个accepted locale change仍构造JSON并fanout到`EditorTopic::i18n()`，而当前产品UI直接读取i18n
service。Editor17应在确认无plugin/runtime consumer后hard-cut该duplicate path；若确有外部consumer，
只在外部边界序列化同一typed revision/cause并保留现有bounded resync，不能维持第二authority。

## Unreal源码依据与适配边界

- `TextLocalizationManager.h:50-72`把display string作为共享引用保存于text-id lookup table，支持Zircon
  保留共享immutable string body，不支持每次projection复制正文。
- `TextLocalizationManager.h:270-285`暴露全局与local text revision，让缓存以revision mismatch决定
  recache。这是Zircon缺失的核心合同。
- `TextLocalizationManager.cpp:1795-1817`在write lock内推进非零revision、清local revisions，并在game
  thread广播或调度到game thread。Zircon应把accepted revision与UI apply分离但保持settings顺序。
- Unreal还区分language change与new localization data的失效原因，避免错误时序的font/glyph cache
  刷新。Zircon因此不能用generic bus event作为未来所有文本资源的全局clear信号。

Zircon当前embedded localization总量约8 KiB，无依据引入Unreal规模的异步bundle I/O、全局singleton或
复杂live table。先解决revision和稳定帧projection，之后只按实测scale crossover选择map/slot结构。

## 依赖有序优化计划

1. Editor17为i18n增加non-zero `TextRevision`、typed invalidation cause和O(1) compact token；保留现有
   settings-generation rejection与captured-locale consistency。
2. Settings `PERF-MVP-591`每个accepted affected locale slot只在unlock后通知一次；stable frame不读取
   settings snapshot或重新sync i18n。
3. Notifications `PERF-MVP-596`把text revision纳入Decision/Toast/Progress统一generation；每个accepted
   source/text revision最多build/apply一次，stable tick translations和localized-row builds为0。
4. EditorUI08保存last-applied token并只更新visible rows；未来bundle/font job必须revision-check commit，
   stale result不可显示。
5. 删除unused JSON path，或把external compatibility serialization限制在唯一typed consumer边界。
6. 接线后用allocator、WPR/xperf和功耗验证，locale change渲染变化再用RenderDoc检查glyph/draw parity。

## 量化验收

| matrix | 必须记录 | acceptance |
|---|---|---|
| locales `2/10/100`，keys `54/1K/100K`，active/fallback/missing | lookup p50/p95、comparisons、locale/key allocations、returned text bytes、RSS | fallback locale allocation=0；found string body copy=0；missing-key状态按entries+bytes有界；map/slot按实测crossover决定 |
| stable `1/1M` retained ticks，notifications `0/1/max` | token reads、locale locks、translations、rows built/applied、allocations | 初次apply后stable translation/localization/projection=0；token read O(1) |
| accepted/no-op/stale change，producers `1/16`，slow/rejected sink | settings/text revisions、queue entries/bytes/age、sink wall、JSON bytes、resync、UI applies | visible change revision/apply各<=1；no-op/stale=0；queue顺序/界限/恢复保持；unused JSON=0 |
| F4 locale switch，至少31次cold/warm可比run | WPR CPU/contention/context switch、allocator/RSS、package power、RenderDoc glyph/draw | stable-frame i18n成本消失；locale切换无mixed text、stale apply或glyph回归；跨run分布稳定 |

## 本轮静态门

- 新M0契约按TDD先RED后GREEN；与notification当前契约合计9/9 Python tests通过。
- `rustfmt --edition 2021 --check`为7/7通过；scoped `git diff --check`通过，仅输出既有LF/CRLF提示。
- 未运行Rust/Cargo、allocator、WPR、功耗或RenderDoc：managed validator已归档且无current-source可执行
  文件。模块仍为dynamic/structural pending，不得写入`review.md`，不触发里程碑commit或企微通知。
