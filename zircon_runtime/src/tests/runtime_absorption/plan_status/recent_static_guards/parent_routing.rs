use super::document_sources::RecentStaticGuardSources;

impl RecentStaticGuardSources {
    pub(super) fn assert_parent_routing(&self) {
        for (label, source) in [
            ("Runtime 01", self.runtime_01_plan),
            ("Runtime 02", self.runtime_02_plan),
            ("Runtime 03", self.runtime_03_plan),
            ("Runtime 04", self.runtime_04_plan),
            ("Runtime 05", self.runtime_05_plan),
            ("Runtime 06", self.runtime_06_plan),
            ("Runtime 07", self.runtime_07_plan),
            ("Runtime 08", self.runtime_08_plan),
            ("Runtime 09", self.runtime_09_plan),
            ("Runtime 10", self.runtime_10_plan),
            ("Runtime 11", self.runtime_11_plan),
            ("Runtime 12", self.runtime_12_plan),
            ("Runtime 13", self.runtime_13_plan),
            ("Runtime 14", self.runtime_14_plan),
        ] {
            assert!(
                source.contains("## 状态与产出记录"),
                "{label} parent should keep the current status routing section"
            );
        }
        assert!(
            self.runtime_index.contains("| 范围 | 记录位置 |"),
            "runtime index should keep aggregate status routing"
        );
    }
}
