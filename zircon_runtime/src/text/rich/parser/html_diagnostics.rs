use crate::text::{
    RichTextAuthoringDiagnostic, RichTextAuthoringDiagnosticCode,
    RichTextAuthoringDiagnosticSeverity, RichTextAuthoringRecovery,
};

use super::super::html_subset::{
    HtmlAttributeApplicationIssues, HtmlEntityIssues, HtmlTokenIssues,
};
use super::builder::RichParseBuilder;

pub(super) fn push_html_authoring_diagnostic(
    result: &mut RichParseBuilder,
    code: RichTextAuthoringDiagnosticCode,
    source_range: (u32, u32),
    recovery: RichTextAuthoringRecovery,
) {
    result.push_authoring_diagnostic(RichTextAuthoringDiagnostic {
        severity: RichTextAuthoringDiagnosticSeverity::Warning,
        code,
        source_range,
        recovery,
    });
}

pub(super) fn push_html_token_issues(
    result: &mut RichParseBuilder,
    issues: HtmlTokenIssues,
    source_range: (u32, u32),
) {
    if issues.malformed_tag {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::MalformedTag,
            source_range,
            RichTextAuthoringRecovery::PreservedAsText,
        );
    }
    if issues.unterminated_quoted_attribute {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::UnterminatedQuotedAttribute,
            source_range,
            RichTextAuthoringRecovery::PreservedAsText,
        );
    }
    if issues.unsupported_attribute {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::UnsupportedAttribute,
            source_range,
            RichTextAuthoringRecovery::IgnoredAttribute,
        );
    }
    if issues.malformed_attribute {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::MalformedAttribute,
            source_range,
            RichTextAuthoringRecovery::IgnoredAttribute,
        );
    }
}

pub(super) fn push_html_entity_issues(
    result: &mut RichParseBuilder,
    issues: HtmlEntityIssues,
    source_range: (u32, u32),
) {
    if issues.unrecognized_entity {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::UnrecognizedEntity,
            source_range,
            RichTextAuthoringRecovery::PreservedAsText,
        );
    }
    if issues.malformed_entity {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::MalformedEntity,
            source_range,
            RichTextAuthoringRecovery::PreservedAsText,
        );
    }
}

pub(super) fn push_html_attribute_application_issues(
    result: &mut RichParseBuilder,
    issues: HtmlAttributeApplicationIssues,
    source_range: (u32, u32),
) {
    if issues.unsupported_style_property {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::UnsupportedStyleProperty,
            source_range,
            RichTextAuthoringRecovery::IgnoredStyleDeclaration,
        );
    }
    if issues.invalid_attribute_value {
        push_html_authoring_diagnostic(
            result,
            RichTextAuthoringDiagnosticCode::InvalidAttributeValue,
            source_range,
            RichTextAuthoringRecovery::IgnoredAttribute,
        );
    }
}
