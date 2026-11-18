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
