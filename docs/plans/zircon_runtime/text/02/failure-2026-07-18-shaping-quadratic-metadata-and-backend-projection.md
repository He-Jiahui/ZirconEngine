---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: shaping-quadratic-metadata-and-backend-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/shaping
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/font/database.rs
---

# Shaping二次扫描、重复backend与worker状态放大

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/shaping/**`当前源18/18 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md`
- 联动责任：face/cache/worker联动Text01/Text09。
- 交接原因：BIDI/script/linebreak/backend与glyph projection由Text02拥有；shared font bytes和worker总预算分别需要Text01/Text09协作，不能在性能计划复制owner。

## 失败现象与复现证据

PERF-MVP-234：UAX14 opportunity、script segment与line start原来分别按glyph/line全表重扫；前三项已直接改为partition/precomputed index。horizontal/vertical backend projection仍按每个backend glyph扫描全部boundary与source glyph，并collect overlap Vec，最坏O(G²)+O(G)临时分配。BIDI line order重复计算reordered levels，fallback spans按grapheme临时codepoint Vec/family String。

PERF-MVP-235：service与cosmic各构建一次BidiInfo；cosmic Advanced shape后，language/variable horizontal及vertical upright segments再次RustyBuzz shape，并逐段重建face/variations/features/buffer。thread-local cache每worker持最多4 locale FontSystem，generation变化由caller同步重建。静态证据见`docs/plans/performance/01/2026-07-18-text-shaping-static-review.md`。

## 最低共享层根因

shaping pipeline没有一份贯穿itemization、BIDI、script、fallback、backend与projection的indexed paragraph context。各阶段各自从String/ranges重建查找结构；为补cosmic未暴露的language/variation/vertical细节，又在已shape glyph上执行第二backend。FontDatabase暴露bytes而非generation-owned parsed face，使segment backend重复构造资源。

## 架构修复验收

- 保留已落`partition_point` line-break/script与precomputed line starts；源码门禁不得回退全表fold/find/prefix scan。
- paragraph context一次构建BIDI levels、line starts、break opportunities、script/fallback segments，并以cursor/index供各glyph阶段消费。
- horizontal/vertical backend projection用sorted cluster boundaries与two-pointer/interval cursor；每backend glyph只访问重叠source cluster，禁止filter+collect全source Vec。
- BIDI base/levels/line order共用一个analysis，line visual/logical结果从同一次reordered levels派生。
- 选择唯一shape backend：language/variation/vertical进入一次shape；禁止cosmic Advanced输出后再整segment RustyBuzz shape。若保留cosmic layout，只消费其未被二次替换的结果。
- Text01提供generation-owned shared bytes/parsed face/instance；同face多segment copied bytes=0。Text09限制per-worker locale systems总数/bytes，generation refresh有counter并移出敏感caller路径。
- 1/100/1k/10k Latin/CJK/RTL/vertical记录metadata/projection visits、overlap alloc、BIDI/backend calls、face bytes、FontSystem bytes与refresh time；复杂度近O(G log N)或O(G)。
- locl/variable/kerning/features、fallback face/instance、TTB/BTT、source/visual range、UAX9/14与产品像素全部等价。

## 禁止临时方案

- 不得只给二次扫描预留Vec capacity；访问复杂度与per-glyph collect必须一起消除。
- 不得关闭language/variation/vertical二次backend而丢失locl/vmtx精度；应收敛为一次正确shape。
- 不得让每worker无限增加locale FontSystem或以更多线程掩盖重复shape。
- 不得缓存borrowed RustyBuzz Face跨越font generation；shared parsed face必须携generation/face identity并可失效。

## 修复结果与回传

Open state: `PERF-MVP-234已直接修复line-break/script/line-start三处并通过静态门禁；PERF-MVP-234其余projection与PERF-MVP-235 single-shape/face/worker预算等待Text02联动Text01/09回传规模counter、current-source Cargo与产品trace`。
