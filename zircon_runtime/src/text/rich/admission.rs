use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::text::RichTextAuthoringDiagnosticCode;

/// Default request-local bound for rich source and visible output.
///
/// This follows the existing retained-text document byte scale while remaining a separate parser
/// policy: cache bypass never grants permission to exceed the parser's representation budget.
pub const DEFAULT_RICH_TEXT_PARSE_BYTE_BUDGET: usize = 32 * 1024 * 1024;
/// Default bound for the compiled text exposed to non-visual semantic consumers.
pub const DEFAULT_RICH_TEXT_SEMANTIC_TEXT_BYTES: usize = 32 * 1024 * 1024;
/// Default bound for recognized markup delimiters in one parse request.
pub const DEFAULT_RICH_TEXT_PARSE_TOKEN_BUDGET: usize = 65_536;
/// Default maximum encoded byte length for one recognized markup token.
pub const DEFAULT_RICH_TEXT_TOKEN_BYTES: usize = 64 * 1024;
/// Default maximum number of attributes materialized for one markup token.
pub const DEFAULT_RICH_TEXT_ATTRIBUTES_PER_TOKEN: usize = 64;
/// Default maximum key/value bytes materialized for one markup token.
pub const DEFAULT_RICH_TEXT_ATTRIBUTE_BYTES_PER_TOKEN: usize = 16 * 1024;
/// Default maximum dynamic metadata returned by one decorator invocation.
pub const DEFAULT_RICH_TEXT_DECORATOR_METADATA_BYTES_PER_CALL: usize = 64 * 1024;
/// Default maximum dynamic metadata retained across all materialized runs.
pub const DEFAULT_RICH_TEXT_RETAINED_RUN_METADATA_BYTES: usize = 32 * 1024 * 1024;
/// Default bound for simultaneously active inline style/link tags.
pub const DEFAULT_RICH_TEXT_ACTIVE_TAG_DEPTH: usize = 128;
/// Default bound for simultaneously active BBCode block/paragraph owners.
pub const DEFAULT_RICH_TEXT_BLOCK_DEPTH: usize = 32;
/// Default bound for simultaneously active rich tables.
pub const DEFAULT_RICH_TEXT_TABLE_DEPTH: usize = 8;
/// Default maximum number of materialized styled runs in one parse request.
pub const DEFAULT_RICH_TEXT_RUNS: usize = 131_072;
/// Default maximum number of materialized paragraph overrides in one parse request.
pub const DEFAULT_RICH_TEXT_PARAGRAPHS: usize = 16_384;
/// Default maximum number of materialized tables in one parse request.
pub const DEFAULT_RICH_TEXT_TABLES: usize = 4_096;
/// Default maximum number of materialized cells across all tables in one parse request.
pub const DEFAULT_RICH_TEXT_TABLE_CELLS: usize = 65_536;
/// Default maximum number of retained cell projection indices in one compiled artifact.
pub const DEFAULT_RICH_TEXT_PROJECTION_INDICES: usize = 262_144;
/// Default maximum retained non-fatal authoring diagnostics in one compiled rich artifact.
pub const DEFAULT_RICH_TEXT_AUTHORING_DIAGNOSTICS: usize = 256;
/// UAX#9 explicit formatting depth limit, applied independently from markup nesting.
pub const DEFAULT_RICH_TEXT_BIDI_CONTROL_DEPTH: usize = 125;

const MAX_RICH_TEXT_INDEXED_BYTES: usize = u32::MAX as usize;

/// Declares whether one rich-text source may use legacy bidi embeddings and overrides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RichTextContentTrust {
    /// External or otherwise untrusted content. Balanced isolates and directional marks remain valid.
    #[default]
    Untrusted,
    /// Author-controlled content that may use balanced legacy embeddings and overrides.
    TrustedAuthoring,
}

/// Request-local rich-text parser capacity limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RichParseBudget {
    pub max_source_bytes: usize,
    pub max_output_bytes: usize,
    pub max_semantic_text_bytes: usize,
    pub max_tokens: usize,
    pub max_token_bytes: usize,
    pub max_attributes_per_token: usize,
    pub max_attribute_bytes_per_token: usize,
    pub max_decorator_metadata_bytes_per_call: usize,
    pub max_retained_run_metadata_bytes: usize,
    pub max_active_tag_depth: usize,
    pub max_block_depth: usize,
    pub max_table_depth: usize,
    pub max_runs: usize,
    pub max_paragraphs: usize,
    pub max_tables: usize,
    pub max_table_cells: usize,
    pub max_projection_indices: usize,
    pub max_authoring_diagnostics: usize,
    pub max_bidi_control_depth: usize,
}

