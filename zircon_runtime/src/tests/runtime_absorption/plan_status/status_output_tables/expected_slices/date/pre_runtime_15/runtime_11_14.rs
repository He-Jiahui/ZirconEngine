pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 14 animation runtime-status focused recheck timeout" {
        Some("2026-06-15")
    } else if slice == "Runtime 14 animation family 28-file audit sync"
        || slice == "Runtime 14 navigation fallback runtime owner split"
        || slice == "Runtime 07 owner-budget 36-hotspot navigation split sync"
    {
        Some("2026-06-17")
    } else if slice == "Runtime 14 Module family guard anchors 审计同步" {
        Some("2026-06-15")
    } else if slice == "Runtime 12 gamepad event-owner 漂移同步"
        || slice == "Runtime 01 Tech-stack 行为测试锚审计同步"
        || slice == "Runtime 02 core/root/generated 镜像文档守卫"
        || slice == "Runtime 02 guard-test anchors 审计同步"
        || slice == "Runtime 10 Dynamic API 行为测试锚审计同步"
        || slice == "Runtime 10 dynamic_api_session 吸收守卫拆分"
        || slice == "Runtime 12 Input stack 行为测试锚审计同步"
        || slice == "Runtime 04 Asset pipeline 行为测试锚审计同步"
        || slice == "Runtime 08 ECS 行为测试锚审计同步"
        || slice == "Runtime 05 status-output Runtime 08 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 12 gamepad event-owner row"
        || slice == "Runtime 05 status-output Runtime 12 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 04 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 03 module-doc row"
        || slice == "Runtime 05 status-output Runtime 03 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 10 behavior-test row"
        || slice == "Runtime 05 plan-status output-anchor module split"
        || slice == "Runtime 05 plan-status output-anchor budget guard"
        || slice == "Runtime 05 status-output status/date helper split"
        || slice == "Runtime 05 status-output expected anchor split"
        || slice == "Runtime 05 status-output row-data group split"
        || slice == "Runtime 05 plan-status root module split"
        || slice == "Runtime 05 plan-status support inventory split"
        || slice == "Runtime 05 plan-status anchor inventory split"
        || slice == "Runtime 05 plan-status markdown renderer split"
        || slice == "Runtime 05 plan-status source helper split"
        || slice == "Runtime 05 status-output expected row data split"
        || slice == "Runtime 05 full scene closeout failed evidence"
        || slice == "Runtime 05 full scene closeout no-result recheck"
        || slice == "Runtime 05 scene:: failure support-first triage"
        || slice == "Runtime 05 scene:: lower-layer diagnostic matrix"
        || slice == "Runtime 05 serialization source folder-split guard sync"
        || slice == "Runtime 03 world bootstrap fixed-loop stage guard sync"
        || slice == "Runtime 05 cargo-gates early Runtime 03 split"
        || slice == "Runtime 05 cargo-gates early Runtime 01 split"
        || slice == "Runtime 05 cargo-gates early Runtime 02 split"
        || slice == "Runtime 05 cargo-gates early Runtime 04 split"
        || slice == "Runtime 05 cargo-gates early Runtime 06 split"
        || slice == "Runtime 05 cargo-gates early Runtime 08 split"
        || slice == "Runtime 05 cargo-gates early Runtime 07 split"
        || slice == "Runtime 05 cargo-gates late Runtime 10 split"
        || slice == "Runtime 05 cargo-gates late Runtime 11 split"
        || slice == "Runtime 05 cargo-gates late Runtime 12 split"
        || slice == "Runtime 05 cargo-gates late Runtime 13 split"
        || slice == "Runtime 05 cargo-gates late Runtime 14 split"
        || slice == "Runtime 05 status-output all-index-row coverage guard"
        || slice == "Runtime 03 Schedule/frame-loop 行为测试锚审计同步"
        || slice == "Runtime 08 First-stage event update guard"
    {
        Some("2026-06-15")
    } else if slice == "Runtime 11 JobSystem 2026-06-20 验证窗口探测" {
        Some("2026-06-20")
    } else if slice == "Runtime 11 JobSystem core-min 验证窗口探测" {
        Some("2026-06-20")
    } else if slice == "Runtime 11 JobSystem current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 11 JobSystem inventory split" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 JobSystem Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 worker wait-assist" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 worker wait-assist core-min 验证窗口探测" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 worker wait-assist core-min test binary 验证" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 core-min test binary task/guard batch" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 ecs_schedule source-guard lifetime anchor repair" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 ecs_schedule core-min Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 tasks core-min Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 worker_pool core-min Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 rayon core-min Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 tasks default Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 worker_pool default Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 rayon default Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 ecs_schedule default Cargo 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 full-lib default Cargo closeout attempt" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 core runtime full-lib triage recheck" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 asset broader failure triage core-min 复验" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 full-lib default after asset triage recheck" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 full-lib default after graphics exposure retry" {
        Some("2026-06-21")
    } else if slice == "Runtime 12 cursor host requests" {
        Some("2026-06-20")
    } else if slice == "Runtime 12 input validation window recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 12 Input stack current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 12 Input stack inventory split"
        || slice == "Runtime 12 Input stack Markdown renderer split"
        || slice == "Runtime 12 input boundary grouped manager import guard repair"
    {
        Some("2026-06-21")
    } else if slice == "Runtime 14 module family current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 14 module family markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 13 Script binding current audit recheck" {
        Some("2026-06-20")
    } else if slice == "Runtime 13 script binding Markdown renderer split" {
        Some("2026-06-21")
    } else if slice == "Runtime 11 JobSystem 行为测试锚审计同步" {
        Some("2026-06-17")
    } else {
        None
    }
}
