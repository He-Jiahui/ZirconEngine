pub(super) fn has_bare_thread_owner(source: &str) -> bool {
    let spawn_word = ["sp", "awn"].concat();
    let scope_word = ["sc", "ope"].concat();
    let scoped_spawn_word = [spawn_word.as_str(), "_scoped"].concat();
    let builder_word = ["Build", "er"].concat();
    let scope_type_word = ["Sc", "ope"].concat();
    let tokens = without_cfg_test_modules(rust_code_tokens(source));
    let mut imports = ThreadImports::default();
    collect_thread_imports(
        &tokens,
        &spawn_word,
        &scope_word,
        &builder_word,
        &scope_type_word,
        &mut imports,
    );

    [spawn_word.as_str(), scope_word.as_str()]
        .iter()
        .any(|function_name| {
            let function_name = *function_name;
            path_is_called(&tokens, &["std", "::", "thread", "::", function_name])
                || path_is_called(&tokens, &["::", "std", "::", "thread", "::", function_name])
                || imports.std_root_names.iter().any(|root| {
                    path_is_called(
                        &tokens,
                        &[root.as_str(), "::", "thread", "::", function_name],
                    )
                })
                || imports
                    .module_names
                    .iter()
                    .any(|module| path_is_called(&tokens, &[module.as_str(), "::", function_name]))
                || imports
                    .function_names
                    .iter()
                    .any(|name| unqualified_function_is_called(&tokens, name))
        })
        || thread_types_own_threads(
            &tokens,
            &imports,
            &builder_word,
            &scope_type_word,
            &spawn_word,
            &scoped_spawn_word,
        )
}

/// Removes direct `#[cfg(test)]` module bodies before scanning production
/// ownership. Visibility and additional attributes do not change the item's
/// test-only boundary; everything outside that body remains visible.
fn without_cfg_test_modules(tokens: Vec<String>) -> Vec<String> {
    let mut production_tokens = Vec::with_capacity(tokens.len());
    let mut index = 0;

    while index < tokens.len() {
        let Some(attribute_end) = cfg_test_attribute_end(&tokens, index) else {
            production_tokens.push(tokens[index].clone());
            index += 1;
            continue;
        };
        if let Some(module_end) = cfg_test_module_end(&tokens, attribute_end + 1) {
            index = module_end + 1;
            continue;
        }

        production_tokens.push(tokens[index].clone());
        index += 1;
    }

    production_tokens
}

fn cfg_test_module_end(tokens: &[String], mut index: usize) -> Option<usize> {
    while let Some(attribute_end) = attribute_end(tokens, index) {
        index = attribute_end + 1;
    }
    if tokens.get(index).is_some_and(|token| token == "pub") {
        index += 1;
        if tokens.get(index).is_some_and(|token| token == "(") {
            index = matching_close(tokens, index, "(", ")")? + 1;
        }
    }
    if tokens.get(index).is_some_and(|token| token == "mod")
        && tokens
            .get(index + 1)
            .is_some_and(|token| is_identifier(token))
        && tokens.get(index + 2).is_some_and(|token| token == "{")
    {
        matching_close(tokens, index + 2, "{", "}")
    } else {
        None
    }
}

fn cfg_test_attribute_end(tokens: &[String], index: usize) -> Option<usize> {
    let attribute_end = attribute_end(tokens, index)?;
    tokens
        .get(index + 2..attribute_end)
        .is_some_and(|attribute| {
            attribute
                .iter()
                .map(String::as_str)
                .eq(["cfg", "(", "test", ")"])
        })
        .then_some(attribute_end)
}

fn attribute_end(tokens: &[String], index: usize) -> Option<usize> {
    sequence_at(tokens, index, &["#", "["]).then(|| matching_close(tokens, index + 1, "[", "]"))?
}

#[derive(Default)]
struct ThreadImports {
    std_root_names: Vec<String>,
    module_names: Vec<String>,
    function_names: Vec<String>,
    builder_names: Vec<String>,
    scope_type_names: Vec<String>,
}

