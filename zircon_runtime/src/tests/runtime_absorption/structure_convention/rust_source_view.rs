pub(super) fn production_section(source: &str) -> String {
    let (code, comment_free_source) = rust_lexical_views(source);
    let spans = cfg_test_item_spans(&code, &comment_free_source);
    mask_cfg_test_spans(comment_free_source.into_bytes(), &spans)
}

pub(super) fn production_code_view(source: &str) -> String {
    let (code, cfg_source) = rust_lexical_views(source);
    let spans = cfg_test_item_spans(&code, &cfg_source);
    mask_cfg_test_spans(code.into_bytes(), &spans)
}

fn mask_cfg_test_spans(mut production: Vec<u8>, spans: &[(usize, usize)]) -> String {
    for &(start, end) in spans {
        blank_bytes_preserving_lines(&mut production, start, end);
    }
    String::from_utf8(production).expect("masked Rust source should remain valid UTF-8")
}

fn rust_lexical_views(source: &str) -> (String, String) {
    let source_bytes = source.as_bytes();
    let mut rendered = source_bytes.to_vec();
    let mut cfg_source = source_bytes.to_vec();
    let mut index = 0;
    let mut block_comment_depth = 0usize;

    while index < source_bytes.len() {
        if block_comment_depth > 0 {
            if source_bytes[index..].starts_with(b"/*") {
                blank_bytes_preserving_lines(&mut rendered, index, index + 2);
                blank_bytes_preserving_lines(&mut cfg_source, index, index + 2);
                block_comment_depth += 1;
                index += 2;
            } else if source_bytes[index..].starts_with(b"*/") {
                blank_bytes_preserving_lines(&mut rendered, index, index + 2);
                blank_bytes_preserving_lines(&mut cfg_source, index, index + 2);
                block_comment_depth -= 1;
                index += 2;
            } else {
                blank_bytes_preserving_lines(&mut rendered, index, index + 1);
                blank_bytes_preserving_lines(&mut cfg_source, index, index + 1);
                index += 1;
            }
            continue;
        }

        if source_bytes[index..].starts_with(b"//") {
            let end = source_bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(source_bytes.len(), |offset| index + offset);
            blank_bytes_preserving_lines(&mut rendered, index, end);
            blank_bytes_preserving_lines(&mut cfg_source, index, end);
            index = end;
            continue;
        }
        if source_bytes[index..].starts_with(b"/*") {
            blank_bytes_preserving_lines(&mut rendered, index, index + 2);
            blank_bytes_preserving_lines(&mut cfg_source, index, index + 2);
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        if let Some(end) = raw_string_end(source_bytes, index) {
            blank_bytes_preserving_lines(&mut rendered, index, end);
            index = end;
            continue;
        }
        if let Some(end) = quoted_literal_end(source_bytes, index) {
            blank_bytes_preserving_lines(&mut rendered, index, end);
            index = end;
            continue;
        }
        index += 1;
    }

    (
        String::from_utf8(rendered).expect("masked Rust source should remain valid UTF-8"),
        String::from_utf8(cfg_source).expect("cfg Rust source should remain valid UTF-8"),
    )
}

fn raw_string_end(source: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start;
    if source
        .get(cursor)
        .copied()
        .is_some_and(|byte| matches!(byte, b'b' | b'c'))
    {
        cursor += 1;
    }
    if source.get(cursor) != Some(&b'r') {
        return None;
    }
    cursor += 1;
    let hash_start = cursor;
    while source.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    if source.get(cursor) != Some(&b'"') {
        return None;
    }
    let hash_count = cursor - hash_start;
    cursor += 1;
    while cursor < source.len() {
        if source[cursor] == b'"'
            && source
                .get(cursor + 1..cursor + 1 + hash_count)
                .is_some_and(|hashes| hashes.iter().all(|byte| *byte == b'#'))
        {
            return Some(cursor + 1 + hash_count);
        }
        cursor += 1;
    }
    Some(source.len())
}

fn quoted_literal_end(source: &[u8], start: usize) -> Option<usize> {
    let quote_index = match (source.get(start), source.get(start + 1)) {
        (Some(b'"' | b'\''), _) => start,
        (Some(b'b' | b'c'), Some(b'"')) | (Some(b'b'), Some(b'\'')) => start + 1,
        _ => return None,
    };
    let quote = source[quote_index];
    if quote == b'\'' {
        return character_literal_end(source, quote_index);
    }

    let mut cursor = quote_index + 1;
    let mut escaped = false;
    while cursor < source.len() {
        let byte = source[cursor];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    Some(source.len())
}

fn character_literal_end(source: &[u8], quote_index: usize) -> Option<usize> {
    let mut cursor = quote_index + 1;
    if source.get(cursor) == Some(&b'\\') {
        cursor += 1;
        cursor = match source.get(cursor).copied()? {
            b'x' => cursor.checked_add(3)?,
            b'u' if source.get(cursor + 1) == Some(&b'{') => source
                .get(cursor + 2..)?
                .iter()
                .position(|byte| *byte == b'}')
                .map(|offset| cursor + 3 + offset)?,
            _ => cursor + 1,
        };
    } else {
        let width = utf8_scalar_width(*source.get(cursor)?);
        if width == 0 || source.get(cursor..cursor + width).is_none() {
            return None;
        }
        cursor += width;
    }
    (source.get(cursor) == Some(&b'\'')).then_some(cursor + 1)
}

fn utf8_scalar_width(first: u8) -> usize {
    match first {
        0x00..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => 0,
    }
}

#[derive(Clone, Debug)]
enum CfgExpression {
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Box<Self>),
    Test,
    Atom(String),
}

fn cfg_test_item_spans(code: &str, cfg_source: &str) -> Vec<(usize, usize)> {
    let bytes = code.as_bytes();
    let mut spans = Vec::new();
    let mut cursor = 0;
    while cursor + 1 < bytes.len() {
        if !bytes[cursor..].starts_with(b"#[") {
            cursor += 1;
            continue;
        }
        let group_start = cursor;
        let mut attribute_cursor = cursor;
        let mut constraints = Vec::new();
        loop {
            let attribute_end = matching_delimiter_end(bytes, attribute_cursor + 1, b'[', b']');
            if attribute_end <= attribute_cursor + 2 {
                break;
            }
            constraints.extend(cfg_attribute_constraints(
                &cfg_source[attribute_cursor + 2..attribute_end - 1],
            ));
            attribute_cursor = skip_ascii_whitespace(bytes, attribute_end);
            if !bytes
                .get(attribute_cursor..)
                .is_some_and(|tail| tail.starts_with(b"#["))
            {
                break;
            }
        }
        let item_start = attribute_cursor;
        let Some(item_end) = rust_item_end_if_supported(bytes, item_start) else {
            cursor = item_start.max(group_start + 2);
            continue;
        };
        if cfg_constraints_require_test(&constraints) {
            spans.push((group_start, item_end));
            cursor = item_end;
        } else {
            cursor = item_start.max(group_start + 2);
        }
    }
    spans
}

fn cfg_attribute_constraints(attribute: &str) -> Vec<CfgExpression> {
    let normalized = normalize_cfg_meta(attribute);
    if let Some(predicate) = normalized
        .strip_prefix("cfg(")
        .and_then(|inner| inner.strip_suffix(')'))
        .and_then(parse_cfg_expression)
    {
        return vec![predicate];
    }
    let Some(arguments) = normalized
        .strip_prefix("cfg_attr(")
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        return Vec::new();
    };
    let parts = split_cfg_arguments(arguments);
    let Some(condition) = parts.first().and_then(|part| parse_cfg_expression(part)) else {
        return Vec::new();
    };
    parts[1..]
        .iter()
        .flat_map(|nested| cfg_attribute_constraints(nested))
        .map(|constraint| {
            CfgExpression::Any(vec![
                CfgExpression::Not(Box::new(condition.clone())),
                constraint,
            ])
        })
        .collect()
}

