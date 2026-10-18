"""Read a Rust unit-test owner independently from its production source."""
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def read_rust_test_file(relative_path: str) -> str:
    path = Path(relative_path)
    if path.is_absolute() or '..' in path.parts or 'tests' not in path.parts:
        raise ValueError('Rust test-source checks must name a repository tests directory')
    return (ROOT / path).read_text(encoding='utf-8')