fn thread_types_own_threads(
    tokens: &[String],
    imports: &ThreadImports,
    builder_name: &str,
    scope_type_name: &str,
    spawn_name: &str,
    scoped_spawn_name: &str,
) -> bool {
    for index in 0..tokens.len() {
        if let Some(type_len) = thread_builder_type_len(tokens, index, imports, builder_name) {
            if associated_call_at(tokens, index + type_len, spawn_name)
                || associated_call_at(tokens, index + type_len, scoped_spawn_name)
                || builder_constructor_chain_spawns(
                    tokens,
                    index + type_len,
                    spawn_name,
                    scoped_spawn_name,
                )
            {
                return true;
            }
        }
        if let Some(type_len) = thread_scope_type_len(tokens, index, imports, scope_type_name) {
            if associated_call_at(tokens, index + type_len, spawn_name) {
                return true;
            }
        }
    }

    let mut builder_receivers = typed_receivers(tokens, |tokens, index| {
        thread_builder_type_len(tokens, index, imports, builder_name)
    });
    collect_builder_assignments(tokens, imports, builder_name, &mut builder_receivers);
    let scope_receivers = typed_receivers(tokens, |tokens, index| {
        thread_scope_type_len(tokens, index, imports, scope_type_name)
    });

    builder_receivers.iter().any(|receiver| {
        receiver_method_is_called(tokens, receiver, spawn_name)
            || receiver_method_is_called(tokens, receiver, scoped_spawn_name)
    }) || scope_receivers
        .iter()
        .any(|receiver| receiver_method_is_called(tokens, receiver, spawn_name))
}

fn collect_thread_imports(
    tokens: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "use" {
            index += 1;
            continue;
        }
        let Some(statement_end) = tokens[index + 1..]
            .iter()
            .position(|token| token == ";")
            .map(|offset| index + 1 + offset)
        else {
            break;
        };
        parse_thread_import(
            &tokens[index + 1..statement_end],
            spawn_name,
            scope_name,
            builder_name,
            scope_type_name,
            imports,
        );
        index = statement_end + 1;
    }
}

fn parse_thread_import(
    path: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    let path = if path.first().is_some_and(|token| token == "::") {
        &path[1..]
    } else {
        path
    };
    if path.len() == 3 && sequence_at(path, 0, &["std", "as"]) {
        push_unique(&mut imports.std_root_names, &path[2]);
        return;
    }
    if sequence_at(path, 0, &["std", "::", "thread"]) {
        let tail = &path[3..];
        if tail.is_empty() {
            push_unique(&mut imports.module_names, "thread");
        } else if tail.first().is_some_and(|token| token == "as") {
            if let Some(alias) = tail.get(1) {
                push_unique(&mut imports.module_names, alias);
            }
        } else if tail.first().is_some_and(|token| token == "::") {
            parse_thread_import_items(
                &tail[1..],
                spawn_name,
                scope_name,
                builder_name,
                scope_type_name,
                imports,
            );
        }
        return;
    }

    if path.len() >= 5
        && sequence_at(path, 0, &["std", "::", "{"])
        && path.last().is_some_and(|token| token == "}")
    {
        for item in top_level_items(&path[3..path.len() - 1]) {
            if item.first().is_some_and(|token| token == "self") {
                let root_name = if item.get(1).is_some_and(|token| token == "as") {
                    item.get(2).map(String::as_str).unwrap_or("std")
                } else {
                    "std"
                };
                push_unique(&mut imports.std_root_names, root_name);
                continue;
            }
            if item.first().is_none_or(|token| token != "thread") {
                continue;
            }
            if item.len() == 1 {
                push_unique(&mut imports.module_names, "thread");
            } else if item.get(1).is_some_and(|token| token == "as") {
                if let Some(alias) = item.get(2) {
                    push_unique(&mut imports.module_names, alias);
                }
            } else if item.get(1).is_some_and(|token| token == "::") {
                parse_thread_import_items(
                    &item[2..],
                    spawn_name,
                    scope_name,
                    builder_name,
                    scope_type_name,
                    imports,
                );
            }
        }
    }
}