fn normalize_cfg_meta(source: &str) -> String {
    let mut normalized = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in source.chars() {
        if in_string {
            normalized.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
            normalized.push(character);
        } else if !character.is_whitespace() {
            normalized.push(character);
        }
    }
    normalized
}

fn parse_cfg_expression(source: &str) -> Option<CfgExpression> {
    if source == "test" {
        return Some(CfgExpression::Test);
    }
    for operator in ["all", "any", "not"] {
        let Some(arguments) = source
            .strip_prefix(operator)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
        else {
            continue;
        };
        let parsed = split_cfg_arguments(arguments)
            .into_iter()
            .map(parse_cfg_expression)
            .collect::<Option<Vec<_>>>()?;
        return Some(match operator {
            "all" => CfgExpression::All(parsed),
            "any" => CfgExpression::Any(parsed),
            "not" if parsed.len() == 1 => CfgExpression::Not(Box::new(parsed.into_iter().next()?)),
            _ => return None,
        });
    }
    (!source.is_empty()).then(|| CfgExpression::Atom(source.to_string()))
}

fn cfg_constraints_require_test(constraints: &[CfgExpression]) -> bool {
    if constraints.is_empty() {
        return false;
    }
    let combined = CfgExpression::All(constraints.to_vec());
    let mut atoms = std::collections::BTreeSet::new();
    collect_cfg_atoms(&combined, &mut atoms);
    if atoms.len() > 16 {
        return false;
    }
    let atoms: Vec<_> = atoms.into_iter().collect();
    for bit_mask in 0..(1usize << atoms.len()) {
        if evaluate_cfg_with_test_false(&combined, &atoms, bit_mask) {
            return false;
        }
    }
    true
}

