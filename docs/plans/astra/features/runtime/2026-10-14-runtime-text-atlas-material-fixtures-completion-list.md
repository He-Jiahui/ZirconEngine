# Runtime：TextAtlas / 材质夹具修复完成列表

状态：`source_reviewed / native_performance_pending`；详情与最终哈希见[优化记录](../../../optimize/zircon_runtime/2026-10-14-functional-fixture-contract-repairs-record.md)。

- [x] CR-TEXT-RASTER-0001：8×8/64B 合法源保持 glyph cap=1 的队列溢出边界。
- [x] CR-TEXT-RASTER-0002：8×6/48B 合法源保持 byte cap=64 的字节溢出边界。
- [x] 原 source identity、queued/overflow 字节计数、page/padding 与全部其他断言保留。
- [x] 材质 byte204/248 分别校验 cos/sin，保留 0.4 弧度与 256B ABI。
- [x] 原 ordinary 身份、生产 witness、fmt/diff、root guard 及独立静态复核完成。
- [x] 三个实际修复用例与旧选择无重叠，确定第14批待交接选择和 Runtime ordinary 下限94。
- [ ] 实际批量原生回执、full-input seal、全部必需用例及计数通过。
- [ ] 所有适用原性能门槛与资源预算通过后关闭历史 BUG 与整体验收。

静态和交接不代表 native 或性能通过。未创建构建/缓存/原生夹具或源码副本；保留必要交付证据，实际释放 0B。
