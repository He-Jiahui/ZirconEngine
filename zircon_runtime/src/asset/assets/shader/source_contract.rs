use std::fmt;

use super::ShaderEntryPointAsset;

const MATERIAL_SURFACE_ENTRY_POINT: &str = "zr_material_surface";
const MATERIAL_SURFACE_DECLARATION: &str = "fn zr_material_surface(";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderSurfaceSourceContract {
    MaterialFunction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderSurfaceSourceContractError {
    MissingMaterialFunction,
    DuplicateMaterialFunction,
    MaterialFunctionAbiMismatch,
    MaterialFunctionOwnsExecutableEntryPoints,
}

impl fmt::Display for ShaderSurfaceSourceContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMaterialFunction => write!(
                formatter,
                "surface shader requires exactly one `fn {MATERIAL_SURFACE_ENTRY_POINT}(` material function"
            ),
            Self::DuplicateMaterialFunction => write!(
                formatter,
                "surface shader declares `fn {MATERIAL_SURFACE_ENTRY_POINT}(` more than once"
            ),
            Self::MaterialFunctionAbiMismatch => write!(
                formatter,
                "surface material function must accept exactly one `ZrVertexOutput` (or `ZrSurfaceInput` alias) and return `ZrSurfaceOutput`"
            ),
            Self::MaterialFunctionOwnsExecutableEntryPoints => write!(
                formatter,
                "surface material-function shader must not also own executable entry points"
            ),
        }
    }
}

impl std::error::Error for ShaderSurfaceSourceContractError {}

