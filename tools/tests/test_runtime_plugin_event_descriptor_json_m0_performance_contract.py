from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/dynamic_api/session/event_mirror.rs"


class RuntimePluginEventDescriptorJsonM0PerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("fn encode_plugin_event_prefix(")
        end = cls.source.index("fn check_plugin_event_encoding_deadline", start)
        cls.encoder = cls.source[start:end]

    def test_subscription_state_retains_preencoded_descriptor_json(self) -> None:
        self.assertIn("event_id_json: Box<[u8]>", self.source)
        self.assertIn("payload_schema_json: Box<[u8]>", self.source)
        self.assertIn(
            "encode_plugin_event_descriptor(&request.event_id)", self.source
        )
        self.assertIn(
            "encode_plugin_event_descriptor(&request.payload_schema)", self.source
        )

    def test_page_encoder_writes_cached_descriptor_bytes(self) -> None:
        self.assertIn("bytes.write_all(event_id_json)?", self.encoder)
        self.assertIn("bytes.write_all(payload_schema_json)?", self.encoder)
        self.assertNotIn("serde_json::to_writer(&mut bytes, event_id)", self.encoder)
        self.assertNotIn(
            "serde_json::to_writer(&mut bytes, payload_schema)", self.encoder
        )

    def test_page_encode_attempt_is_observable(self) -> None:
        self.assertEqual(
            self.encoder.count('"plugin_event.page_encode_attempt"'), 1
        )


if __name__ == "__main__":
    unittest.main()
