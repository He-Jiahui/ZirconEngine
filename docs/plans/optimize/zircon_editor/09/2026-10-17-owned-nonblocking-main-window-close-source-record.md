# Editor09 主窗口关闭：源码优化记录

日期：2026-10-17。任务：optimize-20261013-01a13451。

本轮已把拥有完整生命周期的关闭操作接入实际主窗口。40 个 UI 及其依赖源文件已写入实际 checkout；关闭操作保留对应的 Session、场景和文档关闭能力、最终自动保存及设置持久化 owner，在真实后台任务结束前继续保持 Pending。原生、GUI 和性能验收仍未执行，本记录只登记源码进展。

## 已写入的行为

- 场景和文档分别取得精确的关闭能力。普通编辑入口在关闭期间被拒绝；最终保存使用与激活、操作和修订对应的能力。
- 后台捕获前先取得独占的单文档许可；保存负载按实际 Vec.capacity 计入占用，并把许可保留到写入结束。UI 关闭轮询只检查必要的状态和元数据。
- 自动保存、共享 jobs、设置持久化和 ProjectClose Session 依次推进。结果通道可读或队列变空不代表真实运行任务终止；错误、期限到达及尚未结束的任务继续保留 owner 和唤醒路径。
- Core v2 已修复诊断提交后误确认后续队尾、历史失败跳过其余取消操作、以及持锁提交时同步观察者重入三处问题。该部分已有独立的限范围源码审查，原生用例仍为 UNRUN。
- 并发出现的 app.rs 外观令牌句柄及观察者格式化改动已保留，未归属为本任务成果。

## 源码证据

UI 候选 patch：editor09-owned-nonblocking-main-window-close-v1-candidate.patch，97290 bytes，SHA256 281a7e2822ccd11368aa62c82d127c1679ab2b96929ab03b062ff16ea5275b77。

UI binding：39010 bytes，SHA256 28b3c376f6e3cea0b22a2f562e16ce05a074aecee46029dce891f90b17ebf5fb。UI source receipt：51807 bytes，SHA256 6e4e0c6ac56890c4af2addd2dd82a3c11414c851ed12d27b6427f3e4760fd874。三者均位于 .codex/outbox/reports/optimize-20261013-01a13451/。

实际 UI writer：PowerShell 32084 / Python 37628，chunk fff957，outer exit 0；40 个文件写入、fsync、双次原始字节读回完成，7 个只读 Git 子进程均已 communicate/wait 并退出 0。receipt 中的父进程待退出描述由该外部真实终态补充，未改写历史 receipt。

Core v2 source receipt：71785 bytes，SHA256 14b4ff171d39b06918994b04be13aa1065a49261928bf943c8f0b9c7e961bd49；writer PowerShell 31512 / Python 33880，chunk 39cbf0，outer exit 0。独立源码 reader PowerShell 25320 / Python 12716，chunk 0f3da6，outer exit 0。

## 验证与未完成项

新增五个主窗口行为用例，其中三例使用真实项目、admitted Session、激活凭据与 heartbeat，覆盖可逆 Cancel、quiesce 后版本变化保留能力、真实 Pending 资产任务与同 tick legacy poll。与 Runtime shutdown、Settings、Core v2、Runtime220 回归合批，全部仍为 UNRUN。既有输入夹具和两项项目关闭测试保留旧身份。

保留既有 31 条 Cargo、2 条 Python、原命令 floors、历史精确 named/subset 门槛、原十七个新增身份、22 个性能组及 Runtime03 绝对 P95 和原相对门槛。给原 runtime row2 与 editor row3 追加选择条件，不能用新增用例补足旧 floor。旧 B25 已有源码漂移且 launcher inactive，需由 Jenkins owner 绑定后续源码和执行环境；落盘交接不代表 queue ACK、执行或通过。

