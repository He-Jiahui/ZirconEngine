---
title: Runtime42 M0 Effective Manifest Registration Filter
category: zircon_runtime
report_id: Runtime42-M0
date: 2026-08-24
baseline_head: 0e2bdaa9d3f6949e351ce4e77ccf1aca9e7032b1
baseline_epoch: 383
session_id: optimize-runtime42-manifest-registration-filter-r2-20260824
implementation_status: implementation_complete
validation_status: managed_cargo_pending
review_status: independent_review_pending
---

# Runtime42 M0 Effective Manifest Registration Filter

## 目标与边界

本切片关闭父计划确认的启动组合正确性断路：`active_plugin_registration_refs()` 只检查 registration 自身的
`project_selection.enabled/supports_target`，没有检查本次组装的 effective `ProjectPluginManifest`。因此一个已经被
项目禁用或根本未被选择的插件，仍可把 asset importer、render feature、shader source 和 runtime provider 注入
Asset/Graphics 模块。

本轮只收敛 plugin registration extension admission。Feature 多 provider 选择、required capability、compiled
feature/schema 差异与完整 `RuntimeCompositionPlan` 仍由父计划后续里程碑负责；不得用本切片冒充完整 composition
compiler。

## Current-source 与参考复核

实现前逐行复核了：

- `assembly/registration_reports.rs` 的三个 target/profile 入口、registration input merge 和 availability 投影；
- `ProjectPluginManifest::enabled_for_target`、mode baseline overlay、`RuntimePluginId::parse_key` 归一化；
- plugin/feature registration report、extension registry、asset importer merge 与 load diagnostic 路径；
- 现有 registration/manifest 行为测试和结构约束。

参考结论：

- Unreal `FPluginManager::ConfigureEnabledPluginForCurrentTarget` 先形成 target-enabled plugin map，后续
  `GetEnabledPlugins()` 只投影 `Plugin.IsEnabled()` 的对象；发现到插件不等于获得启动贡献资格。
- Bevy `PluginGroupBuilder::finish()` 只把 `entry.enabled` 的插件加入 App；disabled entry 可保留排序位置，但不会
  执行 plugin build。
- Godot `GDExtensionManager` 由单一 extension map 管理 load/initialize/reload 生命周期；Fyrox 由 Executor/Engine
  统一持有并 enable plugins。两者同样不让一份旁路注册列表独立决定运行时贡献。

Zircon 本轮吸收共同底线：effective project selection 是 extension admission 的权威输入；registration 自身状态
只能进一步收紧，不能绕过项目选择。

## RED 合同

测试先构造 Sound 与 Animation 两份 registration，各注册相同 importer ID/matcher：

1. manifest 只选择 Sound 时，Animation 的 extension 必须被排除，不得产生 duplicate importer fatal；旧实现会把
   两份 registration 都 merge，因此该断言失败。
2. manifest 显式禁用 Animation 时，即便 registration 自身 enabled，其 extension 仍必须被排除。
3. manifest 同时选择 Sound 与 Animation 时，duplicate importer fatal 必须保留，防止实现错误地丢弃全部
   registration 或吞掉真实 extension 冲突。
4. manifest 以 `audio` alias 选择 Sound、同时选择 Animation 时，两个 importer 仍必须冲突，锁定 extension
   admission 与 availability 共用 canonical runtime identity。

test-only managed ticket `10a499511133479e8bceab87d931348d` 在 materialization 阶段失败：validation copy 检测到
5 个无关 Runtime74 UI 文件发生 `validation_copy_baseline_drift`，测试没有编译或执行。该终态不是有效 RED；r2 必须
在相同 test-only source manifest 上取得真实断言失败后，才能把 RED gate 标记完成。

当前 GREEN ticket `b4b3660ed888416da802ea19f059084c` 也在编译前被相同 materialization gate 拒绝：Cargo
closure 中存在 6 个已加入 index、但不在 pinned HEAD 且不属于本 Session 的 Runtime74/RHI 新路径。重放不会绕过
该 fail-closed 规则；必须先由原 owner 收口这些 staged additions。

## 算法与 hard cut

1. 每个入口先取得同一份 effective manifest，再交给 registration admission；不再允许先 merge registration、后
   计算 manifest。
