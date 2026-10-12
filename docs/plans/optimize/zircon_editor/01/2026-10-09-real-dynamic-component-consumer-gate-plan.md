# Editor01：真实动态组件消费者与覆盖门接入

日期：2026-10-09。状态：设计与消费者合同已明确，覆盖门实现、native、性能和产品验收待完成。

## Owner 和证据

Editor 保留实际 Window/module/pane/UiAsset session 的选择、Source 和发布 owner；Runtime UiSurface 继续独占 tree、focus、capture、IME 及私有状态。验证 helper 放在现有 owner 的 cfg(test) 关联入口，复用原真实驱动，返回实际 UiV2SourceFileReceipt。资源回执只包含 asset_id、source_path、resource_uri、canonical physical_path 和原始 SHA-256；同一驱动先完成实际画面、输入与退休断言。

原 [覆盖门](../../../../../zircon_editor/src/tests/ui/boundary/zui_asset_governance/asset_imports.rs) 的静态根/命名导入遍历与三条现有 Workbench design contract 继续参与检查。当前审查库存为 258 个生产文档、205 个 Component、53 个静态根；最终检查继续遍历全部实际组件资产并要求 offenders.is_empty。

## 接入顺序

1. ten-core 与 Particle：在实际采用的 panes driver 上复用当前 Source Arc、render command ref、生产绘制、host hit 和 typed callback；收集真实 mounted owner 回执，核对新 FileCache 的物理源闭包；逐页 Scene 退休后返回。
2. Tools：复用 UiHostWindow::exercise_module_tools_sources_for_test 的实际 native pointer 驱动。43 个选择、同源 popup clip、真实子页 paint-hit、typed callback 及 Scene 退休完成后才返回。父页只计实际 Assets / Ability / Render / Hud，其他六 core 由 ten-core driver 验证。
3. UiAsset palette：原 21-row 遍历驱动使用实际 PreviewHost 的 preview_source_receipts 与 preview_surface_frame。确认 catalog 默认实例属性、规范 Source、同一画面 Arc、实际命中、normalized 输入以及退休；保持四条原 Rust case。
4. Generated Drawer：已有 native body 直接实例化 canonical Drawer wrapper，唯一 builtin pane Surface 在实际可见生命周期解除原 collapsed 状态。46 authored events、正几何/clip、同 frame input、关闭与 source/focus/capture/IME/route 退休全部通过后，返回实际 wrapper + Panel 回执。
5. 覆盖门执行这些真实 helper，将已经完成验证的实际 Source 闭包并入现有可达集合；原资产遍历及全量 offender-empty 断言保持。

## 当前落点

- ten-core 与 Particle 已有实际源码采用回执；各 native case 和物理源/画面/输入资格尚待批量执行。
- ten-core / Particle 实际驱动提取候选已冻结：[Source witness 补丁](../../../../../.codex/outbox/editor-real-component-source-witness-proposal-v1/candidate.patch)。两条原 Rust case 保留，仅提取真实 owner 的关联入口，返回完整物理 Source 闭包；当前公共 callback reexport 的并行改动要求在 Tools 实际采用后重新绑定。
- 全组件覆盖门候选已冻结：[覆盖门补丁](../../../../../.codex/outbox/editor-real-component-coverage-gate-proposal-v2/candidate.patch)。同一外层 env_lock 调用五个真实消费者入口，再用无 persistent store 的新 FileCache 核对实际返回的物理文件、身份字段和原始 SHA-256。该复核新发现的依赖只有在真实消费者也返回它时才获得覆盖资格。覆盖门 v2 按既有 tests::support::env_lock().lock() 持有真实 MutexGuard，统一保护五条驱动；v1 的测试入口和未持锁错误由当前源码推导发现并保留为历史。候选通过 edition2021 格式幂等及严格 git apply --check；两个补丁均未实际采用，也没有 native 通过记录。
- Tools 完整候选与 UiAsset 的共享 retained-pane ABI 正在修复独立审查发现的禁用 popup、内容折叠和 capture/IME 生命周期问题。
- Generated 等待共享 ABI 实际采用后重新绑定前像。私有 pane converter 上下文显式携带当前模板 runtime 和既有 builtin runtime；Generated 使用后者的唯一 mutable Surface。编译 arena 中的 46 个原 binding/control/event 对应显式 UiActionRef，真实 report 经现有 EditorAdapter 转为 canonical typed payload；没有 action 引用的通知仅更新状态。隐藏保存同一 owner 的值，真实 CloseView 退休 holder、Source、画面与 epoch。
- 当前批量清单 v6 保留 15 arms、所有原 profiles/features/Release producers 和完整性能矩阵；UI 136 是源码预期的保守下限。新增 helper 不是额外 Rust test，后续实际采用才更新 selector inventory。

## 三个旧聚合资产

additional、main、extension 三个 aggregate 保持生产库存。[既有只读分析](../../../../../.codex/outbox/editor-legacy-aggregate-consumer-analysis-v1.md) 与 Particle 的实际采用旁证保持独立历史身份。main 仍引用 additional 与 extension；旧导航、paint/hit 祖先和 BlendSpace screenshot consumer 需完成精确迁移。实体退役还需要明确其产品兼容性责任与授权。

库存中的三项继续接受原覆盖门检查。禁止增加 retirement allowlist、改 asset kind、恢复全量 eager import 或登记假根来消除 offender。先完成所有业务消费者接管和可审查的 caller cut，再处理实体退役决定。

## 验证与完成条件

统一批量 native 包含原 139 个逐事件断言、现有 ten-core/Particle source/paint-hit/失效门、Tools/UiAsset/Generated 的实际病例以及原全组件覆盖门。通过后的接收依据为同一 immutable source baseline 的 terminal receipt、实际非零 case 数、全部原性能矩阵和 Editor/Runtime 产品证据。

协调器入口尚未提供，当前没有 Jenkins request ID，也没有 native/performance/product 通过结论。[计划完成列表](../../../astra/features/editor/2026-10-09-editor01-component-consumer-validation-list.md) 跟踪各步骤。
