---
handoff_kind: failure
status: open
created_at: 2026-07-19
updated_at: 2026-07-28
summary_slug: cross-platform-preference-storage-service
origin_plan: docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/woc/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/platform/service_types/mod.rs
  - zircon_runtime/src/platform/preferences/atomic_file.rs
  - zircon_app/src/entry/platform_preferences.rs
  - zircon_runtime/src/platform/target.rs
  - examples/woc/native/apps/woc_client/src/preferences/storage.rs
  - examples/woc/native/apps/woc_client/src/input/keybind/storage.rs
  - examples/woc/native/apps/woc_client/src/input/gamepad/storage.rs
tests:
  - cargo test -p zircon_runtime --lib platform_preference_storage --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test --manifest-path examples/woc/native/Cargo.toml -p woc_client --test input --locked --jobs 1 -- --nocapture --test-threads=1
---

# Frameworks 05：运行时缺少跨平台偏好存储服务

## 来源执行者

- 来源计划：`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md`
- 来源执行切片：M8 desktop client keybind/gamepad preference persistence and M13 platform preparation
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因（2026-07-19）：最低共享原因是 runtime platform 域当时尚未提供中立偏好存储契约及各平台实现；该契约同时服务 native、mobile、browser 游戏角色，不能由 WOC、Editor 私有持久化或单一 host 调用点分别定义。

## 2026-07-19 失败现象与复现证据

交接时 WOC 已在项目内完成目标兼容的键盘与 gamepad 纯存储语义，但无法把 `PreferenceStorage` 接到一个 ZirconEngine 拥有、覆盖 Windows/Linux/macOS/Android/iOS/WebGPU/WASM 的运行时服务：

- 2026-07-19 失败时，`zircon_runtime/src/core/framework/platform/mod.rs` 只导出 module identity 与 `RuntimeTargetMode`，没有 user-data/preference key/value 契约。
- 2026-07-19 失败时，当前已 folder-backed 的 `zircon_runtime/src/platform/service_types/mod.rs` owner 中 `PlatformDriver` 和 `PlatformManager` 仍是零字段类型；Manager 当时只投影 capability report，没有偏好存储入口、用户数据根或 backend handle。
- `zircon_runtime/src/platform/target.rs` 已定义 desktop/mobile/browser/headless 八种目标分类，因此跨目标归属明确属于 platform 合同，而非 WOC 本地 OS 分支。
- 失败时的只读扫描 `git grep -n -I -E "Preference(Store|Storage)|SettingsStore|localStorage|user.data" -- zircon_runtime zircon_runtime_interface zircon_app` 未找到可供运行时游戏使用的统一服务。Editor 当前的 `zircon_editor/src/core/settings/io.rs` owner 仍是 editor-private settings 文档存储，既不公开给游戏角色，也不覆盖 browser/mobile backend。
- WOC 项目内合同现由 `examples/woc/native/apps/woc_client/src/preferences/storage.rs` 唯一拥有；失败时它只能先定义可注入的最小 read/write 合同。键盘和 gamepad JSON、作用域、损坏回退与不可用存储降级已有源代码和测试，但真实平台适配仍无法无特例完成。

这不是 Cargo 队列失败，也没有产品通过声明。2026-07-19 交接时 WOC 受管 Cargo 预约仍未获显式绑定，以上证据仅证明当时的接口缺席与归属边界。

## 2026-07-19 最低共享层根因

交接时 Platform 域已拥有目标枚举、capability matrix、module/manager 名义入口，却没有持久用户偏好的中立服务合同、backend capability 或 host 注入路径。若各游戏自行选择 OS 目录、浏览器 Web Storage 或移动端 app-data API，会复制平台政策并让同一项目在不同 host 上产生不一致的错误、生命周期和隔离语义；复用 Editor 私有 store 则会反转 runtime/editor 依赖并把 TOML appearance 文档误当通用游戏偏好服务。

## 架构修复验收

- 在中立 platform contract owner 定义命名空间化的偏好 read/write（以及实现所需的 remove/flush/error）合同；公开错误必须区分 unavailable、denied、quota/capacity、corrupt backend 与 transient I/O，不以静默内存 fallback 冒充持久化成功。
- `PlatformManager` 或等价 runtime service 通过 host 注入 backend：desktop 使用宿主批准的 user-data root 与原子文件提交，Android/iOS 使用各自 app sandbox，WebGPU/WASM 使用浏览器持久存储；headless/server 明确报告 unsupported 或由 host 显式提供 backend。
- Platform capability report 增加稳定的 persistent-preferences 能力行，并覆盖所有 `PlatformTarget` 与 client/headless target mode 组合；不得在游戏 key 名或 WOC 路径上写特例。
- WOC 的项目内 `PreferenceStorage` 适配器只委托引擎服务，键盘 `woc_keybinds[:scope]` 与 gamepad `woc_gamepad` 行为测试保持不变；fresh-process native 重载、两个隔离角色 scope、损坏值回退、写入拒绝与 browser/mobile backend 测试通过。
- 执行 frontmatter 中 focused runtime 门和 WOC upward gate，并在 M13 用真实 Windows、Linux、macOS、Android、iOS 与浏览器 host 证明同一配置合同；compile-only 不算平台验收。

