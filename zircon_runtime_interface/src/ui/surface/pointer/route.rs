use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::ui::dispatch::UiInputModifiers;
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiPoint;
use crate::ui::surface::UiHitPath;

use super::{UiPointerActivationPhase, UiPointerButton, UiPointerEventKind};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPointerRoutingPath {
    #[default]
    HitPath,
    ExplicitRootToLeaf(Vec<UiNodeId>),
}

impl UiPointerRoutingPath {
    pub fn from_root_to_leaf(root_to_leaf: Vec<UiNodeId>) -> Self {
        Self::ExplicitRootToLeaf(root_to_leaf)
    }

    pub fn from_bubble_route(mut bubble_route: Vec<UiNodeId>) -> Self {
        bubble_route.reverse();
        Self::from_root_to_leaf(bubble_route)
    }

    fn from_bubble_route_or_hit_path(
        hit_path: &UiHitPath,
        mut bubble_route: Vec<UiNodeId>,
    ) -> Self {
        if bubble_route.iter().copied().eq(hit_path.bubble_route()) {
            Self::HitPath
        } else {
            bubble_route.reverse();
            Self::ExplicitRootToLeaf(bubble_route)
        }
    }

    pub fn root_to_leaf<'path>(&'path self, hit_path: &'path UiHitPath) -> &'path [UiNodeId] {
        match self {
            Self::HitPath => &hit_path.root_to_leaf,
            Self::ExplicitRootToLeaf(root_to_leaf) => root_to_leaf,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiPointerRoute {
    pub kind: UiPointerEventKind,
    pub button: Option<UiPointerButton>,
    pub modifiers: UiInputModifiers,
    pub activation_phase: UiPointerActivationPhase,
    pub point: UiPoint,
    pub scroll_delta: f32,
    pub target: Option<UiNodeId>,
    pub hit_path: UiHitPath,
    pub routing_path: UiPointerRoutingPath,
    pub stacked: Vec<UiNodeId>,
    pub entered: Vec<UiNodeId>,
    pub left: Vec<UiNodeId>,
    pub captured: Option<UiNodeId>,
    pub pressed: Option<UiNodeId>,
    pub click_target: Option<UiNodeId>,
    pub release_inside_pressed: bool,
    pub focused: Option<UiNodeId>,
    pub fallback_to_root: bool,
    pub root_targets: Vec<UiNodeId>,
}

impl UiPointerRoute {
    pub fn routing_root_to_leaf(&self) -> &[UiNodeId] {
        self.routing_path.root_to_leaf(&self.hit_path)
    }

    pub fn bubble_route(
        &self,
    ) -> impl DoubleEndedIterator<Item = UiNodeId> + ExactSizeIterator + '_ {
        self.routing_root_to_leaf().iter().rev().copied()
    }

    pub fn hit_candidates(&self) -> impl Iterator<Item = UiNodeId> + '_ {
        if self.stacked.is_empty() {
            UiPointerHitCandidates::Bubble(self.routing_root_to_leaf().iter().rev().copied())
        } else {
            UiPointerHitCandidates::Stacked(self.stacked.iter().copied())
        }
    }

    pub fn into_bubble_route(mut self) -> Vec<UiNodeId> {
        let mut root_to_leaf = match self.routing_path {
            UiPointerRoutingPath::HitPath => std::mem::take(&mut self.hit_path.root_to_leaf),
            UiPointerRoutingPath::ExplicitRootToLeaf(root_to_leaf) => root_to_leaf,
        };
        root_to_leaf.reverse();
        root_to_leaf
    }
}

impl PartialEq for UiPointerRoute {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.button == other.button
            && self.modifiers == other.modifiers
            && self.activation_phase == other.activation_phase
            && self.point == other.point
            && self.scroll_delta == other.scroll_delta
            && self.target == other.target
            && self.hit_path == other.hit_path
            && self.routing_root_to_leaf() == other.routing_root_to_leaf()
            && self.stacked == other.stacked
            && self.entered == other.entered
            && self.left == other.left
            && self.captured == other.captured
            && self.pressed == other.pressed
            && self.click_target == other.click_target
            && self.release_inside_pressed == other.release_inside_pressed
            && self.focused == other.focused
            && self.fallback_to_root == other.fallback_to_root
            && self.root_targets == other.root_targets
    }
}

