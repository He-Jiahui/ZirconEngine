use super::HostInvalidationMask;

impl HostInvalidationMask {
    pub(crate) fn summary(self) -> String {
        if self.is_empty() {
            return "none".to_string();
        }

        let mut names = Vec::new();
        if self.contains(Self::LAYOUT) {
            names.push("layout");
        }
        if self.contains(Self::TREE_STRUCTURE) {
            names.push("tree-structure");
        }
        if self.contains(Self::PRESENTATION_DATA) {
            names.push("presentation-data");
        }
        if self.contains(Self::PAINT_ONLY) {
            names.push("paint-only");
        }
        if self.contains(Self::POINTER_HOVER) {
            names.push("pointer-hover");
        }
        if self.contains(Self::VIEWPORT_IMAGE) {
            names.push("viewport-image");
        }
        if self.contains(Self::HIT_TEST) {
            names.push("hit-test");
        }
        if self.contains(Self::WINDOW_METRICS) {
            names.push("window-metrics");
        }
        if self.contains(Self::RENDER) {
            names.push("render");
        }
        if self.contains(Self::SHELL_CONTENT) {
            names.push("shell-content");
        }
        if self.contains(Self::WORKBENCH_PROJECTION) {
            names.push("workbench-projection");
        }
        names.join("|")
    }
}
