---
related_code:
  - zircon_runtime/src/graphics/material
tests:
  - graphics material current source 5 of 5 Rust files and 885 lines reviewed
  - all 16 tests read; two source regressions added
  - common shading lookup and duplicate include witness changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, material/include scale counters and F2/plugin reload pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics material静态审查（2026-07-18）

## 当前源覆盖

`graphics/material/**`当前5/5个Rust文件、885行已逐文件静态阅读，16条测试已读：root 1/1、`shading_models/**` 4/4。覆盖builtin/plugin shading-model registry、token/id/channel validation以及plugin forward/GBuffer/deferred include source解析与template消费。

## 直接止损

`resolve_lighting_model`原对PBR/Blinn/Unlit也先由`as_token()`分配String，再在`resolve_token`无条件trim+lowercase产生第二份String。现三个builtin模型直接借用静态normalized token，registry先用borrowed `&str`查表，仅大小写不规范时构造lowercase fallback；Custom只按容量构造一次`custom:name`。material ensure与capture常见lookup不再分配。

plugin include source解析原对每个token把全部matching ready shader records收集进Vec，随后只读取前两个判断missing/duplicate。现使用惰性filter iterator并只取两个witness，移除临时Vec且保留duplicate诊断的前两个locator顺序。两条源码守卫与既有builtin/custom/missing/duplicate/template行为测试锁定语义。

## 剩余根因

PERF-MVP-358/404继续负责plugin include owner。`from_project_asset_manager`对每个plugin descriptor的forward/GBuffer/deferred token分别扫描全部ready shader records；每个candidate又重复trim、slash replace、lowercase、suffix format，并在命中后同步`load_shader_asset`及复制完整runtime WGSL。resource streamer、deferred pipeline和prewarm会各自构造source set，缺少shader registry generation上的token→artifact index与single-flight owner。

builtin registry本身只有3项、plugin ID范围有界，BTreeMap与registration duplicate检查是控制面，不在没有规模证据时单独编号。最终由Runtime04发布normalized include-token index与content-addressed parsed module Arc，Render08/material streamer按generation借用，Plugins01 reload/revoke只失效affected shading model；稳定scan/normalize/load/source clone=0。

本地Bevy specialization cache按key复用pipeline ID，UE shader job cache按输入hash去重并将DDC/worker工作异步化。采用“registry generation一次规范化、lookup借用、shader正文唯一artifact”的原则，不复制其注册API。

## 验收状态

5/5静态阅读、两条RED→GREEN回归、rustfmt、源码合同与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，16条测试没有current-source执行结果。descriptors/ready shader records 1/100/10k、stable/1% reload、lookup 1/1M下的token/String alloc、record visits、path normalize bytes、shader loads/source clone bytes、registry generation、CPU p95/RSS未量化；F2 custom shading与plugin reload产品路径也未执行，继续留在`pending.md`，不进入`review.md`。
