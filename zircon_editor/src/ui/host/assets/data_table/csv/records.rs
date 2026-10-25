use super::DataTableCsvError;

/// RFC-style comma/quote records, with CRLF or LF separators. Quoted line endings are data.
/// The optional initial UTF-8 BOM is transport syntax; no cell/name whitespace is trimmed.
pub(super) fn read(input: &str) -> Result<Vec<Vec<String>>, DataTableCsvError> {
    let bytes = input.as_bytes();
    let mut at = if input.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let mut rows = Vec::new();
    if at == bytes.len() {
        return Ok(rows);
    }
    'records: loop {
        let mut row = Vec::new();
        loop {
            let cell = if bytes.get(at) == Some(&b'"') {
                at += 1;
                let mut chunk = at;
                let mut value = String::new();
                loop {
                    match bytes.get(at) {
                        Some(b'"') => {
                            // ASCII delimiters are always UTF-8 boundaries.
                            value.push_str(&input[chunk..at]);
                            at += 1;
                            if bytes.get(at) == Some(&b'"') {
                                value.push('"');
                                at += 1;
                                chunk = at;
                            } else {
                                break;
                            }
                        }
                        Some(_) => at += 1,
                        None => {
                            return Err(DataTableCsvError::Syntax {
                                byte: at,
                                reason: "unterminated quoted cell",
                            });
                        }
                    }
                }
                value
            } else {
                let start = at;
                while let Some(byte) = bytes.get(at) {
                    match byte {
                        b',' | b'\r' | b'\n' => break,
                        b'"' => {
                            return Err(DataTableCsvError::Syntax {
                                byte: at,
                                reason: "quote in an unquoted cell",
                            });
                        }
                        _ => at += 1,
                    }
                }
                input[start..at].to_owned()
            };
            row.push(cell);
            match bytes.get(at) {
                None => {
                    rows.push(row);
                    break 'records;
                }
                Some(b',') => at += 1,
                Some(b'\n') => {
                    at += 1;
                    rows.push(row);
                    if at == bytes.len() {
                        break 'records;
                    }
                    continue 'records;
                }
                Some(b'\r') if bytes.get(at + 1) == Some(&b'\n') => {
                    at += 2;
                    rows.push(row);
                    if at == bytes.len() {
                        break 'records;
                    }
                    continue 'records;
                }
                _ => {
                    return Err(DataTableCsvError::Syntax {
                        byte: at,
                        reason: "expected comma, LF, CRLF or end of input after cell",
                    });
                }
            }
        }
    }
    Ok(rows)
}

pub(super) fn write(output: &mut String, cells: &[String]) {
    for (index, cell) in cells.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        if cell.contains([',', '"', '\r', '\n']) {
            output.push('"');
            for ch in cell.chars() {
                if ch == '"' {
                    output.push('"');
                }
                output.push(ch);
            }
            output.push('"');
        } else {
            output.push_str(cell);
        }
    }
    output.push_str("\r\n");
}