impl RichParseBudget {
    pub const fn new(max_source_bytes: usize, max_output_bytes: usize) -> Self {
        Self {
            max_source_bytes,
            max_output_bytes,
            max_semantic_text_bytes: max_output_bytes,
            max_tokens: DEFAULT_RICH_TEXT_PARSE_TOKEN_BUDGET,
            max_token_bytes: DEFAULT_RICH_TEXT_TOKEN_BYTES,
            max_attributes_per_token: DEFAULT_RICH_TEXT_ATTRIBUTES_PER_TOKEN,
            max_attribute_bytes_per_token: DEFAULT_RICH_TEXT_ATTRIBUTE_BYTES_PER_TOKEN,
            max_decorator_metadata_bytes_per_call:
                DEFAULT_RICH_TEXT_DECORATOR_METADATA_BYTES_PER_CALL,
            max_retained_run_metadata_bytes: DEFAULT_RICH_TEXT_RETAINED_RUN_METADATA_BYTES,
            max_active_tag_depth: DEFAULT_RICH_TEXT_ACTIVE_TAG_DEPTH,
            max_block_depth: DEFAULT_RICH_TEXT_BLOCK_DEPTH,
            max_table_depth: DEFAULT_RICH_TEXT_TABLE_DEPTH,
            max_runs: DEFAULT_RICH_TEXT_RUNS,
            max_paragraphs: DEFAULT_RICH_TEXT_PARAGRAPHS,
            max_tables: DEFAULT_RICH_TEXT_TABLES,
            max_table_cells: DEFAULT_RICH_TEXT_TABLE_CELLS,
            max_projection_indices: DEFAULT_RICH_TEXT_PROJECTION_INDICES,
            max_authoring_diagnostics: DEFAULT_RICH_TEXT_AUTHORING_DIAGNOSTICS,
            max_bidi_control_depth: DEFAULT_RICH_TEXT_BIDI_CONTROL_DEPTH,
        }
    }

    pub const fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub const fn with_max_semantic_text_bytes(mut self, max_semantic_text_bytes: usize) -> Self {
        self.max_semantic_text_bytes = max_semantic_text_bytes;
        self
    }

    pub const fn with_max_token_bytes(mut self, max_token_bytes: usize) -> Self {
        self.max_token_bytes = max_token_bytes;
        self
    }

    pub const fn with_attribute_limits(
        mut self,
        max_attributes_per_token: usize,
        max_attribute_bytes_per_token: usize,
    ) -> Self {
        self.max_attributes_per_token = max_attributes_per_token;
        self.max_attribute_bytes_per_token = max_attribute_bytes_per_token;
        self
    }

    pub const fn with_metadata_limits(
        mut self,
        max_decorator_metadata_bytes_per_call: usize,
        max_retained_run_metadata_bytes: usize,
    ) -> Self {
        self.max_decorator_metadata_bytes_per_call = max_decorator_metadata_bytes_per_call;
        self.max_retained_run_metadata_bytes = max_retained_run_metadata_bytes;
        self
    }

    pub const fn with_max_active_tag_depth(mut self, max_active_tag_depth: usize) -> Self {
        self.max_active_tag_depth = max_active_tag_depth;
        self
    }

    pub const fn with_representation_limits(
        mut self,
        max_runs: usize,
        max_paragraphs: usize,
        max_tables: usize,
        max_table_cells: usize,
        max_projection_indices: usize,
    ) -> Self {
        self.max_runs = max_runs;
        self.max_paragraphs = max_paragraphs;
        self.max_tables = max_tables;
        self.max_table_cells = max_table_cells;
        self.max_projection_indices = max_projection_indices;
        self
    }

    pub const fn with_nesting_limits(
        mut self,
        max_active_tag_depth: usize,
        max_block_depth: usize,
        max_table_depth: usize,
    ) -> Self {
        self.max_active_tag_depth = max_active_tag_depth;
        self.max_block_depth = max_block_depth;
        self.max_table_depth = max_table_depth;
        self
    }

    pub const fn with_max_authoring_diagnostics(
        mut self,
        max_authoring_diagnostics: usize,
    ) -> Self {
        self.max_authoring_diagnostics = max_authoring_diagnostics;
        self
    }

    pub const fn with_max_bidi_control_depth(mut self, max_bidi_control_depth: usize) -> Self {
        self.max_bidi_control_depth = max_bidi_control_depth;
        self
    }

