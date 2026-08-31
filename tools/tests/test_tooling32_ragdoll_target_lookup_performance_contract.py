from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/physics/runtime/src/skeletal/profile.rs"


class RagdollTargetLookupPerformanceContract(unittest.TestCase):
    def test_spawn_builds_one_borrowed_target_lookup_before_the_bone_loop(self):
        production = SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        spawn = production.split("pub fn spawn(", 1)[1].split("pub fn spawn_configured(", 1)[0]

        lookup = spawn.index("SkeletalTargetLookup::new(rows)")
        loop = spawn.index("for bone in bones")
        self.assertLess(lookup, loop)
        self.assertIn("let local_bone = target_lookup", spawn)
        self.assertIn(".resolve(&bone.bone_path)", spawn)
        self.assertNotIn("resolve_unique_target(rows", spawn)

    def test_lookup_borrows_rows_and_tracks_ambiguous_names_once(self):
        production = SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]

        self.assertIn("struct SkeletalTargetLookup<'a>", production)
        self.assertIn("HashMap<&'a str, Option<&'a SkeletalPoseTarget>>", production)
        self.assertIn(".and_modify(|target| *target = None)", production)
        self.assertNotIn("fn resolve_unique_target", production)


if __name__ == "__main__":
    unittest.main()
