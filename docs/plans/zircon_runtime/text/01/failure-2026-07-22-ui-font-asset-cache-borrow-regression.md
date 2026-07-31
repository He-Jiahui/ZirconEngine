---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: ui-font-asset-cache-borrow-regression
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_runtime/text/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/font_assets.rs
tests:
  - python -m unittest -v tools.tests.test_text_01_composite_activation.Text01CompositeActivationTests.test_renderer_invalidation_consumes_semantic_font_asset_changes
  - cargo +1.94.1 test -p zircon_runtime --lib text_font_database_shared_face_tracks_owner_mapping_without_render_input_change --locked --jobs 1 -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib text_font_asset_shared_face_owner_mapping_changes_trigger_invalidation --locked --jobs 1 -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
  - cargo +1.94.1 test -p zircon_runtime --lib text_font_asset --locked --jobs 1 --color never
---

# Text01：UI font asset cache 借用回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行者：`editor-layout15-native-keyboard-return-r3-20260722`
- 来源执行切片：native-keyboard window contract fresh locked upward gate，snapshot `680`
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：`UiFontAssetCache` 的资源 identity、negative cache、默认字体投影与返回记录生命周期均由 Text01 字体资源计划持有；Layout15 不应局部改写该缓存合同。

## 失败现象与复现证据

受管 Windows reservation `0905384dda9c4f57836d0cf4329ee2c0`、job `4576d0ee13194594a5dfe684bec27c13`、run `3d7907529ce14245a00399c50e4eff57` 执行：

```text
cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
```

向上门已越过先前 Plugins01 E0631/E0308，但在 `zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs:151:23` 稳定 exit `101`：

```text
error[E0502]: cannot borrow `*font_assets` as mutable because it is also borrowed as immutable
```

`ensure_font_asset_record<'a>` 的 cache-hit 分支通过 `font_assets.get(asset_ref)` 返回引用到 `EnsuredUiFontAsset<'a>`；同一函数的 miss/reload 分支随后调用 `font_assets.entry(...)` 获取可变借用。当前控制流把 identity probe、借用返回与 reload mutation 分成两次 map lookup，导致返回生命周期 `'a` 与后续可变 entry 借用冲突，测试二进制未生成。

## 最低共享层根因

字体缓存没有用一个单一 entry/branch authority 表达“identity 命中则返回借用；identity 失配则替换状态”。借用型返回值与可变更新路径分离后，编译器不能证明两条路径互斥。该问题属于 Text01 resource cache 生命周期设计，不属于 Layout15 调用点。

## 架构修复验收

- 以单一 `HashMap::entry`/等价 owner 分支表达 cache hit 与 reload，确保命中分支返回的 `LoadedUiFontAsset` 借用和 miss/reload 的可变更新在类型系统中互斥。
- 保持 `ResourceCacheIdentity` 失配触发重载、`Missing`/`Error` negative cache、默认字体 `CompositeFontDescriptor`/family 投影与成功加载后 font database publish 语义。
- 不得为规避借用错误 clone `LoadedUiFontAsset`、扩大到 `'static`、使用 raw pointer/unsafe，或删除 identity/negative-cache 合同。
- 增补/保留 cache hit、identity change reload、Missing/Error、default font projection 与 face-count/publish 回归；运行 Text01 focused test 和原始 `zircon_editor --lib --locked --no-run` 向上门。

## 禁止临时方案

- 不得把 `EnsuredUiFontAsset.record` 改成无条件 owned clone 来绕过借用。
- 不得在 Layout15 增加字体缓存副本或调用点重试。
- 不得删除 resource identity 比较、negative cache 或默认字体投影以缩短控制流。

## 修复结果与回传

Resolving state：`text01-ui-font-cache-entry-fix-r2-20260722` 已在原 Text01 owner 租约排空且
`font_assets.rs` 仍保持失败哈希 `77C095405130194B05D0105C042DF74FCCD2AD4B7D37575D23F3D9888208500D`
后接管精确三文件范围。实现现已把 identity 命中和 reload 收束到一次
`font_assets.entry(asset_ref.to_string())`：`Occupied` identity 命中直接通过 `into_mut()` 返回借用，
其余 `Occupied`/`Vacant` 分支在同一 entry authority 内加载并替换/插入，未 clone record、未扩大生命周期、
未使用 `unsafe`。

新增 `text_font_asset_cache_uses_one_entry_lookup_authority` 结构回归，先在旧实现上得到
`entry_count=1 has_get=True` 的静态 RED，再在新实现上得到 `entry_count=1 has_get=False` 的 GREEN；
Rust `1.94.1` `rustfmt --check`、`git diff --check` 和同构 `HashMap::Entry` 借用生命周期 `rustc` 探针均通过。
现有 Missing/Error negative cache、同 revision 状态恢复、resource revision reload、default font family/composite
投影及 face-count 行为测试保留不变。

Text01 owner 的上一条 managed job `ad4801e0c0d9428794dea964d5f82764` exit `101`，但发生在 Cargo 编译前：
验证副本缺少 `zircon_plugins/first_party_editor_catalog/Cargo.toml` workspace member；该结果既未验证也未否定本修复。

