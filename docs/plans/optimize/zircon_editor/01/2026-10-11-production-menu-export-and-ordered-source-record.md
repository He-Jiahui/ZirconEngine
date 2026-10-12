# Editor01：生产菜单导出与完整 Source 闭包修复

日期：2026-10-11。两项修复已实际采用，native 与性能仍待统一批量验证。

## 生产导出

第四批三包编译检查报告 E0425：真实生产 `module_tools_menu_source_is_live` 调用 workbench 节点转换，但内部 re-export 被 `cfg(test)` 裁掉。实现和模块本来就参与生产编译。仅移除这一个属性，保留同一 crate-private 函数及所有行为。实际 formatter 只换行一个原有 import；完整逆改重现原 LF 文件。

源码范围：`zircon_editor/src/ui/retained_host/ui.rs`。三个输入检查、实际 edition2021 格式及 diff 检查通过，共享索引和旧锁的内容/元数据保持。采用回执 SHA256：`bff7b069824faa11a7bf2d9f135bcaa1f65d14227a9a5d16830b597072b7752a`。

## Tools 有序 Source 检查

实际六菜单/43页测试在两个现有选择点核对真实已挂载 Source：新 FileCache 不启用持久化，按生产解析器取得严格主题，再加载实际选中物理文件与主题。完整有序 Vec 比较覆盖五个身份字段和原始 SHA；原画面、命中、两次 typed input、Scene 退休和环境锁断言保持。

独审拒绝了直接拼开发目录的主题路径；修正后采用完整 v2。源码只改现有 `app/tests/module_tools_menus/support.rs`，17 输入核对、16 未改依赖、实际格式和 diff 检查通过。保留原六测试与43页遍历，新增 Rust case / 验证命令 / 最低数量增量均为0。

采用回执 SHA256：`e2c76c35d32450efc260325d1180efb2708ba8f02ec2f0f101daddcae2c647a7`。源码结构检查没有性能测量结论；下一批保留原15项命令/14性能producer及全部原预算。

[对应完成列表](../../../astra/features/editor/2026-10-11-editor01-menu-export-source-completion-list.md)。
