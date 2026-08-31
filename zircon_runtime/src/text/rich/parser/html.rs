use crate::text::{
    LinkRef, RichParseResult, RichTextAuthoringDiagnosticCode, RichTextAuthoringRecovery,
    StyleOverride,
};

use super::super::html_subset::{self, HtmlToken};
use super::super::{RichParseBudget, RichTextContentTrust, RichTextParseError};
use super::active_tags::{ActiveTag, ActiveTagClose, ActiveTagStack, current_link, current_style};
use super::bidi_diagnostics::{
    push_literal_bidi_control_diagnostic, push_source_bidi_control_diagnostics,
};
use super::builder::RichParseBuilder;
use super::html_diagnostics::{
    push_html_attribute_application_issues, push_html_authoring_diagnostic,
    push_html_entity_issues, push_html_token_issues,
};
use super::run_alignment::align_runs_to_graphemes_bounded;
use super::{
    ClosingDelimiterFrontier, append_inline_object, append_source_text_with_metadata,
    append_text_with_metadata, markup_source_range, next_char_len, repeated_opening_delimiter_skip,
};

pub(super) fn parse_html(
    markup: &str,
    budget: RichParseBudget,
    content_trust: RichTextContentTrust,
) -> Result<RichParseResult, RichTextParseError> {
    let mut result = RichParseBuilder::new(budget, content_trust);
    let mut active_tags = ActiveTagStack::new(budget.max_active_tag_depth);
    let close_frontier = ClosingDelimiterFrontier::new(markup, ">");
    let mut index = 0;
    let mut text_start = 0;

    while index < markup.len() {
        let remaining = &markup[index..];
        let repeated_opener_skip = repeated_opening_delimiter_skip(remaining, b'<');
        if repeated_opener_skip > 0 {
            index += repeated_opener_skip;
            continue;
        }
        if remaining.starts_with('<') && !close_frontier.has_close_at_or_after(index + 1) {
            if html_subset::looks_like_tag_candidate(remaining) {
                append_html_text(
                    &mut result,
                    &markup[text_start..index],
                    text_start,
                    current_style(&active_tags),
                    current_link(&active_tags),
                )?;
                let code = if html_subset::has_unterminated_attribute_quote(remaining) {
                    RichTextAuthoringDiagnosticCode::UnterminatedQuotedAttribute
                } else {
                    RichTextAuthoringDiagnosticCode::MalformedTag
                };
                push_html_authoring_diagnostic(
                    &mut result,
                    code,
                    markup_source_range(index, remaining.len())?,
                    RichTextAuthoringRecovery::PreservedAsText,
                );
                append_source_text_with_metadata(
                    &mut result,
                    remaining,
                    index,
                    current_style(&active_tags),
                    None,
                    current_link(&active_tags),
                )?;
                text_start = markup.len();
            }
            break;
        }
        let Some((token_len, token)) = html_subset::token_at(remaining, budget.tokenizer_budget())?
        else {
            index += next_char_len(remaining);
            continue;
        };
        result.admit_tokens(1)?;
        let token_source_range = markup_source_range(index, token_len)?;
        append_html_text(
            &mut result,
            &markup[text_start..index],
            text_start,
            current_style(&active_tags),
            current_link(&active_tags),
        )?;
        push_html_token_issues(&mut result, token.issues(), token_source_range);
        match token {
            HtmlToken::Open { name, .. } if name == "br" => {
                append_text_with_metadata(
                    &mut result,
                    "\n",
                    current_style(&active_tags),
                    None,
                    current_link(&active_tags),
                );
            }
            HtmlToken::Open {
                name, attributes, ..
            } if name == "img" => {
                let mut issues = html_subset::HtmlAttributeApplicationIssues::default();
                let inline = html_subset::inline_image(&attributes, &mut issues);
                push_html_attribute_application_issues(&mut result, issues, token_source_range);
                if let Some(inline) = inline {
                    append_inline_object(
                        &mut result,
                        current_style(&active_tags),
                        current_link(&active_tags),
                        inline,
                    );
                }
            }
            HtmlToken::Open {
                name,
                attributes,
                self_closing,
                ..
            } if name == "a" => {
                let mut style = current_style(&active_tags);
                let mut issues = html_subset::HtmlAttributeApplicationIssues::default();
                let link = html_subset::link(&attributes, &mut style, &mut issues);
                push_html_attribute_application_issues(&mut result, issues, token_source_range);
                if let Some(link) = link {
                    if !self_closing {
                        active_tags.push(ActiveTag {
                            name,
                            style,
                            link: Some(link),
                            source_range: token_source_range,
                        })?;
                    }
                }
            }
            HtmlToken::Open {
                name,
                attributes,
                self_closing,
                ..
            } if html_subset::is_style_tag(&name) => {
                let mut style = current_style(&active_tags);
                let mut issues = html_subset::HtmlAttributeApplicationIssues::default();
                let applied =
                    html_subset::apply_style_tag(&name, &attributes, &mut style, &mut issues);
                push_html_attribute_application_issues(&mut result, issues, token_source_range);
                if applied && !self_closing {
                    let link = current_link(&active_tags);
                    active_tags.push(ActiveTag {
                        name,
                        style,
                        link,
                        source_range: token_source_range,
                    })?;
                }
            }
            HtmlToken::Close { name } if html_subset::is_style_tag(&name) || name == "a" => {
                match active_tags.close(&name) {
                    ActiveTagClose::NotFound => push_html_authoring_diagnostic(
                        &mut result,
                        RichTextAuthoringDiagnosticCode::UnmatchedClosingTag,
                        token_source_range,
                        RichTextAuthoringRecovery::DroppedMarkup,
                    ),
                    ActiveTagClose::Closed { implicitly_closed } if implicitly_closed > 0 => {
                        push_html_authoring_diagnostic(
                            &mut result,
                            RichTextAuthoringDiagnosticCode::ImplicitlyClosedTag,
                            token_source_range,
                            RichTextAuthoringRecovery::ImplicitlyClosed,
                        )
                    }
                    ActiveTagClose::Closed { .. } => {}
                }
            }
            HtmlToken::Malformed { .. } => append_source_text_with_metadata(
                &mut result,
                &markup[index..index + token_len],
                index,
                current_style(&active_tags),
                None,
                current_link(&active_tags),
            )?,
            HtmlToken::Open { .. } | HtmlToken::Close { .. } | HtmlToken::Ignored => {
                push_html_authoring_diagnostic(
                    &mut result,
                    RichTextAuthoringDiagnosticCode::UnsupportedTag,
                    token_source_range,
                    RichTextAuthoringRecovery::DroppedMarkup,
                );
            }
        }
        index += token_len;
        text_start = index;
    }
    append_html_text(
        &mut result,
        &markup[text_start..],
        text_start,
        current_style(&active_tags),
        current_link(&active_tags),
    )?;
    for source_range in active_tags.source_ranges() {
        push_html_authoring_diagnostic(
            &mut result,
            RichTextAuthoringDiagnosticCode::UnclosedTag,
            source_range,
            RichTextAuthoringRecovery::ClosedAtEndOfInput,
        );
    }
    result.runs = align_runs_to_graphemes_bounded(&result.text, &result.runs, budget.max_runs)?;
    result.finish()
}

