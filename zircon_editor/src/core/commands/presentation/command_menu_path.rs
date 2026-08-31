use serde::{Deserialize, Serialize};

use crate::core::editor_operation::EditorOperationPath;

use super::EditorCommandMenuSegment;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCommandMenuPath {
    root: EditorCommandMenuSegment,
    groups: Vec<EditorCommandMenuSegment>,
    leaf: EditorCommandMenuSegment,
}

impl EditorCommandMenuPath {
    pub fn new(
        root: EditorCommandMenuSegment,
        groups: impl IntoIterator<Item = EditorCommandMenuSegment>,
        leaf: EditorCommandMenuSegment,
    ) -> Self {
        Self {
            root,
            groups: groups.into_iter().collect(),
            leaf,
        }
    }

    pub fn builtin(command_id: &EditorOperationPath, root_id: &str, group_ids: &[&str]) -> Self {
        let root = EditorCommandMenuSegment::parse(root_id, format!("menu.{root_id}.label"))
            .expect("built-in command menu root is valid");
        let groups = group_ids.iter().map(|group_id| {
            EditorCommandMenuSegment::parse(*group_id, format!("menu.{root_id}.{group_id}.label"))
                .expect("built-in command menu group is valid")
        });
        let leaf = EditorCommandMenuSegment::parse(
            command_id.as_str(),
            format!("command.{}.label", command_id.as_str()),
        )
        .expect("built-in command menu leaf is valid");
        Self::new(root, groups, leaf)
    }

    pub fn root(&self) -> &EditorCommandMenuSegment {
        &self.root
    }

    pub fn groups(&self) -> &[EditorCommandMenuSegment] {
        &self.groups
    }

    pub fn leaf(&self) -> &EditorCommandMenuSegment {
        &self.leaf
    }

    pub fn segments(&self) -> impl ExactSizeIterator<Item = &EditorCommandMenuSegment> {
        std::iter::once(&self.root)
            .chain(self.groups.iter())
            .chain(std::iter::once(&self.leaf))
    }

    pub fn stable_path(&self) -> String {
        let mut segments = self.segments();
        let first = segments
            .next()
            .expect("command menu paths always contain a root and leaf");
        let mut path = first.id().as_str().to_owned();
        for segment in segments {
            path.push('/');
            path.push_str(segment.id().as_str());
        }
        path
    }
}
