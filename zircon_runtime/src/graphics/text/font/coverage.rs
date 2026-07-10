use ttf_parser::Face;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FontCoverage {
    Known(Vec<(u32, u32)>),
    Unknown,
}

impl FontCoverage {
    pub(super) fn from_sfnt_bytes(bytes: &[u8], face_index: u32) -> Self {
        let Ok(face) = Face::parse(bytes, face_index) else {
            return Self::Unknown;
        };
        let Some(cmap) = face.tables().cmap else {
            return Self::Unknown;
        };

        let mut codepoints = Vec::new();
        for subtable in cmap.subtables {
            if !subtable.is_unicode() {
                continue;
            }
            subtable.codepoints(|codepoint| {
                if char::from_u32(codepoint)
                    .and_then(|ch| face.glyph_index(ch))
                    .is_some()
                {
                    codepoints.push(codepoint);
                }
            });
        }
        codepoints.sort_unstable();
        codepoints.dedup();

        if codepoints.is_empty() {
            Self::Unknown
        } else {
            Self::Known(compact_codepoint_ranges(codepoints))
        }
    }

    pub(super) fn contains(&self, codepoint: char) -> bool {
        match self {
            Self::Known(ranges) => {
                let codepoint = codepoint as u32;
                ranges
                    .iter()
                    .any(|(start, end)| *start <= codepoint && codepoint <= *end)
            }
            Self::Unknown => true,
        }
    }

    #[cfg(test)]
    pub(super) fn from_codepoints(codepoints: &[char]) -> Self {
        let mut codepoints = codepoints
            .iter()
            .map(|codepoint| *codepoint as u32)
            .collect::<Vec<_>>();
        codepoints.sort_unstable();
        codepoints.dedup();
        Self::Known(compact_codepoint_ranges(codepoints))
    }
}

fn compact_codepoint_ranges(codepoints: Vec<u32>) -> Vec<(u32, u32)> {
    let mut ranges = Vec::new();
    let mut iter = codepoints.into_iter();
    let Some(mut start) = iter.next() else {
        return ranges;
    };
    let mut end = start;
    for codepoint in iter {
        if codepoint == end.saturating_add(1) {
            end = codepoint;
            continue;
        }
        ranges.push((start, end));
        start = codepoint;
        end = codepoint;
    }
    ranges.push((start, end));
    ranges
}