    pub(crate) fn admitted_source_bytes(self) -> usize {
        self.max_source_bytes.min(MAX_RICH_TEXT_INDEXED_BYTES)
    }

    pub(crate) fn admitted_output_bytes(self) -> usize {
        self.max_output_bytes.min(MAX_RICH_TEXT_INDEXED_BYTES)
    }

    pub(crate) fn admitted_semantic_text_bytes(self) -> usize {
        self.max_semantic_text_bytes
            .min(MAX_RICH_TEXT_INDEXED_BYTES)
    }

    pub(crate) const fn tokenizer_budget(self) -> RichTokenizerBudget {
        RichTokenizerBudget {
            max_token_bytes: self.max_token_bytes,
            max_attributes: self.max_attributes_per_token,
            max_attribute_bytes: self.max_attribute_bytes_per_token,
        }
    }

    pub(crate) fn admit_source(self, actual_bytes: usize) -> Result<(), RichTextParseError> {
        let max_bytes = self.admitted_source_bytes();
        if actual_bytes > max_bytes {
            return Err(RichTextParseError::SourceByteBudgetExceeded {
                actual_bytes,
                max_bytes,
            });
        }
        Ok(())
    }
}

impl Default for RichParseBudget {
    fn default() -> Self {
        Self {
            max_source_bytes: DEFAULT_RICH_TEXT_PARSE_BYTE_BUDGET,
            max_output_bytes: DEFAULT_RICH_TEXT_PARSE_BYTE_BUDGET,
            max_semantic_text_bytes: DEFAULT_RICH_TEXT_SEMANTIC_TEXT_BYTES,
            max_tokens: DEFAULT_RICH_TEXT_PARSE_TOKEN_BUDGET,
            max_token_bytes: DEFAULT_RICH_TEXT_TOKEN_BYTES,
            max_attributes_per_token: DEFAULT_RICH_TEXT_ATTRIBUTES_PER_TOKEN,
            max_attribute_bytes_per_token: DEFAULT_RICH_TEXT_ATTRIBUTE_BYTES_PER_TOKEN,
            max_decorator_metadata_bytes_per_call:
                DEFAULT_RICH_TEXT_DECORATOR_METADATA_BYTES_PER_CALL,
            max_retained_run_metadata_bytes: DEFAULT_RICH_TEXT_RETAINED_RUN_METADATA_BYTES,
            max_active_tag_depth: DEFAULT_RICH_TEXT_ACTIVE_TAG_DEPTH,
            max_block_depth: DEFAULT_RICH_TEXT_BLOCK_DEPTH,
            max_table_depth: DEFAULT_RICH_TEXT_TABLE_DEPTH,
            max_runs: DEFAULT_RICH_TEXT_RUNS,
            max_paragraphs: DEFAULT_RICH_TEXT_PARAGRAPHS,
            max_tables: DEFAULT_RICH_TEXT_TABLES,
            max_table_cells: DEFAULT_RICH_TEXT_TABLE_CELLS,
            max_projection_indices: DEFAULT_RICH_TEXT_PROJECTION_INDICES,
            max_authoring_diagnostics: DEFAULT_RICH_TEXT_AUTHORING_DIAGNOSTICS,
            max_bidi_control_depth: DEFAULT_RICH_TEXT_BIDI_CONTROL_DEPTH,
        }
    }
}

