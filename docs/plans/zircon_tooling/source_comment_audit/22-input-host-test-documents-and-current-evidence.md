# 22：输入宿主请求测试文档与当前证据复核

## 范围和状态

本阶段已在四个既有测试文件补入九个函数的中文文档、四处模块职责说明及一个可证明的缺陷标签。文档说明测试职责、使用时机、断言范围、局部夹具生命周期和参考接口；无参数测试不增加虚构的参数契约。既有注释、断言、程序行为和公共 API 均保留。

文件范围为 `zircon_runtime/src/input/tests/gamepad_bridge.rs`、`zircon_app/src/entry/runtime_entry_app/gamepad/tests/rumble.rs`、`zircon_runtime/src/input/tests/input_manager/host_requests.rs` 和 `zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs`。新增十四段共一百一十行；覆盖和问题逐项记录存放在精确输入账本，计划只记录阶段状态。

**本阶段源码注释复核完成；正式账本发布、Jenkins 阶段提交及全仓审查仍未完成。**

## 审查与证据边界

主审完整读取四个文件和九个文档候选，独立核对直接实现、父模块挂载、条件编译、宿主排空、ABI 转换及编码上下文。独立 agent 的链接复核覆盖全部七十七处 Markdown 链接、三十个目标和七十三段源码证据，正文中的链接也纳入检查。

`CR-INPUT-BRIDGE-0001` 已在错误断言附近标为 `BUG:`。当前源码守卫只读取两个父文件，漏读手柄子模块；首个输入变体断言可证明为假。核心 agent 另行确认了这条代码路径。该问题保持未修复，不把静态证明写成实际 Cargo 失败回执，也不把测试扫描错误归因于生产桥接缺失。

手柄轴批次还完成了独立 agent 终态与当前支持文件的复核。此前按钮设置新增八行注释造成支持文件整体哈希变化；主审仅在内存恢复精确旧字节、校验旧范围并映射当前行号。历史 seal、原始注释作者和本次新增注释来源保持分别记录。

## 验证和精确输入

四个测试文件的候选均通过 stdin `rustfmt`，输出与拟写字节一致；安装后 `git diff --check` 通过。独立最终检查确认全部八份输入记录的长度与 SHA256、四文件当前哈希、十四处新增片段、七十三段当前证据以及源码标签和问题账本的一致性。移除本次新增片段后，四个文件均恢复精确的审查起始哈希。未运行 Cargo、Rustdoc 或动态测试。

测试批次最终输入封包：

- 路径：`E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/gamepad-test-root-review-r1/final/seal.json`
- SHA256：`6b33c7a8192d3c47fc8eb104c7decfa617c8d534dd7dc5c62bf62adf53e3fc70`
- 其中 `current-coverage.jsonl` 绑定最终文件哈希；`comment-additions.jsonl` 分别记录原始候选作者和主审新增片段；`issue-ledger.jsonl` 记录未修复问题和后续动作。

手柄轴当前支持复核输入：

- 路径：`E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/axis-independent-current-reconciliation-r1/result/seal.json`
- SHA256：`6435a191fc47febffb9aacd47c7b3949f8bb9842e9b8bf8e34ea32f7d5bdb670`
- 复核范围：四十九个注释契约、一百四十三个定义绑定、十一路由、二十三段支持和五十五个 `Self` 作用域；不据此扩展全仓覆盖结论。

以上均为保留的任务审查与增量交接输入，不能当作 Jenkins 发出的源码或产物身份、业务采用、动态验收及提交回执。

## 交付和管理流程

正式目录 `docs/_data/source-comment-audit` 的写入授权已由用户同意，但当前配置及生效权限仍缺少该目录的写入项。精确覆盖和问题更新保留在已登记输入中，等待真实授权生效；未修改权限配置，也未以其他目录代替正式发布。

版本集成、生成流程和清理由 Jenkins 承接，遵守[产物管理标准](../../../tools/agent-artifact-management.md)及[源码增量交接规范](../../../../.codex/skills/jenkins-coordination/references/incremental-patch-validation.md)。Jenkins chat 的告知与精确阶段提交仍待可用入口及实际回执；未把 outbox 草稿标为送达。共享索引锁和退役协调器历史资料保持原状。

本阶段未删除产物，实际释放空间为零字节。原有审查输入、失败证据和未知归属内容保留；任务尚在进行，不能报告清理关闭。

## 下一阶段

继续输入公共契约六文件、默认输入管理器及状态实现的逐对象主审；关联帧状态、触控手柄、即时队列和回放测试保持独立范围。现有单位契约和不完整回放问题保留未决状态。只有正式清单中每个文件的最终哈希审查记录有效、待处理文件清零，并且源码问题标签与正式问题账本一致时，才可报告全仓完成。

## 参考

- [统一文档契约](04-documentation-contract-and-review.md)
- [手柄轴批次](20-gamepad-axis-documentation-contracts.md)
- [按钮契约澄清](21-gamepad-button-scope-clarifications.md)
