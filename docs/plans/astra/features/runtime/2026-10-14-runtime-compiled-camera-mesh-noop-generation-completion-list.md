# Runtime compiled Camera/Mesh 计划完成列表

对应 [2026-10-14-compiled-camera-mesh-noop-generation-record.md](../../../optimize/zircon_runtime/2026-10-14-compiled-camera-mesh-noop-generation-record.md)；Runtime63、Camera37/99za、Mesh69。

- [x] Camera3 / Mesh6 相同值不进入可变 World 访问，不增加 generation。
- [x] 保留原 stale/缺失组件/转换错误顺序与 morph 真实扩容。
- [x] 保留原 22 个普通测试；新增 5 个 public-write 回归。
- [x] scoped fmt/diff、exact2 change guard、独立源码复核通过。
- [ ] Jenkins exact-source 27 ordinary / 0 failed / 0 ignored 及合批测试通过。
- [ ] 原性能矩阵、RSS 与冻结 Runtime03 绝对 P95 验收通过。
- [ ] 产品与 BUG 闭环完成。

状态：source/static 完成，native/performance/product pending。未运行原生测试，未宣称已测 P95/RSS 收益；实际释放空间 0 B。