fn collect_cfg_atoms(expression: &CfgExpression, atoms: &mut std::collections::BTreeSet<String>) {
    match expression {
        CfgExpression::All(arguments) | CfgExpression::Any(arguments) => {
            for argument in arguments {
                collect_cfg_atoms(argument, atoms);
            }
        }
        CfgExpression::Not(argument) => collect_cfg_atoms(argument, atoms),
        CfgExpression::Atom(atom) => {
            atoms.insert(atom.clone());
        }
        CfgExpression::Test => {}
    }
}

fn evaluate_cfg_with_test_false(
    expression: &CfgExpression,
    atoms: &[String],
    bit_mask: usize,
) -> bool {
    match expression {
        CfgExpression::All(arguments) => arguments
            .iter()
            .all(|argument| evaluate_cfg_with_test_false(argument, atoms, bit_mask)),
        CfgExpression::Any(arguments) => arguments
            .iter()
            .any(|argument| evaluate_cfg_with_test_false(argument, atoms, bit_mask)),
        CfgExpression::Not(argument) => !evaluate_cfg_with_test_false(argument, atoms, bit_mask),
        CfgExpression::Test => false,
        CfgExpression::Atom(atom) => atoms
            .iter()
            .position(|candidate| candidate == atom)
            .is_some_and(|index| bit_mask & (1 << index) != 0),
    }
}

