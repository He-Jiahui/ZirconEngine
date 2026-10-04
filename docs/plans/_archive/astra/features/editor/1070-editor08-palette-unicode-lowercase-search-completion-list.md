# Editor1070：命令面板 Unicode 小写搜索

状态：最低支持层修复与回归已写入并归属；managed 测试与产品性能验收待完成。

优化记录：`docs/plans/optimize/zircon_editor/08/2026-09-30-palette-unicode-lowercase-search.md`。

## 完成列表

- [x] 定位文档 ASCII 小写与查询 Unicode 小写不一致，确认错误发生在 posting 索引之前。
- [x] 保留 ASCII 原缓冲区路径，写入 Unicode 文档小写修复及真实 catalog 查询回归；应用后格式检查通过。
- [x] 完成独立静态复核、当前字节与 archived 所有者准入；取得精确 live lease 后应用并登记两份源码。
- [ ] 通过这一批 Editor 的 managed 分组验证和真实查询回归，保留正常窗口、展示与 locale 缓存行为。
- [ ] 完成主计划所需完整 Unicode 规范化与字形匹配合同；当前局部小写修复不关闭全部 `E-CMD-P1-37`。
- [ ] 达到主计划规定的真实 10k/100k 查询和产品延迟、分配、RSS 门槛。

候选包 `.codex/state/session-coordinator/async-validation-batches/offline-candidates/editor08-palette-unicode-search-root-v2`，prepared SHA `0fe5f73fbf904084216201ce763495039ac7caa6fc6c0b18b48285a98fa0a486`。源码应用前 `b0aadad016d58c20dd94abaf7a066c07adaf84eeb92fd9215d198dd6ffb50e45`；应用后 `92db70fa00cbee06efe96529a52882ffc32bd2859144d35f9c56eb55af14f7e6`。新回归源码 `988a26aae65426e46680b4cd5b884a4002262baefd634494f8a26035b6f7af42`。Rust 回归尚未执行；格式检查和记录写入不构成测试或性能通过证据。

应用回执：`.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor08-palette-unicode-lowercase-two-source-guarded-apply.json`，SHA `1c30d45fd41a9643c69ae635b7ed9b1d3f5ff9b223c755ab11197a1229158801`。独立静态复核有效回执 SHA `7aee143f76a9aa2d7fedc1124ac3cd57a33eb39803c786340aec47209aae421e`；Root 只修复该回执尾部换行序列化，原始复核字节保留。以上回执不包含 Rust 测试和产品性能通过结果。