fn append_html_text(
    result: &mut RichParseBuilder,
    text: &str,
    source_start: usize,
    style: StyleOverride,
    link: Option<LinkRef>,
) -> Result<(), RichTextParseError> {
    let mut observer_error = None;
    let (decoded, issues) = html_subset::decode_entities_with_issues_observing(
        text,
        |(fragment_start, fragment_end), fragment, decoded_entity| {
            if observer_error.is_some() {
                return;
            }
            let absolute_start = source_start
                .checked_add(fragment_start)
                .unwrap_or(usize::MAX);
            if decoded_entity {
                match markup_source_range(absolute_start, fragment_end - fragment_start) {
                    Ok(source_range) => {
                        match push_literal_bidi_control_diagnostic(result, fragment, source_range) {
                            Ok(()) => {}
                            Err(error) => observer_error = Some(error),
                        }
                    }
                    Err(error) => observer_error = Some(error),
                }
            } else if let Err(error) =
                push_source_bidi_control_diagnostics(result, fragment, absolute_start)
            {
                observer_error = Some(error);
            }
        },
    );
    if let Some(error) = observer_error {
        return Err(error);
    }
    append_text_with_metadata(result, decoded.as_ref(), style, None, link);
    if issues != html_subset::HtmlEntityIssues::default() {
        push_html_entity_issues(
            result,
            issues,
            markup_source_range(source_start, text.len())?,
        );
    }
    Ok(())
}