snapshot `715` 绑定的 managed focused job `a0660f0127a04f94b9327038d0381cb9`、run
`98a02d5a23d6471dad124c93822a526a` 执行 `cargo +1.94.1 test -p zircon_runtime --lib text_font_asset
--locked --jobs 1 --message-format short --color never -- --test-threads=1`，在完整 Runtime test crate 上 exit `101`。
日志精确统计 `error[E0502] = 0`、`font_assets.rs` error `= 0`，证明原借用错误已越过；测试二进制仍被当前
共享源码 44 条其他错误阻断。该轮同时暴露本切片 `tests.rs` 缺少 `ProjectAssetManager` 四处与
`LoadedUiFontAsset` 两处显式导入，现已在同一测试 owner 内补齐；其余错误属于 Plugins01 私有重导出、
Render10 `render_layer_mask`、Text01 其他 font-database 重构和 Runtime UI input API 漂移。

当前待基于补充导入后的新快照重跑 focused gate，并执行原始 `zircon_editor --lib --locked --no-run`
向上门；未声明 focused behavior、fixed 或 accepted。

2026-07-23 current-source default/UI lib-test job `f9f5581fb83b40c2a3cc81aa15f5bcaa`、run
`b98dc769094b4bd9b96fc445fd8a1332` 已越过原 E0502、完成测试二进制并执行 785 个匹配用例；
最终 `776 passed / 7 failed / 2 ignored / 8083 filtered`、exit `101`、无 live PID。两个 UI font
asset 失败均为旧 fixture 假设：family-less project manifest 经资产导入会发布源字体解析出的 `Fira Sans`，
前一个 recovery 测试则在共享数据库中遗留同名稳定 asset-reference owner，污染了后继 `face_count + 1`
前置条件。fixture 现已在 recovery 结束和后继开始时显式退休该 owner，保留严格 `+1/-1` 生命周期断言；
另一个 Text-owned SDF fixture 路径也已在最低测试 owner 修正。
4 个 foreign substring-filter 失败已分别登记 Render01 `732714`、Runtime07 `732788`、Runtime15
`732833`/`732834`。本 lifecycle 仍为 `implementation_complete / managed_validation_pending`，待 fresh
精确 UI font asset rerun及上行门禁，不把该 red broad batch 记为 fixed。

补充导入后的 snapshot `743` 已由受管 Windows reservation
`5f4a7b334e964006aa49093d31d790b4`、job `ee9817382cfc429f82a976f204e1d73e`、run
`2103a0ad17bf404cb0c1d76cbddf6128` 重新执行同一 focused 命令，绑定精确三文件 manifest：失败记录
`95E45A4B6FFA77B98867EDBB0646CCC03B3E2953913117981FA910DB816B97EB`、`font_assets.rs`
`A07FB7C6762927A6D4B40235F5D176D14A8249B458A144D84AE74CB26D4829F0`、`tests.rs`
`E656DB4D957F703944653EF1F3335A9562F3ABE62EA6BD7CDDC35626A8FF1EF2`。job 已于
`2026-07-22T02:15:21.718557+00:00` 自动 release；target 仅位于受管池
`D:\cargo-targets\zircon-engine\pool\854b9fd60ea2c53644d7d5b47acd856f5cd84d504fbbd5ab51fd30e8e2a19464`。

该轮 exit `101`，但日志精确统计 `E0502 = 0`、`font_assets.rs = 0`、Text `tests.rs = 0`、
`ProjectAssetManager = 0`、`LoadedUiFontAsset = 0`；由此确认原借用错误与本切片补充导入均已越过。
测试二进制仍未生成，`stdout.log` 为 0 bytes；唯一终止原因是共享源码中的 20 条外部 Plugins01
`RuntimePluginId` 非 `Copy` 连锁错误，集中在 `runtime_profile/availability_projection.rs`、builtin target
module/catalog、descriptor/project selection 与 plugin workspace shape 测试。该外部错误不属于 Text01 精确三文件
范围，本记录保持 `open`；待 Plugins01 恢复共享编译后仍须执行 focused behavior 与 Layout15 原始向上门，方可回传
`fixed`/`accepted`。

随后 Text01 在最低 owner 完成了缓存与数据库生命周期收敛：`font_assets` 以稳定 asset reference 作为
数据库 owner，reload/replace/remove 均返回 `database_changed` 与 `asset_mapping_changed` 两种独立语义；
UI 只消费这两个语义变化触发 reshape、bitmap 与 SDF invalidation，不再用 face count 推断变化，也不再在
renderer 构造期重复发布字体库。多 owner 行为测试覆盖两个资产引用同一物理字体时，第一个 owner 的
`Missing -> Ready -> Missing` 会持续报告 mapping change，而 face 由第二个 owner 保留、generation 不抖动；
最后一个 owner 删除时才退休 face。共享 owner topology 与 UI asset-mapping lifecycle 的独立 review 均为
`0 Critical / 0 Important / 0 Minor`，19 项 Python 边界守卫已通过。

