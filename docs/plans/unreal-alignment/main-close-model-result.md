# 主窗口等待模型导入时，应由同一个关闭操作接收最终结果

ZE 固定 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。初次提交仅调查、去重与测试计划，未修改生产代码。以下为源码上可跟踪的调用结果，不是实际关闭窗口的验收。

## 1. 哪个正常操作受影响？

正常项目的 retained Editor 宿主通过其已绑定的资产面板动作提交一个有效三角形 OBJ。该请求仍由宿主持有时，关闭**同一**主窗口。这里考察已提交后的交接，不假定当前默认模板通过键盘填写路径一定可达，也不把调用面板动作当作桌面端到端输入。ZE 的[已受理、非阻塞关闭合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_editor/09/2026-10-17-owned-nonblocking-main-window-close-source-record.md#L5-L33)明确区分“模型任务还没结束”与“关闭失败”。[Import 按钮](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/runtime/builtin/bindings.rs#L405-L412)进入[有类型的待办票据](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/assets/workspace/model_import_settlement.rs#L136-L193)，后端走[内置 OBJ 导入器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/ingest/asset_importer.rs#L329-L334)及[真实资产提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/manager/service_contracts/asset_manager_contract.rs#L311-L362)，不是手填完成状态。

## 2. 用户会看到什么，有何例外？

即使导入已经成功落盘、工作线程也已经结束，原生关闭仍可能停在 `Preparing`，等到既有 5 秒预算后进入 `RecoveryRequired`。这不表示导入数据丢失、线程死锁或所有关闭必定失败。`Retry project close` 只是重试同一被拒入口；`Resume` 要求尚未真正开始关闭，而此时已在 `Quiescing`。若显式的 `Keep committed assets without adding to scene` 决策在截止前确实交付并被处理，它能释放待办；不能把存在决策分支当作默认工作台一定显示、命中该按钮的证明。见[重试、继续与 Keep 的实际处理](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/assets/workspace/model_import_settlement.rs#L285-L426)、[截止预算](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/main_window_close.rs#L19-L20)、[Recovery 边界](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/main_window_close.rs#L504-L520)。

## 3. 为什么真实的完成还不够？

[主窗口关闭回调](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app.rs#L435-L443)创建唯一 owner。[宿主轮询](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/host_lifecycle/tick.rs#L76-L162)先运行该 owner，再轮询模型结果；[主窗口准备](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/main_window_close.rs#L282-L299)将 `PendingModelImport` 视为等待。[取消处理](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/assets/workspace/model_import_settlement.rs#L195-L210)只提出协作取消并保留票据，这一步是正确的。随后[模型结算](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/assets/workspace/model_import_settlement.rs#L212-L259)实际收到成功回执或取消终态，却转调仅用于项目关闭的旧入口；[旧入口的主权保护](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/project_close.rs#L247-L259)正确返回 `MainWindowCloseOwned`。[模型侧把拒绝记为 CloseRetryRequired 并放回待办](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/assets/workspace/model_import_settlement.rs#L318-L406)，下一轮主窗口于是仍然等这个已结束的票据。只需控制“先请求关闭、后由 UI 消费票据”的正常顺序，不需要让两个线程卡在一条特定指令上。

## 4. Unreal 如何区分取消和完成？

主窗口关闭判定（`Engine/Source/Editor/MainFrame/Private/Frame/MainFrameHandler.h:97–119`）询问Interchange 的活跃状态（`Engine/Source/Runtime/Interchange/Engine/Private/InterchangeManager.cpp:2795–2815`）；仍有导入时可直接拒绝关闭并提醒用户，与 ZE 已经受理后异步等待的政策不同。真实导入任务的完成回调（`Engine/Source/Runtime/Interchange/Engine/Private/Tasks/InterchangeTaskCompletion.cpp:341–346`）在结果定稿后释放实际任务。可移植的是“真实终态交还给正确的所有者”，不是照抄 UE 的 UI。

- `System::Core::Tasks::Cancellation`（`Engine/Source/Runtime/Core/Tests/Tasks/TasksTest.cpp:1492–1515`）：一项任务观察取消，另一项可以忽略，测试最后显式等待。它不验证 Editor 的窗口或导入回执。
- `FInterchangeImportTest`（`Engine/Plugins/Tests/InterchangeTests/Source/InterchangeTests/Private/InterchangeImportTest.cpp:21–163`）及`Editor.Interchange` 测试计划的名称（`Engine/Plugins/Tests/InterchangeTests/Source/InterchangeTests/Private/InterchangeImportTestPlan.cpp:332–357`）：由测试计划发起真实导入、等完成再取回结果。没有查到覆盖“同一主窗口已经接管关闭，再收到导入终态”的一对一 UE 用例。

## 5. 修复属于哪里，又不该改什么？

让已经匹配同一项目 activation、同一关闭操作的主窗口**接收并持有**模型终态，再解除这一项等待；保留成功的持久回执、显式取消和真正失败的差异。后续仍由主窗口依次进行[自动保存和共享 Jobs 排空](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/main_window_close.rs#L382-L408)与[Settings、Session 与退出](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/main_window_close.rs#L423-L485)。不得删除 `MainWindowCloseOwned`，不放大超时，不伪造成功，不自动重导或在关闭期间把模型插进场景。项目单独关闭走其原有策略；不更改 ECS、资产序列化或 Native ABI。与 [#199](https://github.com/He-Jiahui/ZirconEngine/pull/199) 的物理固定步和 [#202](https://github.com/He-Jiahui/ZirconEngine/pull/202) 的首次目录发布无关。

## 6. 如何判定是真失败？

使用[完整宿主测试入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/support.rs#L78-L143)与[已有原生关闭夹具](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/close_prompt.rs#L937-L979)打开真正的 RenderableEmpty 项目，绑定 authoring scene 和 document。先验证真实 OBJ 文件、Runtime Ready 记录和 Job 完成；在 UI 尚未消费票据时发出 native close，按产品顺序驱动轮询。核对同一 activation、同一 Job、没有其它保存阻塞、没有场景插入，再判断下一轮是否还错误地等待同一模型。无导入时的正常关闭、未完成时必须继续等待、无主窗口接管时的普通导入是控制。源码上的模块登记不等于测试已启动，夹具失败/定时器到期不能代替上述行为断言。[既有模型项目级关闭用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/assets/workspace/tests/model_import_settlement.rs#L625-L789)和[删除类关闭用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/close_prompt.rs#L1209-L1318)均非该组合。

本地参考为 6.0.0 / UE5 / CL 0，未知上游 revision；没有运行 UE。这些哈希固定参考文件而非发行版本：

- `InterchangeImportTest.cpp`：`226ba896bdfda37392ef4c0c1ef9c1d48f6c928628b6f9fc574cc4461057e71a`
- `InterchangeImportTestPlan.cpp`：`c33e9cd020b6ec697a94c31666f88c726253532b9cd95291393b6f627189264d`
- `MainFrameHandler.h`：`7af5ae26133131a34d36d69c6a23177e9e7b51f9a85b7ad75e781d5c05c51794`
- `TasksTest.cpp`：`ea053c22aea351be0429279101ca3efc2ec6cb8fed4db69cb5b8b61609f8b7a9`
- `InterchangeManager.cpp`：`03010245c202690106f6fdd4b88ff58d39bb63ca897a30fbd3078b5a6342c8bb`
- `InterchangeTaskCompletion.cpp`：`f76564609370c37d0f578a2ac825bfb5e09c4276510176704c4b6264642fb430`

## 7. 在原 owning Editor 宿主上的首次执行

[固定的仅测试提交](https://github.com/He-Jiahui/ZirconEngine/blob/781eadc434b1ad1705ba245d2c9080ed457aa64e/zircon_editor/src/ui/retained/app/tests/model_close.rs)仅增加该问题的回归及挂载。Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib`，Rust 的 `test` profile（包内依赖需要 `target-editor-host`、`test-support`）。过滤 `model_close_owner_`，`--ignored --test-threads 1 --show-output`；专用物理项目、最近工程与用户设置，不借用实际用户状态。

真实 retained libtest 两例：0 过、2 败。未观测到预设的错误行为签名。

两例都在切换到新项目的 authoring scene 时遭到同一代际拒绝，尚未发起 OBJ 导入或主窗口关闭；不能把这两次失败当成关闭所有权问题的 RED。

无导入对照和“导入已持久提交”主案分别创建独立 RenderableEmpty 输入，正式 `create_project_and_open` 成功返回，随后实际 `apply_startup_session` 返回 `the authoring scene belongs to gateway generation 1, but generation 2 is current`。两次失败位置相同。没有 `MODEL_CLOSE_OWNER_READY` 或 `MODEL_CLOSE_OWNER_UNEXPECTED_REVIEW`，也没有成功对照证明无导入时能正常退出。没有提前消费票据、关闭 watcher、删除代际检查或改用伪造的 Manager。

这一结果指向项目初始会话与已在运行宿主的切换边界；不能由此认定默认桌面用户必定无法打开项目。后续须独立核准真正冷启动并已绑定文档的宿主，在相同存储、原生主窗回调和真实 worker 至少均到达后，再判断关闭缺陷及是否应该修生产。

编译回执 `fe062feca2b7e6a6bf3fefb93adb2a5bdc7f6a43fce5d93e43516f643cce7f11`；执行回执 `e6f619b96ddce0bbf52a13e12d4424ac8d826a1497c797573bc1e8fe4c871328`，stdout `8171daf9c1ebf9e0d733218268804e0a879b338f511d983874b103aea0b1512e`。

未改生产逻辑；没有把其它 PR 的修复叠上来。本阶段不是 UE、GPU 或 stock 桌面端到端验证。

## 8. 无导入主窗可退出；另一例卡在提交真实模型之前

[原样保留两条行为断言的冷启动测试](https://github.com/He-Jiahui/ZirconEngine/blob/74b4702ca61da912f18189d06cf804fb92dc1568/zircon_editor/src/ui/retained/app/tests/model_close.rs)不再把旧 World 装到新 generation。正式项目预检直接进入 retained 工作台，两例均核到同根项目、active Scene document 及同一 Edit World。**无导入对照通过**：原生 close callback → 正常 tick → 协调器 Closed、Jobs Quiescent、活动持有者释放、退出标记成立，而不是仅看到第一步的 KeepWindowShown。

**导入例未到行为断言。** 状态行是 `Project opened: CloseModel`，其中 `project_generation=27 assets=7 ready=7 failed=0 registry_diagnostics=1`。这并不等于 Editor 已能按 URI 看见启动 Scene：夹具在「Editor 能取得 Scene 行、且没有待处理的 Scene reload」的联合门前用尽预算。这个结果本身不区分哪个谓词未满足，不能把所有原因归给 [#202](https://github.com/He-Jiahui/ZirconEngine/pull/202)。已留项目仍有 Ready 会话记录，注册表未出现目标 `close-triangle.obj`；没有提交测试 OBJ、也没有为此例调用原生关窗。不能宣称永久死锁、菜单不可用，或模型结果已由关闭所有者接收。

Windows x64/MSVC 1.94.1；`zircon_editor --no-default-features --lib --profile test`，随后实际运行 `model_close_owner_ --ignored --test-threads 1 --show-output`。**1 过 / 1 败**；编译 `401262d49e19327f44b41dcb6072dbb8c16bc91a3f85de7be6bbee9dde192972`，执行 `f7db1c8f92e893d398a4845790ca9a7fc828652498102a4a0a56f813db626cc1`，stdout `ef241790d7fd553f963adc824308ccb6e9676203b6bac2b525a311b37ba1ebdf`。下一步分开采样真实注册目录和 reload，保留 watcher、权限、会话身份、工作线程及关闭断言；未取得同一终态任务的 READY/review 证据前不改生产。
