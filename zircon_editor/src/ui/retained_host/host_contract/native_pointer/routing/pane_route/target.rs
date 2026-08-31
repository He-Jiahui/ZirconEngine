use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerRouteHit;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum PaneAssetSurface {
    Activity,
    Browser,
}

impl PaneAssetSurface {
    pub(in crate::ui::retained_host::host_contract) const fn as_str(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Browser => "browser",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum PaneAssetReferenceList {
    References,
    UsedBy,
}

impl PaneAssetReferenceList {
    pub(in crate::ui::retained_host::host_contract) const fn as_str(self) -> &'static str {
        match self {
            Self::References => "references",
            Self::UsedBy => "used_by",
        }
    }
}

pub(in crate::ui::retained_host::host_contract) enum PanePointerTarget<'a> {
    Hierarchy,
    Welcome,
    Console,
    Inspector,
    BrowserAssetDetails,
    AssetTree(PaneAssetSurface),
    AssetContent(PaneAssetSurface),
    AssetReference(PaneAssetSurface, PaneAssetReferenceList),
    TemplateNode(TemplateNodePointerRouteHit<'a>),
    ViewportToolbar {
        surface_key: &'a str,
        control_id: Option<&'a str>,
    },
    SceneViewport(&'a str),
    GameViewport(&'a str),
    UiAsset,
    Other,
}