独立 UI 审查确认了一个 P1：真实 quiesce 已使 Session Closing、停止 heartbeat 并提交部分 effects 后，版本失配仍显示不能恢复这些状态的 Cancel；预 quiesce Cancel 也没有退掉旧 owner。生产窄修已实际写入 main_window_close.rs 与原四十文件之外的 prompt_actions/actions.rs：仅真实可逆状态允许复核/取消，Pending 即同步真实 operation，post-quiesce 失配保留能力并进入 Recovery。实际 writer PowerShell 39612 / Python 48788 / 65a2e9，outer exit 0。main_window_close 当前 21219 bytes / SHA256 926476c75ac80c3e8d14616bc2e5f27f6888e223f6bf963f1a83dacbd7cbffb3；actions 当前 4342 bytes / SHA256 099faf0b9a7d722c1e94e0b741e129e1098f2762b0582aeb56092f543452fa8e。最初四十文件证据不改写，当前记录范围增加该真实 Cancel handler。

三个真实 Session 回归及两项旧测试修复已写入，独立增量窄审 SOURCE_ONLY PASS。中央 legacy commit 在主窗口关闭已请求且未 Complete 时返回 typed MainWindowCloseOwned，防止删除回调提前释放 Session；Complete 后允许 ui.run 原有清理。公共全 union guard、原生、GUI、序列化峰值、RSS、P95 继续 OPEN。容量记账不能代替测量。

本轮未创建构建产物、源码副本或临时 helper；证据按登记路径保留，实际释放空间 0 B。

## P1 增量与实际证据

本次增量登记 7 个 owner；当前记录包含 44 个不同源码路径。这是历史交付加精确增量的映射，不是同时点完整 checkout 验证。Jenkins 须绑定实际 union。

- `zircon_editor/src/ui/retained/app/main_window_close.rs`：21219 B，SHA256 `926476c75ac80c3e8d14616bc2e5f27f6888e223f6bf963f1a83dacbd7cbffb3`。
- `zircon_editor/src/ui/retained/app/native_window_close/prompt_actions/actions.rs`：4342 B，SHA256 `099faf0b9a7d722c1e94e0b741e129e1098f2762b0582aeb56092f543452fa8e`。
- `zircon_editor/src/ui/retained/app/tests/support.rs`：13346 B，SHA256 `d3b7db3c47c8f4d3c36ea9850499bf1e5f08bc5c735cee51d54de07c5bf1bc0a`。
- `zircon_editor/src/ui/retained/app/tests/close_prompt.rs`：46401 B，SHA256 `0841350822126f1885604b24581b781f01c8b183c71bcf6c048e5e6c96f4d7b5`。
- `zircon_editor/src/ui/retained/app/project_close.rs`：28278 B，SHA256 `eb057010db710c494c44d68681da1eb943ddaadb93ce2be75dd4bd49c017dde4`。
- `zircon_editor/src/ui/retained/app/assets/deletion.rs`：4806 B，SHA256 `7e0710684e5c57b4d580c3c6a2dac3af3ca73c35c81efc0c5090d3e0bd0f9a83`。
- `zircon_editor/src/ui/retained/app/tests/project_close.rs`：5633 B，SHA256 `307c4eaa920c24a2e7604e6388d92fb1e4deeb1f2f69d96bf3efdb577a35b938`。

六个生产/回归 owner 实际端点读回：PowerShell 5352 / Python 15400 / 3f6002，outer exit 0；旧测试最终读回：PowerShell 33920 / Python 36388 / c5f91a，outer exit 0。旧 tests/project_close.rs 七个测试身份不变，修复两处文本锚点，继续断言 Closing 与 Play teardown 先于 Session release。字节读回不是编辑 fsync 或原生通过。

独立审查基于 root676/root677 实际源码及作者最终读回，不是 reviewer 的新 OS read。Sender 在 context/harness 之前析构，断言失败会断开 worker recv；发送后真实终态观察另有 5 秒期限。

真实夹具计划位于物理 `E:/cargo-targets/zircon-local/zircon-test-output/editor09-final-close-p1-{cancel|recovery|pending}-{pid}-{nanos}`；使用 ProjectAuthority、EditorManager、Scene/Document 和 jobs。尚未创建或执行；执行前登记，先释放 harness 再只清所属唯一 leaf，失败证据保留。新旧用例合入现有 editor row3，旧 floor 保持原 selector/subset 谓词。
