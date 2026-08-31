from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor01CustomMessageSchemaNamespaceContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_schema_identity_is_typed_bounded_and_validated_during_deserialize(self) -> None:
        schema = self.read(
            "zircon_editor/src/core/editor_message/message/schema_id.rs"
        )

        self.assertIn("pub struct EditorMessageSchemaId(Arc<str>);", schema)
        self.assertIn("MAX_EDITOR_MESSAGE_SCHEMA_ID_BYTES: usize = 256", schema)
        self.assertIn('const EDITOR_NAMESPACE: &str = "editor"', schema)
        self.assertIn('const PLUGIN_NAMESPACE: &str = "plugin"', schema)
        self.assertIn("Self::parse(value).map_err(serde::de::Error::custom)", schema)
        self.assertIn("MissingPluginIdentityOrSchema", schema)

    def test_custom_payload_and_constructor_do_not_accept_raw_schema_strings(self) -> None:
        payload = self.read(
            "zircon_editor/src/core/editor_message/message/payload.rs"
        )
        envelope = self.read(
            "zircon_editor/src/core/editor_message/message/envelope.rs"
        )

        self.assertIn("schema_id: EditorMessageSchemaId", payload)
        self.assertNotIn("schema_id: String", payload)
        self.assertIn("fn custom(schema_id: EditorMessageSchemaId", envelope)
        self.assertNotIn("impl Into<String>", envelope)

    def test_all_custom_message_calls_use_typed_schema_values(self) -> None:
        source_root = ROOT / "zircon_editor" / "src"
        offenders = []
        for path in source_root.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            if re.search(r"EditorMessage::custom\(\s*\"", source):
                offenders.append(str(path.relative_to(ROOT)))
            if re.search(r"schema_id:\s*\"", source):
                offenders.append(str(path.relative_to(ROOT)))

        self.assertEqual(offenders, [])

        reflection = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_reflection.rs"
        )
        self.assertEqual(reflection.count("use std::sync::OnceLock;"), 1)


if __name__ == "__main__":
    unittest.main()
