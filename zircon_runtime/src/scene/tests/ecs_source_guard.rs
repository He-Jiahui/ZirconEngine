//! Text-only ECS guards: ignore layout and bound checks to one real declaration.
//! These helpers do not execute or prove the production algorithm.

#[derive(Clone, Copy)]
pub(super) struct Source<'a>(&'a str);

impl<'a> Source<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self(source)
    }

    /// Guard source shape without depending on rustfmt's line breaks or indentation.
    pub(super) fn contains(self, needle: &str) -> bool {
        compact(self.0).contains(&compact(needle))
    }
}

impl std::ops::Deref for Source<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

/// Read one uniquely named braced item, not everything up to a guessed next method.
pub(super) fn source_item<'a>(source: Source<'a>, declaration: &str) -> Source<'a> {
    let source = source.0;
    let start = source
        .find(declaration)
        .unwrap_or_else(|| panic!("missing source declaration: {declaration}"));
    assert!(
        !source[start + declaration.len()..].contains(declaration),
        "ambiguous source declaration: {declaration}"
    );
    let bytes = source.as_bytes();
    let mut index = start + declaration.len();
    let mut depth = 0_usize;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            index += source[index..].find('\n').unwrap_or(bytes.len() - index);
        } else if bytes[index..].starts_with(b"/*") {
            index += 2;
            let mut comments = 1_usize;
            while comments != 0 && index < bytes.len() {
                if bytes[index..].starts_with(b"/*") {
                    comments += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    comments -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            assert_eq!(comments, 0, "unterminated comment in {declaration}");
        } else if bytes[index] == b'r' {
            let mut quote = index + 1;
            while bytes.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if bytes.get(quote) == Some(&b'"') {
                let ending = format!("\"{}", "#".repeat(quote - index - 1));
                index = quote + 1;
                index += source[index..]
                    .find(&ending)
                    .unwrap_or_else(|| panic!("unterminated raw string in {declaration}"))
                    + ending.len();
            } else {
                index += 1;
            }
        } else if bytes[index] == b'"' {
            index += 1;
            loop {
                match bytes.get(index) {
                    Some(b'\\') => index += 2,
                    Some(b'"') => {
                        index += 1;
                        break;
                    }
                    Some(_) => index += 1,
                    None => panic!("unterminated string in {declaration}"),
                }
            }
        } else if bytes[index] == b'\'' && bytes.get(index + 2) == Some(&b'\'') {
            index += 3;
        } else if bytes[index..].starts_with(b"'\\") && bytes.get(index + 3) == Some(&b'\'') {
            index += 4;
        } else {
            match bytes[index] {
                b'{' => depth += 1,
                b'}' => {
                    depth = depth.checked_sub(1).expect("source item closing brace");
                    if depth == 0 {
                        return Source::new(&source[start..=index]);
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }
    panic!("unterminated source item: {declaration}")
}
