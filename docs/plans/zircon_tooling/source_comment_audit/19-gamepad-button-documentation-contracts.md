# 19 — 手柄按钮数据与阈值接口的文档契约审查

## 本阶段范围与状态

审查 `zircon_runtime/src/core/framework/input/gamepad/button/` 的五个第一方 Rust 文件。文档补充、独立主审复核、范围内格式／解析检查与最终内容绑定已完成；覆盖证据暂存于获授权的私有目录，正式账本发布与阶段提交仍待处理。全仓目标继续开放。

遵循[统一文档规范](E:/Git/ZirconEngine/docs/plans/zircon_tooling/source_comment_audit/04-documentation-contract-and-review.md)，承接[第 18 阶段](E:/Git/ZirconEngine/docs/plans/zircon_tooling/source_comment_audit/18-gamepad-device-and-rumble-documentation-contracts.md)。九个函数都有 Markdown 概要、返回、使用时机、必要约束、所有权与生命周期及参考；七个有参函数另有参数章节，两个无参默认构造不虚构参数。类型、字段、常量和变体说明各自职责、有效域及状态拥有者。匿名闭包只在 `//` 中解释非显然状态不变量。

## 调用关系与设计证据

五个起始文件完整读取，96 项 Core 记录与 96 项主审记录逐一对应。覆盖模块与重导出、导入、四个类型、18 个变体及一个元组载荷、八个字段、五个常量、四个实现、九个函数、派生 trait、构造、比较、分支与原有注释。

主审独立读取 29 个支持路径、34 个范围，追踪 App 的设备事件与协议码、ABI 输入转换、动态宿主路由、默认输入管理器、按钮集合、动作求值及帧快照。说明限定到实际实现：先筛选幅值，再判断布尔按压；被过滤的采样保留最后接受值作为基线；按钮动作强度当前由布尔状态产生；宿主 UI 导航有独立路由。

文档明确区分构造建议与实际验证、原始采样与缩放历史、候选状态与已应用状态、拥有式快照与实时设备状态。非有限配置、重叠阈值、未知 `u32` 协议码到 `Other(u16)` 的截断边界均按可证明的当前路径说明，未编造业务意图。180 处实际文档引用按定义及可见作用域静态核对，并映射到最终定义范围；Rustdoc 构建未执行。

## 注释改动与最终内容

新增 54 段、756 行注释，其中 754 行为文档注释、两行为私有不变量说明。五个文件由 160 行变为共 916 行，六条原有注释完整保留。逐文件在 RAM 中移除本次精确插入，恢复起始哈希、字节数和行数；原程序与公共声明字节没有改变。

| 文件（均位于 `zircon_runtime/src/core/framework/input/gamepad/button/`） | 起始／最终行数 | 最终 SHA256 |
| --- | ---: | --- |
| `mod.rs` | 16／33 | `9774cb53f0a7f8cb231bd6a3c6574ca9d36ee94ba96ec3b41067b879dba15c6f` |
| `button.rs` | 23／254 | `397fad18c6b27a16d1ce923a0599bc1bde5f1802137ed8feca3e6973faf2efeb` |
| `settings.rs` | 46／267 | `ec90ef98e7efbcc50ba35f7219e34adcd2f09527f04c51bb7c7e11983768753a` |
| `axis_settings.rs` | 63／286 | `c54c62c38b498a8f2e0f5118ceb5d21650087d251eb3b89226a720fa7a0abaea` |
| `value_state.rs` | 12／76 | `918c09566ac33e31a95a47c2d6b1cac226fe34cac9a15c67bf07b7380697448e` |

## 复核、验证与证据索引

54 段最终文本与五文件实际 diff 分组完整读取，超出输出预算和元数据键误用的读取另留失败与恢复证据。五次限定范围的 Rustfmt stdin 解析／格式比对通过，行尾归一化后的格式精确匹配；限定文件的 `git diff --check` 通过。最终绑定复用同内容的语言证据，没有新增 Cargo、动态测试、硬件操作或完整源码副本。

- [Core FIRST aggregate](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w25_gamepad_button_contracts_core_sol_r1/seal.json)：`732cfb940d39f118ee97abd01993f924ae5ee7596f29df60e4f9e8cc52bb36c7`；五个单元封条保持冻结。
- [主审 r3 封存](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-button-contract-review-20261009-r3/seal.json)：`dd3506c3a2ddbe9adfa0ec3cb184d86b8126860c1349d966d4604d8fb47fbc19`。r1 的语法失败和 r2 候选均保留；r3 收紧匿名闭包说明，具名函数契约不受削减。
- [注释安装与语言证据](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-button-declaration-docs-review-and-language-20261009-r3/seal.json)：`f49b628905c414d490e98aa3d22ee2c156aa96e7efc825f98705026b462fe1ac`。
- [最终覆盖绑定](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-button-final-coverage-20261010-r1/seal.json)：`8cebe2c3618675757f9bda1d61da17eab09b3916af967ce38178bf0a3ca7e0a3`；结果 SHA256 为 `7c975df6277b334469c2d27954fba63c701963417fb24536bb11aa6632371586`。
- [逐文件最终记录](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-button-final-coverage-20261010-r1/coverage-final.jsonl)保留起始／最终哈希、两级审查数量、精确字节还原与待发布状态。
- [问题边界记录](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-button-contract-review-20261009-r3/issue-decisions.jsonl)保留六项调用或意图缺口观察；本批没有新增正式 `BUG:`／`TODO:` 标签，也没有关闭这些观察。

## 来源边界与待完成事项

Core 的 255 条逐单元支持记录保留首次哈希、读取范围与回执。`dynamic_api/session/events.rs`、`state.rs` 的外部业务变动，以及第 18 阶段 `device/id.rs` 的文档变动，分别保留历史与当前读取记录。原支持哈希不重新命名为当前全文审查、注释作者证据或业务采纳。本次新增作者范围仅为表列五个文件的精确注释插入。

阶段提交仍为零；未知拥有者的 `.git/index.lock` 保留。正式账本的用户授权持续有效，但当前权限清单缺少该目的地，私有记录尚未发布。旧协调器保持退役，Jenkins 说明仍在 outbox，未发送或取得接受回执。产品排除项及 Editor06、Runtime11、SDK、Sound 历史边界保持有效。

Luna 静态引用辅助审查和四文件相关测试的补充审查继续；它们不写源码，结果仍须独立主审接受。已封存的轴、输入服务与快照、模块构造及公共外观批次继续逐项复核。只有全清单最终内容覆盖、问题账本一致性及实际提交／接受门槛均满足后，才报告全仓完成。