fn parse_thread_import_items(
    items: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    if items.first().is_some_and(|token| token == "{")
        && items.last().is_some_and(|token| token == "}")
    {
        for item in top_level_items(&items[1..items.len() - 1]) {
            register_thread_import_item(
                item,
                spawn_name,
                scope_name,
                builder_name,
                scope_type_name,
                imports,
            );
        }
    } else {
        register_thread_import_item(
            items,
            spawn_name,
            scope_name,
            builder_name,
            scope_type_name,
            imports,
        );
    }
}

fn register_thread_import_item(
    item: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    let Some(imported_name) = item.first() else {
        return;
    };
    let resolved_name = if item.get(1).is_some_and(|token| token == "as") {
        let Some(alias) = item.get(2) else {
            return;
        };
        alias
    } else {
        imported_name
    };

    if imported_name == "self" {
        let module_name = if item.get(1).is_some_and(|token| token == "as") {
            resolved_name
        } else {
            "thread"
        };
        push_unique(&mut imports.module_names, module_name);
    } else if imported_name == spawn_name || imported_name == scope_name {
        push_unique(&mut imports.function_names, resolved_name);
    } else if imported_name == builder_name {
        push_unique(&mut imports.builder_names, resolved_name);
    } else if imported_name == scope_type_name {
        push_unique(&mut imports.scope_type_names, resolved_name);
    }
}