/// Terminal rich-text admission or artifact-construction failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RichTextParseError {
    ParserIdentityExhausted,
    SourceByteBudgetExceeded {
        actual_bytes: usize,
        max_bytes: usize,
    },
    OutputByteBudgetExceeded {
        attempted_bytes: usize,
        max_bytes: usize,
    },
    SemanticTextByteBudgetExceeded {
        attempted_bytes: usize,
        max_bytes: usize,
    },
    TokenBudgetExceeded {
        attempted_tokens: usize,
        max_tokens: usize,
    },
    TokenByteBudgetExceeded {
        attempted_bytes: usize,
        max_bytes: usize,
    },
    AttributeCountBudgetExceeded {
        attempted_attributes: usize,
        max_attributes: usize,
    },
    AttributeByteBudgetExceeded {
        attempted_bytes: usize,
        max_bytes: usize,
    },
    DecoratorMetadataBudgetExceeded {
        tag: String,
        attempted_bytes: usize,
        max_bytes: usize,
    },
    RunMetadataBudgetExceeded {
        attempted_bytes: usize,
        max_bytes: usize,
    },
    DecoratorPanicked {
        tag: String,
    },
    ActiveTagDepthBudgetExceeded {
        attempted_depth: usize,
        max_depth: usize,
    },
    BlockDepthBudgetExceeded {
        attempted_depth: usize,
        max_depth: usize,
    },
    TableDepthBudgetExceeded {
        attempted_depth: usize,
        max_depth: usize,
    },
    RunCountBudgetExceeded {
        attempted_runs: usize,
        max_runs: usize,
    },
    ParagraphCountBudgetExceeded {
        attempted_paragraphs: usize,
        max_paragraphs: usize,
    },
    TableCountBudgetExceeded {
        attempted_tables: usize,
        max_tables: usize,
    },
    TableCellCountBudgetExceeded {
        attempted_cells: usize,
        max_cells: usize,
    },
    ProjectionIndexBudgetExceeded {
        attempted_indices: usize,
        max_indices: usize,
    },
    ArtifactIndexCapacityExceeded {
        index_kind: &'static str,
        actual: usize,
        max: usize,
    },
    ArtifactSourceRangeInvalid {
        range_kind: &'static str,
        start: u32,
        end: u32,
        source_bytes: usize,
    },
    BidiControlNotAllowed {
        code: RichTextAuthoringDiagnosticCode,
        source_range: (u32, u32),
    },
    UnbalancedBidiControl {
        code: RichTextAuthoringDiagnosticCode,
        source_range: (u32, u32),
    },
    BidiControlDepthExceeded {
        attempted_depth: usize,
        max_depth: usize,
        source_range: (u32, u32),
    },
}

impl Display for RichTextParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParserIdentityExhausted => {
                write!(formatter, "rich-text parser identity space is exhausted")
            }
            Self::SourceByteBudgetExceeded {
                actual_bytes,
                max_bytes,
            } => write!(
                formatter,
                "rich source uses {actual_bytes} bytes, exceeding the {max_bytes}-byte budget"
            ),
            Self::OutputByteBudgetExceeded {
                attempted_bytes,
                max_bytes,
            } => write!(
                formatter,
                "rich visible output would use {attempted_bytes} bytes, exceeding the {max_bytes}-byte budget"
            ),
            Self::SemanticTextByteBudgetExceeded {
                attempted_bytes,
                max_bytes,
            } => write!(
                formatter,
                "rich semantic text would use {attempted_bytes} bytes, exceeding the {max_bytes}-byte budget"
            ),
            Self::TokenBudgetExceeded {
                attempted_tokens,
                max_tokens,
            } => write!(
                formatter,
                "rich parse would consume {attempted_tokens} markup tokens, exceeding the configured maximum {max_tokens}"
            ),
            Self::TokenByteBudgetExceeded {
                attempted_bytes,
                max_bytes,
            } => write!(
                formatter,
                "rich markup token uses {attempted_bytes} bytes, exceeding the configured maximum {max_bytes}"
            ),
            Self::AttributeCountBudgetExceeded {
                attempted_attributes,
                max_attributes,
            } => write!(
                formatter,
                "rich token would materialize {attempted_attributes} attributes, exceeding the configured maximum {max_attributes}"
            ),
            Self::AttributeByteBudgetExceeded {
                attempted_bytes,
                max_bytes,
            } => write!(
                formatter,
                "rich token attributes would materialize {attempted_bytes} bytes, exceeding the configured maximum {max_bytes}"
            ),
            Self::DecoratorMetadataBudgetExceeded {
                tag,
                attempted_bytes,
                max_bytes,
            } => write!(
                formatter,
                "rich decorator `{tag}` returned {attempted_bytes} metadata bytes, exceeding the configured per-call maximum {max_bytes}"
            ),
            Self::RunMetadataBudgetExceeded {
                attempted_bytes,
                max_bytes,
            } => write!(
                formatter,
                "rich runs would retain {attempted_bytes} metadata bytes, exceeding the configured maximum {max_bytes}"
            ),
            Self::DecoratorPanicked { tag } => {
                write!(formatter, "rich decorator `{tag}` panicked")
            }
            Self::ActiveTagDepthBudgetExceeded {
                attempted_depth,
                max_depth,
            } => write!(
                formatter,
                "rich active tag depth {attempted_depth} exceeds the configured maximum {max_depth}"
            ),
            Self::BlockDepthBudgetExceeded {
                attempted_depth,
                max_depth,
            } => write!(
                formatter,
                "rich block depth {attempted_depth} exceeds the configured maximum {max_depth}"
            ),
            Self::TableDepthBudgetExceeded {
                attempted_depth,
                max_depth,
            } => write!(
                formatter,
                "rich table depth {attempted_depth} exceeds the configured maximum {max_depth}"
            ),
            Self::RunCountBudgetExceeded {
                attempted_runs,
                max_runs,
            } => write!(
                formatter,
                "rich parse would materialize {attempted_runs} runs, exceeding the configured maximum {max_runs}"
            ),
            Self::ParagraphCountBudgetExceeded {
                attempted_paragraphs,
                max_paragraphs,
            } => write!(
                formatter,
                "rich parse would materialize {attempted_paragraphs} paragraphs, exceeding the configured maximum {max_paragraphs}"
            ),
            Self::TableCountBudgetExceeded {
                attempted_tables,
                max_tables,
            } => write!(
                formatter,
                "rich parse would materialize {attempted_tables} tables, exceeding the configured maximum {max_tables}"
            ),
            Self::TableCellCountBudgetExceeded {
                attempted_cells,
                max_cells,
            } => write!(
                formatter,
                "rich parse would materialize {attempted_cells} table cells, exceeding the configured maximum {max_cells}"
            ),
            Self::ProjectionIndexBudgetExceeded {
                attempted_indices,
                max_indices,
            } => write!(
                formatter,
                "rich compiled projection would retain {attempted_indices} indices, exceeding the configured maximum {max_indices}"
            ),
            Self::ArtifactIndexCapacityExceeded {
                index_kind,
                actual,
                max,
            } => write!(
                formatter,
                "rich artifact {index_kind} index {actual} exceeds the representable maximum {max}"
            ),
            Self::ArtifactSourceRangeInvalid {
                range_kind,
                start,
                end,
                source_bytes,
            } => write!(
                formatter,
                "rich artifact {range_kind} range {start}..{end} is invalid for {source_bytes} source bytes"
            ),
            Self::BidiControlNotAllowed { code, source_range } => write!(
                formatter,
                "untrusted rich text cannot use {} at source range {}..{}",
                code.diagnostic_code(),
                source_range.0,
                source_range.1
            ),
            Self::UnbalancedBidiControl { code, source_range } => write!(
                formatter,
                "rich text has unbalanced {} at source range {}..{}",
                code.diagnostic_code(),
                source_range.0,
                source_range.1
            ),
            Self::BidiControlDepthExceeded {
                attempted_depth,
                max_depth,
                source_range,
            } => write!(
                formatter,
                "rich bidi-control depth {attempted_depth} exceeds the configured maximum {max_depth} at source range {}..{}",
                source_range.0, source_range.1
            ),
        }
    }
}