pub(super) fn classify_surface_source_contract(
    source: &str,
    entry_points: &[ShaderEntryPointAsset],
) -> Result<ShaderSurfaceSourceContract, ShaderSurfaceSourceContractError> {
    let material_function = authored_declaration_matches(source, MATERIAL_SURFACE_DECLARATION);

    match material_function.count {
        1 if entry_points.is_empty()
            && material_function_abi_is_valid(
                source,
                material_function
                    .first_end
                    .expect("one declaration publishes its end offset"),
            ) =>
        {
            Ok(ShaderSurfaceSourceContract::MaterialFunction)
        }
        1 if entry_points.is_empty() => {
            Err(ShaderSurfaceSourceContractError::MaterialFunctionAbiMismatch)
        }
        1 => Err(ShaderSurfaceSourceContractError::MaterialFunctionOwnsExecutableEntryPoints),
        count if count > 1 => Err(ShaderSurfaceSourceContractError::DuplicateMaterialFunction),
        _ => Err(ShaderSurfaceSourceContractError::MissingMaterialFunction),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AuthoredDeclarationMatches {
    count: usize,
    first_end: Option<usize>,
}

fn authored_declaration_matches(source: &str, declaration: &str) -> AuthoredDeclarationMatches {
    let bytes = source.as_bytes();
    let declaration = declaration.as_bytes();
    let mut index = 0;
    let mut block_comment_depth = 0_u32;
    let mut matches = AuthoredDeclarationMatches::default();

    while index < bytes.len() {
        if block_comment_depth > 0 {
            if bytes.get(index..index + 2) == Some(b"/*") {
                block_comment_depth += 1;
                index += 2;
            } else if bytes.get(index..index + 2) == Some(b"*/") {
                block_comment_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if bytes.get(index..index + 2) == Some(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        let has_identifier_boundary =
            index == 0 || !bytes[index - 1].is_ascii_alphanumeric() && bytes[index - 1] != b'_';
        if has_identifier_boundary
            && bytes.get(index..index + declaration.len()) == Some(declaration)
        {
            matches.count += 1;
            index += declaration.len();
            matches.first_end.get_or_insert(index);
            continue;
        }
        index += 1;
    }

    matches
}

fn material_function_abi_is_valid(source: &str, signature_offset: usize) -> bool {
    let mut cursor = WgslTokenCursor::new(source, signature_offset);
    if cursor.take_identifier().is_none() || !cursor.take_byte(b':') {
        return false;
    }
    let Some(input_type) = cursor.take_identifier() else {
        return false;
    };
    if input_type != b"ZrVertexOutput" && input_type != b"ZrSurfaceInput" {
        return false;
    }
    cursor.take_byte(b')')
        && cursor.take_byte(b'-')
        && cursor.take_byte(b'>')
        && cursor.take_identifier() == Some(b"ZrSurfaceOutput".as_slice())
        && cursor.take_byte(b'{')
}

struct WgslTokenCursor<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> WgslTokenCursor<'a> {
    fn new(source: &'a str, index: usize) -> Self {
        Self {
            bytes: source.as_bytes(),
            index,
        }
    }

    fn take_identifier(&mut self) -> Option<&'a [u8]> {
        self.skip_trivia();
        let start = self.index;
        let first = *self.bytes.get(self.index)?;
        if !first.is_ascii_alphabetic() && first != b'_' {
            return None;
        }
        self.index += 1;
        while self
            .bytes
            .get(self.index)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            self.index += 1;
        }
        Some(&self.bytes[start..self.index])
    }

    fn take_byte(&mut self, expected: u8) -> bool {
        self.skip_trivia();
        if self.bytes.get(self.index) != Some(&expected) {
            return false;
        }
        self.index += 1;
        true
    }

    fn skip_trivia(&mut self) {
        loop {
            while self
                .bytes
                .get(self.index)
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                self.index += 1;
            }
            if self.bytes.get(self.index..self.index + 2) == Some(b"//") {
                while self
                    .bytes
                    .get(self.index)
                    .is_some_and(|byte| *byte != b'\n')
                {
                    self.index += 1;
                }
                continue;
            }
            if self.bytes.get(self.index..self.index + 2) != Some(b"/*") {
                break;
            }
            self.index += 2;
            let mut depth = 1_u32;
            while self.index < self.bytes.len() && depth > 0 {
                if self.bytes.get(self.index..self.index + 2) == Some(b"/*") {
                    depth += 1;
                    self.index += 2;
                } else if self.bytes.get(self.index..self.index + 2) == Some(b"*/") {
                    depth -= 1;
                    self.index += 2;
                } else {
                    self.index += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, stage: &str) -> ShaderEntryPointAsset {
        ShaderEntryPointAsset {
            name: name.to_string(),
            stage: stage.to_string(),
        }
    }

    #[test]
    fn material_function_contract_ignores_commented_anchors() {
        let source = r#"
// fn zr_material_surface() {}
/* fn zr_material_surface() {} */
fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(input.color);
}
"#;

        assert_eq!(
            classify_surface_source_contract(source, &[]),
            Ok(ShaderSurfaceSourceContract::MaterialFunction)
        );
    }

    #[test]
    fn duplicate_material_function_is_rejected() {
        assert_eq!(
            classify_surface_source_contract(
                "fn zr_material_surface() {}\nfn zr_material_surface() {}",
                &[],
            ),
            Err(ShaderSurfaceSourceContractError::DuplicateMaterialFunction)
        );
    }

    #[test]
    fn material_function_cannot_also_publish_executable_entry_points() {
        assert_eq!(
            classify_surface_source_contract(
                "fn zr_material_surface() {}\n@fragment fn fs_main() {}",
                &[entry("fs_main", "fragment")],
            ),
            Err(ShaderSurfaceSourceContractError::MaterialFunctionOwnsExecutableEntryPoints)
        );
    }

    #[test]
    fn material_function_requires_the_template_call_signature() {
        assert_eq!(
            classify_surface_source_contract("fn zr_material_surface() {}", &[]),
            Err(ShaderSurfaceSourceContractError::MaterialFunctionAbiMismatch)
        );
        assert_eq!(
            classify_surface_source_contract(
                "fn zr_material_surface(input: ZrVertexOutput) -> vec4<f32> { return vec4<f32>(); }",
                &[],
            ),
            Err(ShaderSurfaceSourceContractError::MaterialFunctionAbiMismatch)
        );
        assert_eq!(
            classify_surface_source_contract(
                "somefn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {}",
                &[],
            ),
            Err(ShaderSurfaceSourceContractError::MissingMaterialFunction)
        );
        assert_eq!(
            classify_surface_source_contract(
                "fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {}",
                &[],
            ),
            Ok(ShaderSurfaceSourceContract::MaterialFunction)
        );
    }

    #[test]
    fn executable_full_pass_is_not_a_surface_material_contract() {
        assert_eq!(
            classify_surface_source_contract(
                "@vertex fn vs_main() {}\n@fragment fn fs_main() {}",
                &[entry("vs_main", "vertex"), entry("fs_main", "fragment")],
            ),
            Err(ShaderSurfaceSourceContractError::MissingMaterialFunction)
        );
    }
}