最新 managed current-source focused behavior 已完成：reservation `d2f96ea5e16e4954bc2b22d3f7eff0fa`、
job `7a3535adc537456e9a3fe3857903652d`、run `ddfbf16587684a9d822fa0e4c76beeca` 执行完整路径
`cargo +1.94.1 test -p zircon_runtime --lib graphics::scene::scene_renderer::ui::text::tests::text_font_asset_shared_face_owner_mapping_changes_trigger_invalidation --locked --jobs 1 -- --exact --test-threads=1`，exit `0`。默认特征 Runtime test-profile 冷构建为 53m11s；该测试实际执行 `1 passed / 0 failed / 9002 filtered`，证明共享物理 face 上的 owner mapping `Missing -> Ready -> Missing` 会触发 UI 的语义失效，而不会重复注册 face 或依赖 face-count 推断。前置数据库 owner-mapping gate `a95d56e3451e469f8ca4c9ebea0228ae` / `39555ec2659f4d3a92609cef34fbe646` 也已实际执行 `1 passed / 0 failed / 5010 filtered`。

该 focused evidence 只关闭本 failure 的两项行为断言；default/UI broad lib-test 与原始 Layout15 `zircon_editor --lib --no-run` 向上门尚未在本 current-source snapshot 返回。因此本失败继续保持 `open / managed_validation_pending`，不声明 `fixed` 或 `accepted`。

Current-source default/UI broad return is also green: managed job `15ff3d8dfd154e93bd1f8fe23ea33aa6`, run `1cffe3b7d1c047958ee2cb49122fe325`, executed `cargo +1.94.1 test -p zircon_runtime --lib text_font_asset --locked --jobs 1 --color never -- --test-threads=1` with exit `0`: `8 passed / 0 failed / 8995 filtered`. It covers one-entry cache authority, production panic guard, resource-revision reload, Missing/Error negative-cache recovery, Ready-to-Missing face retirement, shared owner mapping invalidation, and TTC owner cleanup. The original Layout15 editor upward job `4eefa547982a4bd896813d9fad698f21` / run `ceff37fc13224768af1c365287f242e5` is terminal and released with exit 101 after 50m14s: it compiled Runtime/Text but then reported 56 unrelated editor private-field, pane-template, event DTO, lifetime, and test-type errors. No Text01 source file was diagnosed. This failure remains `open / external_editor_return_pending` rather than being misclassified as Text01 RED.

The current-source production cache-report refinement is green: job `d73b8e1acabc4ca7b25653a3c4931d2f` / run `dd5fa01d1d8a4966af9fc76fa2db727a` completed default-feature `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1 --color never` with exit 0 in 23m09s, after moving `record/loaded/cache_hit/status` to test-only observations so production only carries the semantic `faces_changed` report. The attempted fresh UI broad lib-test job `4bb84ac5c91e4a25bef3cf294e8dad5e` / run `c3c2cc913a854a14b3fc11e7c150a945` could not execute its Text filters because current `deferred/lighting_pipeline/tests/runtime_pipeline.rs` imports removed `create_lighting_pipeline` (external E0432). It is not Text01 RED; retain the prior 8/0 focused/default/UI evidence and leave this failure open pending the external test repair.

外部 Frameworks05 current-source compile job `282d191c8da14fb38a3edd5804464424`、run
`2206a8fe67b9462489bbb403439baf2a` 于 `2026-07-22T03:45:50.235485+00:00` 自然
`released / exit 101 / live PIDs=[]`。其 3 条编译错误中，1 条是 Plugins01 `RuntimePluginId`
moved-value 外部错误；另外 2 条是本 Text owner 的 `text/tests/prepare_report.rs` 仍在构造已经删除的
`ScreenSpaceUiNativePrepareReport.font_faces_changed` 字段。该报告字段原先重复表达 face-count 推断，现已由
`resolved_texts.font_faces_changed()` 的语义信号取代，因此测试 fixture 已删除两处过期字段初始化。
Rust 1.94.1 scoped `rustfmt --check`、`git diff --check` 均通过，fixture 中该旧字段命中为 0；本轮仍是
编译 RED 诊断与静态修复证据，必须由后续 fresh managed gate 复验，不能据此声明 fixed。

2026-07-28 的最新 current-source `text_font_asset` 门 `6ff285ea3dd0433ea8769c2cc7983a9d` /
`eecab6d27cba42919ecdad6f8685bef3` 以 exit `101` 结束，唯一 Text01 诊断来自本次 test-owner 硬切：
`tests/font_assets.rs` 的 `include_str!` 未从 child 目录回退两层，`tests/rendering.rs` 则错误导入了
不存在的 `crate::text::FontDatabase` re-export。两个迁移错误已在最低测试 owner 修正为
`../../text.rs` 和父模块 test-only 窄导入；格式、路径与 scoped diff guard 已通过。旧 job 已释放，新的
current-source reservation `e0b643eb6f8542cd8d0497fcb575f5e5` 正在 CPU FIFO 等待重跑；此前 8/0
结果不能替代该最新快照的 acceptance，故本 failure 继续保持 `open / managed_validation_pending`。
