---
source_report:
  - docs/plans/performance/01/2026-08-23-first-party-runtime-plugin-catalog-projection-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# First-party runtime plugin catalog投影受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：新增`zircon_plugins/first_party_runtime_catalog/src/**` current 5/5文件、1,239行、11 tests静态覆盖；plugin audit全绿，但Cargo/startup allocator/WPR/power仍open。本Session不直接编辑受保护ledger。
- Plan02 M1/M5 + PERF-MVP-629 + Plugins01：把该crate标为首方provider producer；建立唯一`FirstPartyProviderCatalogGeneration`，compiled provider descriptor/registration/validation owner=1，project/profile仅发布selection handles/ranges。
- Runtime06/11 + Plugins01/11：catalog+project+target generation使用per-key single-flight；不同project不被全局mutex互相驱逐，candidate在锁外构建并一次publish，reload只重建affected provider/selection。
- Plugins12：用generated dense provider slot替换O(S*P) typed-if分派；open third-party ID继续返回None。删除强制HashSet/if/no-match拼写的源码形状测试，以duplicate/order/unknown/alloc/build counter行为门替代。
- Plugins12测试维护：手写TOML parser与重复String projection后续改为消费cargo-zircon typed generated snapshot/标准parser；此项只按测试编译执行数据排序，不抢占产品catalog generation。
- `docs/plans/performance/review.md`：只有current Cargo、0/1/15/100 provider与1k manifest/100 project矩阵、allocator/WPR/power通过后迁入；该非渲染切片不要求RenderDoc。本轮不迁移、不提交milestone、不发送完成企微。