2. 以 `manifest.enabled_for_target(target)` 物化 enabled runtime ID 集合，ID 通过
   `RuntimePluginId::parse_key` 复用 availability 的 canonical identity；不能用原始字符串比较，否则 `audio` 与
   `sound` 会在 availability 和 extension admission 中产生两套真值。
3. registration 只有同时满足自身 enabled、自身支持 target、ID 可解析且存在于 effective enabled-ID 集合时才
   可进入 extension input merge。
4. 不保留“registration 默认 enabled 即隐式加入项目”的兼容分支、fallback 或第二套 facade。
5. 两个直接 Runtime 集成 consumer 已从 `manifest=None` 迁移为显式包含 registration selection 的
   `ProjectPluginManifest`；测试不再把旧的隐式加入语义当成受支持合同。

设 effective manifest selection 数为 `M`，registration 数为 `R`。物化集合与过滤总时间为 `O(M + R)`，额外空间
为 `O(M)`；不采用每个 registration 线性扫描 manifest 的 `O(M * R)` 方案。该路径属于启动组装，不宣称经过
profiler 的帧时优化、功耗收益或跨引擎耗时优势。

## 实现与静态证据

三个 target/profile 入口现在都把同一份 effective manifest 传给 `active_plugin_registration_refs()`；第一个入口已
调整顺序，先计算 manifest，再生成 registration input。filter 一次物化 enabled canonical ID `HashSet`，随后保留
registration 自身 enabled/target gate，并对 registration ID 做同一 canonical parse + membership gate。没有保留
registration 默认启用即可旁路项目选择的兼容分支。

生产 diff 为 `+15/-4`；四份 Rust 总 diff 为 `+144/-6`，测试与 consumer hard-cut 覆盖未选择、显式禁用、
真实冲突和 alias 四条 admission 回归。`registration_reports.rs` 为 152 行，三个测试 owner 分别为 351、273、
500 行，均低于生产 800 行 review warning
与测试 1000 行 hard gate。四份 Rust 文件已执行 exact `rustfmt --check`，scoped `git diff --check` 为 GREEN。

r1 因 immutable scope 遗漏两个真实 consumer 而以 `cancelled/superseded` 终结；r2 通过 ownership transfer fingerprint
`082a67dbc43a53e29dd88f2739efc9bca528b3d8a26efcde3c3fc26ab47999da` 原子接收原三份 blob，再扩展到两个
consumer 文件。没有改写或接管 `manifest.rs`、UI document importer 测试等外来 mixed blob。

## 状态

- [x] current source、consumer、identity 与 extension merge 边界复核。
- [x] Unreal/Bevy/Godot/Fyrox 启用真值与执行边界复核。
- [x] 产品级 RED fixture 先于 production 修改写入。
- [ ] managed RED 实际执行并确认旧行为失败；首个 ticket 仅为无效 materialization 失败。
- [x] effective manifest filter 与三个 caller hard cut 实现。
- [x] canonical alias admission、未选择排除、显式禁用排除与真实冲突保留四条回归完成。
- [x] 两个 `manifest=None` consumer 显式迁移到 effective project selection。
- [x] exact rustfmt、scoped diff check 与四个 owner 文件预算复核完成。
- [ ] focused tests、core-min build 与必要 downstream compile GREEN。
- [ ] 独立 reviewer 复核 identity、admission、复杂度与非目标边界。
- [ ] coordinator immutable manifest、service commit 与自动 WeCom 量化通知。

在 managed validation 与独立复审完成前，本里程碑不得提交。

当前 Rust 源码 SHA-256：

- `assembly/registration_reports.rs`: `a0d2d90fd8df2fe4f30e5585049e04acbd3afe2e84ba797f4020af9af00b64e9`
- `tests/registration/behavior.rs`: `bfb1cf7b9a460ffa0ebd0afbdbe34dd762e2adb4dc47742baa7b50dcf4b8863a`
- `tests/plugin_extensions/asset_importer_install.rs`: `ec2ef015d5ca019cad676e5d3fc69026b7ec750df973ca3708e7abbc233b6ed9`
- `tests/plugin_extensions/extension_registry.rs`: `cd45d1913f7fa347f2a5f1695a57b27c358f5789b1ea5c3f5391b86a54775baa`
