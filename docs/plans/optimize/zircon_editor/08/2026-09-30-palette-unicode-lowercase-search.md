---
title: Editor08 命令面板 Unicode 小写搜索记录
category: zircon_editor
date: 2026-09-30
implementation_status: source_applied_attributed
validation_status: managed_editor_tests_not_run
performance_status: product_gates_open
---

# 命令面板 Unicode 小写搜索

本次修复主计划 `E-CMD-P1-37` 中已确认的局部缺陷：`search_document` 只执行 ASCII 小写转换，查询却执行 `trim().to_lowercase()`。因此非 ASCII 大写标签或关键词没有进入与查询相同的小写索引，实际 catalog 查询可能丢失匹配。

修复已应用到文档生成这一公共支持层：ASCII 文档沿用原缓冲区的 `make_ascii_lowercase()`；非 ASCII 文档使用与查询相同的 `str::to_lowercase()`。既有 locale 缓存、UTF-8 字节 posting 索引、排名、MRU、查询窗口和展示文字保持原合同。Unicode 字符可在小写转换时扩展，字符串转换还处理希腊 Sigma 的上下文规则；实现使用项目既有的稳定 API。[Rust 字符串文档](https://doc.rust-lang.org/std/primitive.str.html#method.to_lowercase)

## 源码与回归

源码路径：`zircon_editor/src/core/commands/palette.rs`。应用前 SHA `b0aadad016d58c20dd94abaf7a066c07adaf84eeb92fd9215d198dd6ffb50e45`；应用后 SHA `92db70fa00cbee06efe96529a52882ffc32bd2859144d35f9c56eb55af14f7e6`。独立新回归文件 `zircon_editor/src/core/commands/palette/unicode_search_tests.rs` 应用后 SHA `988a26aae65426e46680b4cd5b884a4002262baefd634494f8a26035b6f7af42`。

三个回归使用真实 `EditorLocalizationBundle → EditorCommandDescriptor → EditorCommandRegistry → catalog.query_window` 路径，覆盖非 ASCII 大写标签、扩展小写映射、希腊全词、关键词与既有 trim 行为，以及 ASCII 匹配和空查询的有界窗口；核对实际命令 ID 和未改变的展示标签。既有活动会话的 `localization_tests.rs` 保持当前字节。

候选包 `.codex/state/session-coordinator/async-validation-batches/offline-candidates/editor08-palette-unicode-search-root-v2`，prepared SHA `0fe5f73fbf904084216201ce763495039ac7caa6fc6c0b18b48285a98fa0a486`。两份源码已通过独立静态复核、当前字节和公共归属准入、精确 live lease、格式与 scoped diff 检查，并完成 baseline attribution。Rust 编译、回归执行和产品性能验收仍待完成。

应用回执：`.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor08-palette-unicode-lowercase-two-source-guarded-apply.json`，SHA `1c30d45fd41a9643c69ae635b7ed9b1d3f5ff9b223c755ab11197a1229158801`。独立静态复核有效回执 SHA `7aee143f76a9aa2d7fedc1124ac3cd57a33eb39803c786340aec47209aae421e`；Root 只修复该回执尾部换行序列化，原始复核字节保留。以上回执不包含 Rust 测试和产品性能通过结果。

## 剩余验收

此局部修复没有实现 NFKC、规范等价、locale 专属大小写或完整 Unicode case folding。字节级 fuzzy 合同保持现有行为，主计划 `E-CMD-P1-37` 的完整语义仍开放。

后续将真实回归与这一批 Editor 改动一并提交 managed Windows 验证，提交后继续独立修复，保留原始回执，不逐项编译或持续监控。10k/100k catalog、真实产品交互延迟、分配和 RSS 门槛没有本次运行证据。

主计划：[Editor08](../08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md)。完成列表：`docs/plans/astra/features/editor/1070-editor08-palette-unicode-lowercase-search-completion-list.md`。