impl Error for RichTextParseError {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RichTokenizerBudget {
    max_token_bytes: usize,
    max_attributes: usize,
    max_attribute_bytes: usize,
}

impl RichTokenizerBudget {
    pub(crate) fn admit_token_bytes(
        self,
        attempted_bytes: usize,
    ) -> Result<(), RichTextParseError> {
        if attempted_bytes > self.max_token_bytes {
            return Err(RichTextParseError::TokenByteBudgetExceeded {
                attempted_bytes,
                max_bytes: self.max_token_bytes,
            });
        }
        Ok(())
    }

    pub(crate) fn admit_attribute(
        self,
        consumed_attributes: usize,
        consumed_bytes: usize,
        name_bytes: usize,
        value_bytes: usize,
    ) -> Result<usize, RichTextParseError> {
        let attempted_attributes = consumed_attributes.checked_add(1).unwrap_or(usize::MAX);
        if attempted_attributes > self.max_attributes {
            return Err(RichTextParseError::AttributeCountBudgetExceeded {
                attempted_attributes,
                max_attributes: self.max_attributes,
            });
        }
        let attempted_bytes = consumed_bytes
            .checked_add(name_bytes)
            .and_then(|bytes| bytes.checked_add(value_bytes))
            .unwrap_or(usize::MAX);
        if attempted_bytes > self.max_attribute_bytes {
            return Err(RichTextParseError::AttributeByteBudgetExceeded {
                attempted_bytes,
                max_bytes: self.max_attribute_bytes,
            });
        }
        Ok(attempted_bytes)
    }
}

pub(crate) fn checked_artifact_index(
    index_kind: &'static str,
    actual: usize,
) -> Result<u32, RichTextParseError> {
    u32::try_from(actual).map_err(|_| RichTextParseError::ArtifactIndexCapacityExceeded {
        index_kind,
        actual,
        max: u32::MAX as usize,
    })
}
