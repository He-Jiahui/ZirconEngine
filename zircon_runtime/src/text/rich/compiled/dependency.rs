use crate::core::resource::ResourceId;
use crate::text::{InlineObjectRef, RichIconAssetId, RichParseResult};

/// Loadable resource dependency retained by one compiled rich-text artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichTextDependency {
    ImageTexture(ResourceId),
    IconAsset(RichIconAssetId),
}

pub(super) fn collect(parsed: &RichParseResult) -> Vec<RichTextDependency> {
    let mut dependencies = parsed
        .runs
        .iter()
        .filter_map(|run| match run.inline.as_ref() {
            Some(InlineObjectRef::Image { texture, .. }) => {
                Some(RichTextDependency::ImageTexture(*texture))
            }
            Some(InlineObjectRef::Icon { asset, .. }) => {
                Some(RichTextDependency::IconAsset(*asset))
            }
            Some(InlineObjectRef::Widget { .. }) | None => None,
        })
        .collect::<Vec<_>>();
    dependencies.sort_unstable();
    dependencies.dedup();
    dependencies
}

#[cfg(test)]
mod tests {
    use crate::core::math::Vec2;
    use crate::core::resource::ResourceId;
    use crate::text::{
        InlineBaseline, InlineObjectRef, RichIconAssetId, RichParseResult, StyledRun,
    };

    use super::{RichTextDependency, collect};

    #[test]
    fn dependency_collection_deduplicates_typed_inline_assets() {
        let first = ResourceId::from_stable_label("res://ui/first.png");
        let second = ResourceId::from_stable_label("res://ui/second.png");
        let icon =
            RichIconAssetId::from_resource_id(ResourceId::from_stable_label("res://ui/icon.png"));
        let parsed = RichParseResult {
            runs: vec![
                image_run(second),
                icon_run(icon),
                image_run(first),
                icon_run(icon),
                image_run(second),
            ],
            ..RichParseResult::default()
        };
        let mut expected = vec![
            RichTextDependency::ImageTexture(first),
            RichTextDependency::ImageTexture(second),
            RichTextDependency::IconAsset(icon),
        ];
        expected.sort_unstable();

        assert_eq!(collect(&parsed), expected);
    }

    fn image_run(texture: ResourceId) -> StyledRun {
        StyledRun {
            inline: Some(InlineObjectRef::Image {
                texture,
                size: Vec2::new(16.0, 16.0),
                baseline: InlineBaseline::Baseline,
                alternative_text: None,
                tooltip: None,
            }),
            ..StyledRun::default()
        }
    }

    fn icon_run(asset: RichIconAssetId) -> StyledRun {
        StyledRun {
            inline: Some(InlineObjectRef::Icon {
                asset,
                size: Vec2::new(16.0, 16.0),
                baseline: InlineBaseline::Baseline,
                alternative_text: Some("Icon".to_owned()),
            }),
            ..StyledRun::default()
        }
    }
}