## 禁止临时方案

- 不得在 WOC 中新增 `cfg(target_os)` 目录选择、直接 `localStorage` 调用、Editor 私有 store 依赖或每平台重复的 key/value 真相源。
- 不得用 process-local map、测试 fixture、写失败吞掉后仍报告 persisted，或把偏好塞进 authoritative ZrVM/world save 作为产品修复。
- 不得添加别名、兼容 shim、单调用点例外、WOC key 特判，或降低 M8/M13 的 fresh-process 与真实平台验收标准。

## 修复结果与回传

2026-07-22 current-source engine repair:

- 中立 `PreferenceStorage` 合同、错误分类、manager handle 与 platform capability projection 已归属 `zircon_runtime::core::framework::platform`，具体 backend 由 `PlatformDriver` 一次性安装；没有 WOC、Editor、游戏 key 或 `cfg(target_os)` 特判。
- `zircon_app::entry` 现在负责 host 注入：desktop 默认选择批准的 user-data root 和 atomic-file backend，mobile/browser/headless 必须由 host 显式注入，否则稳定报告 `Unavailable`。禁用 Platform 的 Minimal profile 即使收到显式 backend 也保持 `Unavailable`，不会为了偏好服务偷偷激活模块。
- backend 只在 `activate_registered_modules()` 完成 Foundation -> Platform 依赖排序后安装；生产代码不再单独调用 `activate_module(PLATFORM_MODULE_NAME)`。Runtime capability diagnostics 与实际安装 backend kind 共用同一真相。
- Rust 1.94.1 rustfmt、exact diff-check 和 `test_frameworks_05_preference_storage_boundary` 7/7 已通过。首次 Frameworks05 全量静态扫描暴露 3 个支撑漂移：Editor manager fixture 与 UI render guard 仍读取拆分前旧文件，platform preference 测试的 concrete Foundation 别名与中立 identity 同名；按最低共享层修复后，完整扫描 52/52 通过（266.389 秒）。
- 2026-07-28 G7 current-owner refresh 将 WOC 合同从退役 input-local owner 硬切到唯一 `preferences/storage.rs`，并把 failure-time Platform/Editor 说明同步到 folder-backed `platform/service_types/mod.rs` 与 `core/settings/io.rs`。focused 文档审计为 `0` violations，三个新 owner 均存在，三类退役路径字符串为 `0`；这只修正机器路径和历史/当前时态，不替代 focused Runtime/WOC managed gate，也不关闭 failure。

Current state: `engine_fix_implemented_pending_managed_validation_and_fixed_return`。`zircon_app/src/entry/engine_entry.rs` 同时含有 Runtime02 尚未验收的 descriptor snapshot 改动，路径级 milestone commit 必须等待两项改动都通过独立门或由协调器显式合并，不能由 Frameworks05 单独吸收。focused runtime Cargo、来源 WOC upward gate 与 failure fixed return 尚未完成，因此不声明产品通过。

## 2026-07-30 Performance01 性能验收补充

- Performance01 current-source增量复读确认：中立合同与`PlatformManager`仍是同步接口；manager只在短RwLock内clone backend `Arc`，随后在锁外调用backend，因此当前锁边界不是瓶颈。
- Desktop atomic-file backend会在调用线程为每次操作重复构造namespace/key哈希与路径；read直接执行文件I/O，write包含staging同步、原子提交、提交文件与Unix父目录耐久化同步，remove也可能同步父目录。这些耐久化步骤不能在缺少崩溃合同证据时当作简单冗余删除。
- 当前WOC仍使用项目内`preferences/storage.rs`合同，Editor未发现生产偏好读写consumer；因此`PERF-MVP-589`是M8/M13或Editor接线前门禁，不把尚未接入的代价误报为现行F0/F2/F4热点。
- 接线验收必须把阻塞backend放入Runtime11统一bounded persistence lane，按key合并latest generation，提供read-your-write及显式flush/shutdown fence，并记录caller filesystem wall、hash/path构造、staged write/fsync、queue entries/bytes/age/coalesce与错误/取消时延。frame/UI caller filesystem wall必须为0；这项动态门不替代本failure原有Runtime/WOC managed gate。