impl Serialize for UiPointerRoute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("UiPointerRoute", 19)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("button", &self.button)?;
        state.serialize_field("modifiers", &self.modifiers)?;
        state.serialize_field("activation_phase", &self.activation_phase)?;
        state.serialize_field("point", &self.point)?;
        state.serialize_field("scroll_delta", &self.scroll_delta)?;
        state.serialize_field("target", &self.target)?;
        state.serialize_field("hit_path", &self.hit_path)?;
        state.serialize_field("bubbled", &ReverseNodePath(self.routing_root_to_leaf()))?;
        state.serialize_field("stacked", &self.stacked)?;
        state.serialize_field("entered", &self.entered)?;
        state.serialize_field("left", &self.left)?;
        state.serialize_field("captured", &self.captured)?;
        state.serialize_field("pressed", &self.pressed)?;
        state.serialize_field("click_target", &self.click_target)?;
        state.serialize_field("release_inside_pressed", &self.release_inside_pressed)?;
        state.serialize_field("focused", &self.focused)?;
        state.serialize_field("fallback_to_root", &self.fallback_to_root)?;
        state.serialize_field("root_targets", &self.root_targets)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for UiPointerRoute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WirePointerRoute {
            kind: UiPointerEventKind,
            button: Option<UiPointerButton>,
            #[serde(default)]
            modifiers: UiInputModifiers,
            #[serde(default)]
            activation_phase: UiPointerActivationPhase,
            point: UiPoint,
            scroll_delta: f32,
            target: Option<UiNodeId>,
            #[serde(default)]
            hit_path: UiHitPath,
            bubbled: Vec<UiNodeId>,
            stacked: Vec<UiNodeId>,
            entered: Vec<UiNodeId>,
            left: Vec<UiNodeId>,
            captured: Option<UiNodeId>,
            #[serde(default)]
            pressed: Option<UiNodeId>,
            #[serde(default)]
            click_target: Option<UiNodeId>,
            #[serde(default)]
            release_inside_pressed: bool,
            focused: Option<UiNodeId>,
            fallback_to_root: bool,
            root_targets: Vec<UiNodeId>,
        }

        let wire = WirePointerRoute::deserialize(deserializer)?;
        let routing_path =
            UiPointerRoutingPath::from_bubble_route_or_hit_path(&wire.hit_path, wire.bubbled);
        Ok(Self {
            kind: wire.kind,
            button: wire.button,
            modifiers: wire.modifiers,
            activation_phase: wire.activation_phase,
            point: wire.point,
            scroll_delta: wire.scroll_delta,
            target: wire.target,
            hit_path: wire.hit_path,
            routing_path,
            stacked: wire.stacked,
            entered: wire.entered,
            left: wire.left,
            captured: wire.captured,
            pressed: wire.pressed,
            click_target: wire.click_target,
            release_inside_pressed: wire.release_inside_pressed,
            focused: wire.focused,
            fallback_to_root: wire.fallback_to_root,
            root_targets: wire.root_targets,
        })
    }
}

struct ReverseNodePath<'path>(&'path [UiNodeId]);

enum UiPointerHitCandidates<'route> {
    Stacked(std::iter::Copied<std::slice::Iter<'route, UiNodeId>>),
    Bubble(std::iter::Copied<std::iter::Rev<std::slice::Iter<'route, UiNodeId>>>),
}

impl Iterator for UiPointerHitCandidates<'_> {
    type Item = UiNodeId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Stacked(iter) => iter.next(),
            Self::Bubble(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Stacked(iter) => iter.size_hint(),
            Self::Bubble(iter) => iter.size_hint(),
        }
    }
}

impl Serialize for ReverseNodePath<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for node_id in self.0.iter().rev() {
            sequence.serialize_element(node_id)?;
        }
        sequence.end()
    }
}