fn split_cfg_arguments(arguments: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in arguments.bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                parts.push(&arguments[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if start < arguments.len() {
        parts.push(&arguments[start..]);
    }
    parts
}

fn skip_ascii_whitespace(code: &[u8], mut cursor: usize) -> usize {
    while code.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    cursor
}

fn rust_item_end_if_supported(code: &[u8], start: usize) -> Option<usize> {
    let signature_end = code[start..]
        .iter()
        .position(|byte| matches!(*byte, b'{' | b';' | b',' | b'}'))
        .map_or(code.len(), |offset| start + offset);
    let signature = std::str::from_utf8(&code[start..signature_end]).ok()?;
    let normalized = signature.split_whitespace().collect::<Vec<_>>().join(" ");
    let item = strip_rust_visibility(&normalized);
    let body_item = [
        "fn ",
        "async fn ",
        "unsafe fn ",
        "const fn ",
        "mod ",
        "struct ",
        "enum ",
        "union ",
        "trait ",
        "impl ",
        "unsafe impl ",
        "extern ",
        "macro_rules!",
        "macro ",
    ]
    .iter()
    .any(|prefix| item.starts_with(prefix));
    let semicolon_item = ["type ", "const ", "static ", "use ", "extern crate "]
        .iter()
        .any(|prefix| item.starts_with(prefix));
    if !body_item && !semicolon_item {
        // Keep unsupported local grammar visible. A conservative false positive is
        // preferable to masking the rest of a production struct, enum, or function.
        return None;
    }

    let mut cursor = start;
    let mut parentheses = 0usize;
    let mut brackets = 0usize;
    let mut braces = 0usize;
    while cursor < code.len() {
        match code[cursor] {
            b'(' => parentheses += 1,
            b')' => parentheses = parentheses.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'{' if parentheses == 0 && brackets == 0 => {
                if body_item && braces == 0 {
                    return Some(matching_delimiter_end(code, cursor, b'{', b'}'));
                }
                braces += 1;
            }
            b'}' => braces = braces.saturating_sub(1),
            b';' if parentheses == 0 && brackets == 0 && braces == 0 => return Some(cursor + 1),
            _ => {}
        }
        cursor += 1;
    }
    None
}

fn strip_rust_visibility(signature: &str) -> &str {
    let trimmed = signature.trim_start();
    if let Some(rest) = trimmed.strip_prefix("pub ") {
        return rest.trim_start();
    }
    let Some(rest) = trimmed.strip_prefix("pub(") else {
        return trimmed;
    };
    rest.find(')')
        .map(|closing| rest[closing + 1..].trim_start())
        .unwrap_or(trimmed)
}

fn matching_delimiter_end(code: &[u8], opening: usize, open: u8, close: u8) -> usize {
    let mut depth = 0usize;
    for (offset, byte) in code[opening..].iter().enumerate() {
        if *byte == open {
            depth += 1;
        } else if *byte == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return opening + offset + 1;
            }
        }
    }
    code.len()
}

fn blank_bytes_preserving_lines(bytes: &mut [u8], start: usize, end: usize) {
    let bounded_end = end.min(bytes.len());
    for byte in &mut bytes[start..bounded_end] {
        if !matches!(*byte, b'\r' | b'\n') {
            *byte = b' ';
        }
    }
}

#[test]
fn runtime_15_rust_production_view_rejects_lexical_and_test_only_false_positives() {
    let source = r###"
fn before() { narrow_owner_query(); }
// text_state.font_database();
const NOTE: &str = "text_state.font_database()";
#[cfg(test)]
fn test_only_database_access() { text_state.font_database(); }
fn after() { text_state.font_database(); }
"###;

    let production = production_code_view(source);
    assert_eq!(production.lines().count(), source.lines().count());
    assert!(!production.contains("fn test_only_database_access"));
    assert_eq!(production.matches("text_state.font_database()").count(), 1);
    assert!(production.contains("fn after()"));
}

#[test]
fn runtime_15_structure_guards_share_the_rust_production_view_owner() {
    let root = super::runtime_src_path("tests/runtime_absorption/structure_convention");
    let mut pending = vec![root];
    let mut violations = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", directory.display()))
        {
            let path = entry.expect("structure guard directory entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", path.display()));
            let (code, cfg_source) = rust_lexical_views(&source);
            let mut search_start = 0;
            while let Some(offset) = code[search_start..].find(".split") {
                let split_start = search_start + offset;
                let opening = skip_ascii_whitespace(code.as_bytes(), split_start + ".split".len());
                if code.as_bytes().get(opening) != Some(&b'(') {
                    search_start = split_start + ".split".len();
                    continue;
                }
                let split_end = matching_delimiter_end(code.as_bytes(), opening, b'(', b')');
                if normalize_cfg_meta(&cfg_source[split_start..split_end]).contains("cfg(test)") {
                    violations.push(path.clone());
                    break;
                }
                search_start = split_start + ".split".len();
            }
        }
    }
    violations.sort();
    assert!(
        violations.is_empty(),
        "Runtime15 structure guards must consume rust_source_view instead of truncating the first cfg(test): {violations:#?}"
    );
}
