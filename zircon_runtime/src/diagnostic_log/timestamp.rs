/// 为日志候选目录后缀和每批日志行前缀生成秒级本地时间字符串。
pub(super) fn current_log_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}

pub(super) fn append_current_log_timestamp_prefix(buffer: &mut Vec<u8>) {
    append_log_timestamp_prefix(buffer, chrono::Local::now().naive_local());
}

pub(super) fn write_current_log_timestamp_prefix(
    output: &mut impl std::fmt::Write,
) -> std::fmt::Result {
    write_log_timestamp_prefix_to(output, chrono::Local::now().naive_local())
}

fn append_log_timestamp_prefix(buffer: &mut Vec<u8>, timestamp: chrono::NaiveDateTime) {
    write_log_timestamp_prefix_to(&mut LogTimestampBytes(buffer), timestamp)
        .expect("writing a diagnostic log timestamp to a byte buffer");
}

pub(super) fn write_log_timestamp_prefix_to(
    output: &mut impl std::fmt::Write,
    timestamp: chrono::NaiveDateTime,
) -> std::fmt::Result {
    timestamp.format("[%Y-%m-%d-%H-%M-%S] ").write_to(output)
}

struct LogTimestampBytes<'a>(&'a mut Vec<u8>);

impl std::fmt::Write for LogTimestampBytes<'_> {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.0.extend_from_slice(value.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, NaiveDate, TimeZone};

    use super::append_log_timestamp_prefix;

    #[test]
    fn direct_timestamp_encoding_preserves_calendar_boundaries() {
        let timestamps = [
            NaiveDate::from_ymd_opt(1970, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(1999, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
            NaiveDate::from_ymd_opt(2000, 2, 29)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2001, 3, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            NaiveDate::from_ymd_opt(2016, 12, 31)
                .unwrap()
                .and_hms_nano_opt(23, 59, 59, 1_500_000_000)
                .unwrap(),
        ];
        for timestamp in timestamps {
            let expected = format!("[{}] ", timestamp.and_utc().format("%Y-%m-%d-%H-%M-%S"));
            let mut buffer = b"previous batch\n".to_vec();
            let start = buffer.len();
            append_log_timestamp_prefix(&mut buffer, timestamp);
            assert_eq!(&buffer[..start], b"previous batch\n");
            assert_eq!(&buffer[start..], expected.as_bytes());
        }
    }

    #[test]
    fn direct_timestamp_encoding_preserves_local_offsets_and_variable_year_width() {
        for (year, offset_seconds) in [
            (-10_000, -8 * 3_600),
            (-1, 5 * 3_600 + 45 * 60),
            (0, 3_600),
            (9_999, -3_600),
            (10_000, 5 * 3_600 + 30 * 60),
        ] {
            let local = NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let timestamp = FixedOffset::east_opt(offset_seconds)
                .unwrap()
                .from_local_datetime(&local)
                .single()
                .unwrap();
            assert_ne!(timestamp.naive_local(), timestamp.naive_utc());
            let expected = format!("[{}] ", timestamp.format("%Y-%m-%d-%H-%M-%S"));
            let mut buffer = b"before ".to_vec();
            let start = buffer.len();
            append_log_timestamp_prefix(&mut buffer, timestamp.naive_local());
            let prefix_len = buffer.len() - start;
            assert_eq!(&buffer[start..], expected.as_bytes());
            if !(0..=9_999).contains(&year) {
                assert!(prefix_len > b"[2026-05-04-16-30-00] ".len());
            }
            buffer.extend_from_within(start..start + prefix_len);
            assert_eq!(&buffer[..start], b"before ");
            assert_eq!(&buffer[start..], expected.repeat(2).as_bytes());
        }
    }
}
