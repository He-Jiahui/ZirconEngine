//! 资产源码守卫的共享辅助；只检查声明存在及裸颜色文本，不解析级联求值。
/// 原子与壳资产测试共同调用此守卫，要求导入主题并出现各自声明的token；语义求值另由解析与运行时验证承担。
pub(super) fn assert_tokenized_assets(assets: &[(&str, &str, &[&str])]) {
    for &(asset_name, asset_source, required_tokens) in assets {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        for &token in required_tokens {
            assert!(
                asset_source.contains(token),
                "{asset_name} must use {token} instead of a local component value"
            );
        }
        assert!(
            !contains_hex_color(asset_source),
            "{asset_name} must not reintroduce a naked hex color"
        );
    }
}

/// 给共享守卫识别源码中的六位十六进制颜色样式；这是文本启发式，不是ZUI词法解析器。
fn contains_hex_color(source: &str) -> bool {
    source
        .as_bytes()
        .windows(7)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}