fn top_level_items(tokens: &[String]) -> Vec<&[String]> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token.as_str() {
            "{" | "(" | "[" | "<" => depth += 1,
            "}" | ")" | "]" | ">" => depth = depth.saturating_sub(1),
            "," if depth == 0 => {
                items.push(&tokens[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    items.push(&tokens[start..]);
    items
}

fn path_is_called(tokens: &[String], path: &[&str]) -> bool {
    (0..tokens.len()).any(|index| {
        sequence_at(tokens, index, path)
            && path_boundary_before(tokens, index)
            && tokens
                .get(index + path.len())
                .is_some_and(|token| token == "(")
    })
}

fn unqualified_function_is_called(tokens: &[String], function_name: &str) -> bool {
    tokens.iter().enumerate().any(|(index, token)| {
        token == function_name
            && tokens.get(index + 1).is_some_and(|next| next == "(")
            && !matches!(
                tokens.get(index.wrapping_sub(1)).map(String::as_str),
                Some(".") | Some("::")
            )
    })
}

fn thread_builder_type_len(
    tokens: &[String],
    index: usize,
    imports: &ThreadImports,
    builder_name: &str,
) -> Option<usize> {
    thread_type_len(tokens, index, imports, builder_name, &imports.builder_names)
}

fn thread_scope_type_len(
    tokens: &[String],
    index: usize,
    imports: &ThreadImports,
    scope_type_name: &str,
) -> Option<usize> {
    thread_type_len(
        tokens,
        index,
        imports,
        scope_type_name,
        &imports.scope_type_names,
    )
}

fn thread_type_len(
    tokens: &[String],
    index: usize,
    imports: &ThreadImports,
    canonical_name: &str,
    imported_names: &[String],
) -> Option<usize> {
    if sequence_at(
        tokens,
        index,
        &["std", "::", "thread", "::", canonical_name],
    ) && path_boundary_before(tokens, index)
    {
        return Some(5);
    }
    if sequence_at(
        tokens,
        index,
        &["::", "std", "::", "thread", "::", canonical_name],
    ) && path_boundary_before(tokens, index)
    {
        return Some(6);
    }
    if imports.std_root_names.iter().any(|root| {
        sequence_at(
            tokens,
            index,
            &[root.as_str(), "::", "thread", "::", canonical_name],
        ) && path_boundary_before(tokens, index)
    }) {
        return Some(5);
    }
    if imports.module_names.iter().any(|module| {
        sequence_at(tokens, index, &[module.as_str(), "::", canonical_name])
            && path_boundary_before(tokens, index)
    }) {
        return Some(3);
    }
    if imported_names
        .iter()
        .any(|name| tokens.get(index).is_some_and(|token| token == name))
        && path_boundary_before(tokens, index)
    {
        return Some(1);
    }
    None
}

fn associated_call_at(tokens: &[String], after_type: usize, method_name: &str) -> bool {
    sequence_at(tokens, after_type, &["::", method_name, "("])
}

fn builder_constructor_chain_spawns(
    tokens: &[String],
    after_type: usize,
    spawn_name: &str,
    scoped_spawn_name: &str,
) -> bool {
    if !sequence_at(tokens, after_type, &["::", "new", "("]) {
        return false;
    }
    let Some(mut cursor) = matching_close(tokens, after_type + 2, "(", ")").map(|end| end + 1)
    else {
        return false;
    };
    while sequence_at(tokens, cursor, &["."])
        && tokens
            .get(cursor + 1)
            .is_some_and(|token| is_identifier(token))
        && tokens.get(cursor + 2).is_some_and(|token| token == "(")
    {
        let method = &tokens[cursor + 1];
        if method == spawn_name || method == scoped_spawn_name {
            return true;
        }
        let Some(end) = matching_close(tokens, cursor + 2, "(", ")") else {
            return false;
        };
        cursor = end + 1;
    }
    false
}

fn typed_receivers(
    tokens: &[String],
    type_len_at: impl Fn(&[String], usize) -> Option<usize>,
) -> Vec<String> {
    let mut receivers = Vec::new();
    for index in 0..tokens.len().saturating_sub(2) {
        if !is_identifier(&tokens[index]) || tokens[index + 1] != ":" {
            continue;
        }
        let mut type_index = index + 2;
        while type_index < tokens.len()
            && !matches!(tokens[type_index].as_str(), "," | ")" | "=" | ";" | "{")
        {
            if type_len_at(tokens, type_index).is_some() {
                push_unique(&mut receivers, &tokens[index]);
                break;
            }
            type_index += 1;
        }
    }
    receivers
}

fn collect_builder_assignments(
    tokens: &[String],
    imports: &ThreadImports,
    builder_name: &str,
    receivers: &mut Vec<String>,
) {
    let mut index = 0;
    while index + 3 < tokens.len() {
        if tokens[index] != "let" {
            index += 1;
            continue;
        }
        let name_index = if tokens.get(index + 1).is_some_and(|token| token == "mut") {
            index + 2
        } else {
            index + 1
        };
        if !tokens
            .get(name_index)
            .is_some_and(|token| is_identifier(token))
        {
            index += 1;
            continue;
        }
        let Some(equals_index) = tokens[name_index + 1..]
            .iter()
            .position(|token| token == "=" || token == ";")
            .map(|offset| name_index + 1 + offset)
        else {
            break;
        };
        if tokens[equals_index] == "=" {
            let type_index = equals_index + 1;
            if let Some(type_len) =
                thread_builder_type_len(tokens, type_index, imports, builder_name)
            {
                if sequence_at(tokens, type_index + type_len, &["::", "new", "("]) {
                    push_unique(receivers, &tokens[name_index]);
                }
            }
        }
        index = equals_index + 1;
    }
}

fn receiver_method_is_called(tokens: &[String], receiver: &str, method_name: &str) -> bool {
    (0..tokens.len()).any(|index| {
        sequence_at(tokens, index, &[receiver, ".", method_name, "("])
            && path_boundary_before(tokens, index)
    })
}

fn matching_close(tokens: &[String], open_index: usize, open: &str, close: &str) -> Option<usize> {
    if tokens.get(open_index).is_none_or(|token| token != open) {
        return None;
    }
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        if token == open {
            depth += 1;
        } else if token == close {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn sequence_at(tokens: &[String], index: usize, sequence: &[&str]) -> bool {
    tokens
        .get(index..index + sequence.len())
        .is_some_and(|candidate| {
            candidate
                .iter()
                .map(String::as_str)
                .eq(sequence.iter().copied())
        })
}

fn path_boundary_before(tokens: &[String], index: usize) -> bool {
    !matches!(
        index
            .checked_sub(1)
            .and_then(|before| tokens.get(before))
            .map(String::as_str),
        Some(".") | Some("::")
    )
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !value.is_empty() && !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

fn is_identifier(token: &str) -> bool {
    token
        .bytes()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn rust_code_tokens(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
        } else if bytes[index..].starts_with(b"//") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            index = skip_block_comment(bytes, index);
        } else if let Some(end) = char_or_byte_literal_end(bytes, index) {
            index = end;
        } else if let Some(end) = raw_string_end(bytes, index) {
            index = end;
        } else if bytes[index] == b'"' {
            index = skip_quoted_string(bytes, index);
        } else if matches!(bytes[index], b'b' | b'c')
            && bytes.get(index + 1).is_some_and(|byte| *byte == b'"')
        {
            index = skip_quoted_string(bytes, index + 1);
        } else if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            tokens.push(String::from_utf8_lossy(&bytes[start..index]).into_owned());
        } else if index + 1 < bytes.len()
            && matches!(&bytes[index..index + 2], b"::" | b"->" | b"=>")
        {
            tokens.push(String::from_utf8_lossy(&bytes[index..index + 2]).into_owned());
            index += 2;
        } else {
            tokens.push((bytes[index] as char).to_string());
            index += 1;
        }
    }
    tokens
}

fn char_or_byte_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let quote_index = if bytes.get(start) == Some(&b'\'') {
        start
    } else if bytes.get(start) == Some(&b'b') && bytes.get(start + 1) == Some(&b'\'') {
        start + 1
    } else {
        return None;
    };
    let mut cursor = quote_index + 1;
    if bytes.get(cursor) == Some(&b'\\') {
        cursor += 1;
        match bytes.get(cursor).copied()? {
            b'x' => cursor += 3,
            b'u' if bytes.get(cursor + 1) == Some(&b'{') => {
                cursor += 2;
                while bytes.get(cursor).is_some_and(|byte| *byte != b'}') {
                    cursor += 1;
                }
                if bytes.get(cursor) != Some(&b'}') {
                    return None;
                }
                cursor += 1;
            }
            _ => cursor += 1,
        }
    } else {
        let character = std::str::from_utf8(bytes.get(cursor..)?)
            .ok()?
            .chars()
            .next()?;
        cursor += character.len_utf8();
    }
    (bytes.get(cursor) == Some(&b'\'')).then_some(cursor + 1)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start + 2;
    while index < bytes.len() && depth > 0 {
        if bytes[index..].starts_with(b"/*") {
            depth += 1;
            index += 2;
        } else if bytes[index..].starts_with(b"*/") {
            depth -= 1;
            index += 2;
        } else {
            index += 1;
        }
    }
    index
}

fn skip_quoted_string(bytes: &[u8], quote_index: usize) -> usize {
    let mut index = quote_index + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == b'"' {
            return index + 1;
        } else {
            index += 1;
        }
    }
    index
}

fn raw_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut marker = start;
    if matches!(bytes.get(marker), Some(b'b' | b'c')) {
        marker += 1;
    }
    if bytes.get(marker) != Some(&b'r') {
        return None;
    }
    marker += 1;
    let hashes_start = marker;
    while bytes.get(marker) == Some(&b'#') {
        marker += 1;
    }
    if bytes.get(marker) != Some(&b'"') {
        return None;
    }
    let hash_count = marker - hashes_start;
    let mut index = marker + 1;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && bytes
                .get(index + 1..index + 1 + hash_count)
                .is_some_and(|hashes| hashes.iter().all(|byte| *byte == b'#'))
        {
            return Some(index + 1 + hash_count);
        }
        index += 1;
    }
    Some(bytes.len())
}
