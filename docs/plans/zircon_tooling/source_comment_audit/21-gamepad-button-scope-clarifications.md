# 21 手柄按钮文档作用域澄清

## 阶段状态

状态：本批次的私有复核、注释安装、局部语言检查和当前哈希绑定已完成；正式账本发布、提交及全仓审查仍未完成。

本阶段补充 [19 手柄按钮声明文档](19-gamepad-button-documentation-contracts.md)。依据 [04 声明文档规范](04-documentation-contract-and-review.md)，明确调用者如何使用比较与转换方法，并保留所有既有程序和注释字节。逐对象决定、引用与错误恢复记录位于独立证据包，本计划仅记录阶段状态。

## 已完成范围

| 文件 | 本次新增文档行数 | 当前行数 | 当前 SHA256 |
|---|---:|---:|---|
| `zircon_runtime/src/core/framework/input/gamepad/button/mod.rs` | 0 | 33 | `9774cb53f0a7f8cb231bd6a3c6574ca9d36ee94ba96ec3b41067b879dba15c6f` |
| `zircon_runtime/src/core/framework/input/gamepad/button/button.rs` | 0 | 254 | `397fad18c6b27a16d1ce923a0599bc1bde5f1802137ed8feca3e6973faf2efeb` |
| `zircon_runtime/src/core/framework/input/gamepad/button/settings.rs` | 6 | 273 | `4920c238ead3de411da8ec9941d8993c4bb4fb88464d51d4d307745f372f0430` |
| `zircon_runtime/src/core/framework/input/gamepad/button/axis_settings.rs` | 2 | 288 | `d89d6161ddb7888a0aa0459aaa7c79c815cd5a064e34cada825cbfecea900714` |
| `zircon_runtime/src/core/framework/input/gamepad/button/value_state.rs` | 0 | 76 | `918c09566ac33e31a95a47c2d6b1cac226fe34cac9a15c67bf07b7380697448e` |

四处加法式插入共八行。模块说明把当前状态实参限定到 `transition_for_value`，并说明两个独立阈值查询不读取状态。类型与方法说明把接受幅值后的顺序限定为 `DefaultInputManager` 的使用路径，保留显式调用者选择幅值尺度和维护状态的责任。`process_value` 首段增加职责摘要，原有返回说明及其顺序完整保留。

五文件共 924 行，沿用并重新绑定 Root 与 Core 各 96 条对象记录。旧 54 段注释逐字节保留；加上本次四段后，共 58 段、764 行注释，其中声明和模块文档 762 行、私有说明两行。完整文档的静态引用记录从 180 次增加到 183 次。

## 独立复核与验证

Root 完整读取新增文字、当前相关文档、默认管理器调用上下文及实际再导出，核对三项独立发现和方法级时机补充。Luna 对四处候选、三个真实 Markdown 链接、局部方法声明及再导出完成静态复核。链接能对应到源码定义；Rustdoc 页面与锚点没有执行验证。

两次失败复核包保持原样：首包的反引号比较和链接计数有误，第二包把 Rust `settings::{...}` 路线误写成单冒号模式。第三包只恢复后一项误判并使用同哈希的既有语义证据。失败包没有改成通过，也没有被用作安装验收。

本次两个文件分别通过 Rustfmt stdin 检查，输出与输入完全相同；`git diff --check` 返回 0。移除新增八行能还原前一批次的完整源码字节，再移除前一批次的精确注释片段能匹配原审查起点哈希。旧记录被新插入分隔时，使用保留片段逐字节映射，不将新范围整体重标成旧作者的注释。

程序行为、公共 API 和既有注释均未改变。没有运行 Cargo、动态测试、Rustdoc、产品捕获或旧协调器操作，没有创建完整源码副本，没有 Git 写入或外部通知。

## 证据位置

私有证据复用 `E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168`。原阶段证据仍位于原目录，没有搬迁、覆盖或重封。

- [本次精确候选](E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/button-contract-clarifications-r1/candidate.json)：SHA256 `aa200a2e21f61fcff3c207053e4ecc109d9c89f76440f540228dfaeae1c9ae72`。
- [有效独立复核封条](E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/button-contract-clarifications-r1/luna_recheck_r3/seal.json)：SHA256 `fe14dd3600418e115f166c5abeed0cb2afff3af0232ea089cd562863a54bc6a8`。
- [本次最终封条](E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/button-contract-clarifications-r1/final/seal.json)：SHA256 `b6a5016c8bfc468b6199327f33cfc3737173c3f0189926d13dd09b3d127f298e`，包含 12 个独立载荷。
- [语言回执](E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/button-contract-clarifications-r1/final/language.json)、[当前五文件覆盖](E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/button-contract-clarifications-r1/final/coverage-final.jsonl) 和 [保留注释片段映射](E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/button-contract-clarifications-r1/final/documentation-preserved-fragment-bindings.jsonl)。
- [产物登记](E:/cargo-targets/zircon-local/reports/source-audit-20260930-01a0f168/artifacts.json)：由 Root 持续维护，保留依赖、必要失败证据和未知历史产物的核对状态。

## 待完成边界

原六项私有观察保持开放，没有增加正式 `BUG:` 或 `TODO:` 标签，也没有关闭既有问题。正式账本目录的用户授权已保留；当前会话的有效文件权限仍未包含该目录，本阶段没有越界写入。

提交、共享索引归属和 Jenkins 实际验收继续依现行仓库流程处理。旧协调器保持退役，私有语言回执不能替代尚未取得的实际验收。其他批次继续独立推进，不据此宣告全仓完成。

本次安装进程已经自然退出。必要交付与失败诊断继续保留，未清理未知或其他会话产物；实际释放空间为 0 字节，任务及产物清单仍为 active。
