//! 动态会话的二进制接口、宿主请求和诊断路由需与共享契约同步。保存同组守卫使用的局部数据或辅助变换。
pub(in super::super) fn slice_between<'a>(
    source: &'a str,
    start_anchor: &str,
    end_anchor: &str,
) -> &'a str {
    let start = source
        .find(start_anchor)
        .unwrap_or_else(|| panic!("source should contain start anchor `{start_anchor}`"));
    let tail = &source[start..];
    let end = tail.find(end_anchor).unwrap_or_else(|| {
        panic!("source should contain end anchor `{end_anchor}` after `{start_anchor}`")
    });
    &tail[..end]
}
