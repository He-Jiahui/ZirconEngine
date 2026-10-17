//! 将外部会话标识压缩为可跨平台使用的单级目录名。
//! 前缀会清理路径字符，哈希后缀用于区分清理或截断前的标识；本模块不创建目录或写文件。

/// 生成结果的最大字节数；前缀和十六进制哈希后缀均为 ASCII。
pub const PROFILE_SESSION_BASENAME_MAX_BYTES: usize = 96;

const PROFILE_SESSION_HASH_SUFFIX_BYTES: usize = 17;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Returns the portable, collision-resistant child basename used by every profile producer and
/// consumer. The readable prefix is ASCII so the byte limit is identical on every platform.
/// 返回所有 profiling 生产方和消费方共用的可移植子目录名。
/// 输入只用于生成字符串；函数不验证目录权限，也不保留输入引用。
pub fn profile_session_basename(session_id: &str) -> String {
    let sanitized = session_id
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect::<String>();
    let basename = sanitized.trim_matches('.');
    let basename = if basename.is_empty() {
        "session"
    } else {
        basename
    };
    let mut basename = if is_windows_reserved_basename(basename) {
        format!("session_{basename}")
    } else {
        basename.to_owned()
    };
    let max_prefix_bytes = PROFILE_SESSION_BASENAME_MAX_BYTES
        .checked_sub(PROFILE_SESSION_HASH_SUFFIX_BYTES)
        .expect("profile basename limit must leave room for its hash suffix");
    basename.truncate(max_prefix_bytes);
    format!("{basename}-{:016x}", stable_session_id_hash(session_id))
}

fn stable_session_id_hash(session_id: &str) -> u64 {
    session_id.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

fn is_windows_reserved_basename(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or(value);
    if ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|reserved| stem.eq_ignore_ascii_case(reserved))
    {
        return true;
    }

    let bytes = stem.as_bytes();
    bytes.len() == 4
        && (bytes[..3].eq_ignore_ascii_case(b"COM") || bytes[..3].eq_ignore_ascii_case(b"LPT"))
        && matches!(bytes[3], b'1'..=b'9')
}

#[cfg(test)]
#[path = "tests/session_path.rs"]
mod tests;
