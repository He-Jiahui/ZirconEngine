"""Move Rust unit-test modules into tests directories without changing module names.

Run with --apply to migrate; the default prints a read-only inventory. Source
contents are never backed up. The receipt stores paths, hashes and test counts.
"""
from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib


ROOT = Path(__file__).resolve().parents[2]
KNOWN_FILES: set[Path] | None = None
SPECIAL = re.compile(r'//|/\*|(?:br|cr|r)\#*"|(?:b|c)?"|(?:b)?\'(?:\\(?:u\{[^}]*\}|x[0-9a-fA-F]{2}|.)|[^\'\\\r\n])\'')
MODULE = re.compile(r'\b(?:pub(?:\([^)]*\))?\s+)?mod\s+(?P<name>\w+)\s*(?P<end>[;{])')
TEST_ATTRIBUTE = re.compile(r'#\s*\[\s*(?:test|(?:\w+::)+test)\s*(?:\([^\]]*\))?\s*\]')
PATH = re.compile(r'#\s*\[\s*path\s*=\s*"([^"\n]+)"\s*\]')
INCLUDE = re.compile(r'\b(include|include_str|include_bytes)!\s*\(\s*"([^"\n]+)"\s*\)')


def rust_files() -> list[Path]:
    if shutil.which('rg') is None:
        files = []
        for directory, children, leaves in os.walk(ROOT):
            children[:] = [name for name in children if not name.startswith('.') and name != 'node_modules'
                           and not (Path(directory) == ROOT and name in {'dev', 'third_party', 'target', 'artifacts'})]
            files.extend(Path(directory) / leaf for leaf in leaves if leaf.endswith('.rs'))
        return sorted(files)
    result = subprocess.run(
        ['rg', '--files', '--no-ignore-vcs', '-g', '*.rs', '-g', '!dev/**', '-g', '!third_party/**',
         '-g', '!**/node_modules/**', '-g', '!target/**', '-g', '!artifacts/**'],
        cwd=ROOT, capture_output=True, text=True, check=True,
    )
    return sorted(ROOT / line for line in result.stdout.splitlines())


def read(path: Path) -> str:
    # Keep existing line endings and BOMs in the canonical source.
    return path.read_bytes().decode('utf-8')


def write_canonical(path: Path, source: str) -> None:
    # Some Windows files reject CREATE_ALWAYS despite permitting in-place
    # writes. Use the existing canonical handle; never stage or rename sources.
    desired = source.encode('utf-8')
    for attempt in range(6):
        try:
            with path.open('r+b' if path.exists() else 'x+b') as stream:
                original = stream.read()
                if original == desired:
                    return
                stream.seek(0)
                stream.write(desired)
                stream.flush()
                if len(desired) < len(original):
                    try:
                        stream.truncate(len(desired))
                    except OSError:
                        # A transient Windows mapping can reject shortening.
                        # Restore the canonical contents before retrying.
                        stream.seek(0)
                        stream.write(original)
                        stream.flush()
                        raise
            return
        except OSError as error:
            if error.errno not in {13, 22} or attempt == 5:
                raise OSError(f'canonical write failed for {path}: {error}') from error
            time.sleep(0.15 * (attempt + 1))


def mask_rust(source: str) -> tuple[str, list[tuple[int, int]]]:
    """Mask comments/literals, including nested comments and raw Rust strings."""
    pieces: list[str] = []
    literals: list[tuple[int, int]] = []
    position = 0
    while match := SPECIAL.search(source, position):
        start = match.start()
        token = match.group()
        if token == '//':
            end = source.find('\n', match.end())
            end = len(source) if end < 0 else end
        elif token == '/*':
            end, depth = match.end(), 1
            while depth:
                opening, closing = source.find('/*', end), source.find('*/', end)
                if closing < 0:
                    raise ValueError('unterminated Rust block comment')
                if 0 <= opening < closing:
                    depth += 1
                    end = opening + 2
                else:
                    depth -= 1
                    end = closing + 2
        elif token.endswith('"') and token[0] == 'r' or token.startswith(('br', 'cr')):
            hashes = token.count('#')
            terminator = '"' + '#' * hashes
            closing = source.find(terminator, match.end())
            if closing < 0:
                raise ValueError('unterminated Rust raw string')
            end = closing + len(terminator)
            literals.append((start, end))
        elif token.endswith('"'):
            end = match.end()
            while end < len(source):
                if source[end] == '\\':
                    end += 2
                elif source[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            literals.append((start, end))
        else:
            end = match.end()
            literals.append((start, end))
        pieces.extend((source[position:start], re.sub(r'[^\r\n]', ' ', source[start:end])))
        position = end
    pieces.append(source[position:])
    return ''.join(pieces), literals


@dataclass
class Module:
    name: str
    start: int
    keyword: int
    delimiter: int
    end: int
    attributes: str
    inline: bool
    parents: tuple[str, ...]

    @property
    def test_only(self) -> bool:
        attributes = mask_rust(self.attributes)[0]
        return any(
            re.search(r'\btest\b', match[1]) and cfg_without_tests(match[1]) is False
            for match in re.finditer(r'#\s*\[\s*cfg\s*\(([^\]]*)\)\s*\]', attributes)
        )


def cfg_without_tests(expression: str) -> bool | None:
    """Evaluate with test disabled and other cfg atoms unknown."""
    expression = expression.strip()
    if expression == 'test':
        return False
    match = re.fullmatch(r'(all|any|not)\s*\((.*)\)', expression, flags=re.DOTALL)
    if not match:
        return None
    args = []
    depth = start = 0
    for position, char in enumerate(match[2]):
        if char == '(':
            depth += 1
        elif char == ')':
            depth -= 1
        elif char == ',' and depth == 0:
            args.append(match[2][start:position])
            start = position + 1
    if match[2][start:].strip():
        args.append(match[2][start:])
    values = [cfg_without_tests(arg) for arg in args]
    if match[1] == 'not':
        return not values[0] if len(values) == 1 and values[0] is not None else None
    if match[1] == 'all':
        return False if False in values else (True if all(value is True for value in values) else None)
    return True if True in values else (False if all(value is False for value in values) else None)


def modules(source: str) -> list[Module]:
    masked, _ = mask_rust(source)
    pairs: dict[int, int] = {}
    stack: list[int] = []
    for match in re.finditer(r'[{}\[\]()]', masked):
        offset = match.start()
        if match.group() in '{[(':
            stack.append(offset)
        else:
            if not stack:
                raise ValueError(f'unbalanced Rust delimiter at {offset}')
            opening = stack.pop()
            pairs[offset] = opening
            pairs[opening] = offset
    result: list[Module] = []
    for match in MODULE.finditer(masked):
        start = match.start()
        previous = start - 1
        while previous >= 0 and masked[previous].isspace():
            previous -= 1
        while previous >= 0 and masked[previous] == ']':
            opening = pairs[previous]
            hash_position = opening - 1
            while hash_position >= 0 and masked[hash_position].isspace():
                hash_position -= 1
            if hash_position < 0 or masked[hash_position] != '#':
                break
            start = hash_position
            previous = start - 1
            while previous >= 0 and masked[previous].isspace():
                previous -= 1
        delimiter = match.end() - 1
        inline = match['end'] == '{'
        end = pairs[delimiter] + 1 if inline else delimiter + 1
        parents = tuple(m.name for m in result if m.inline and m.delimiter < start < m.end)
        result.append(Module(match['name'], start, match.start(), delimiter, end,
                             source[start:match.start()], inline, parents))
    return result


def test_count(source: str) -> int:
    return len(TEST_ATTRIBUTE.findall(mask_rust(source)[0]))


def is_package_build_script(path: Path) -> bool:
    return path.name == 'build.rs' and (path.parent / 'Cargo.toml').is_file()


def module_base(path: Path) -> Path:
    root = path.name in {'lib.rs', 'main.rs', 'mod.rs'} or is_package_build_script(path)
    return path.parent if root else path.with_suffix('')


def resolve_module(path: Path, module: Module, context: tuple[str, ...] = ()) -> Path | None:
    explicit = PATH.search(module.attributes)
    if explicit:
        parents = context + module.parents
        base = module_base(path).joinpath(*parents) if parents else path.parent
        candidate = Path(os.path.normpath(base / explicit[1]))
        exists = candidate in KNOWN_FILES if KNOWN_FILES is not None else candidate.is_file()
        if exists:
            return candidate
        # Repair a stale sibling attribute only when the unambiguous default
        # module file exists (for example bake/foo_tests.rs from bake.rs).
        if not module.parents and not context and explicit[1] == f'{module.name}.rs':
            candidate = module_base(path) / explicit[1]
            exists = candidate in KNOWN_FILES if KNOWN_FILES is not None else candidate.is_file()
            if exists:
                return candidate
        return None
    base = module_base(path).joinpath(*(context + module.parents))
    for candidate in (base / f'{module.name}.rs', base / module.name / 'mod.rs'):
        exists = candidate in KNOWN_FILES if KNOWN_FILES is not None else candidate.is_file()
        if exists:
            return candidate
    return None


def relative(path: Path, base: Path) -> str:
    return Path(os.path.relpath(path, base)).as_posix()


def test_destination(path: Path) -> Path:
    directory = path.parent / 'tests'
    if (path.parent / 'Cargo.toml').is_file():
        # Files directly in a package's tests/ become integration targets.
        # These remain unit modules and must retain their private parent scope.
        directory /= 'unit'
    return directory / ('cases.rs' if path.name == 'tests.rs' else path.name)


def deindent(body: str) -> str:
    """Remove a module's indentation while preserving multiline literal bytes."""
    masked, literals = mask_rust(body)
    offsets = [0, *(m.end() for m in re.finditer('\n', body))]
    removable: list[int] = []
    for offset in offsets:
        line_end = body.find('\n', offset)
        line_end = len(body) if line_end < 0 else line_end
        if masked[offset:line_end].strip():
            removable.append(len(body[offset:line_end]) - len(body[offset:line_end].lstrip(' ')))
    indent = min(removable, default=0)
    lines: list[str] = []
    offset = 0
    for line in body.splitlines(keepends=True):
        in_literal = any(start < offset < end for start, end in literals)
        remove = 0 if in_literal else min(indent, len(line) - len(line.lstrip(' ')))
        lines.append(line[remove:])
        offset += len(line)
    return ''.join(lines).strip('\r\n') + '\n'


def rewrite_references(source: str, old: Path, new: Path, moves: dict[Path, Path],
                       context: tuple[str, ...] = ()) -> str:
    edits: list[tuple[int, int, str]] = []
    for module in modules(source):
        if module.inline:
            continue
        target = resolve_module(old, module, context)
        if target is None:
            continue
        original_target = target
        target = moves.get(target, target)
        if new == old and target == original_target:
            continue
        path_attribute = PATH.search(module.attributes)
        if new != old and not module.parents and path_attribute is None and target in {
            new.parent / f'{module.name}.rs', new.parent / module.name / 'mod.rs',
        }:
            # Attributed unit roots own their new directory. Preserve a
            # default child declaration when it still reaches the same file;
            # adding #[path] would change that child's nested-module base.
            continue
        base = module_base(new).joinpath(*module.parents) if module.parents else new.parent
        attribute = '#[path = ' + json.dumps(relative(target, base)) + ']'
        if path_attribute:
            edits.append((module.start + path_attribute.start(), module.start + path_attribute.end(), attribute))
        else:
            line_start = source.rfind('\n', 0, module.keyword) + 1
            indent = source[line_start:module.keyword]
            indent = indent if not indent.strip() else ''
            newline = '\r\n' if '\r\n' in source else '\n'
            edits.append((module.keyword, module.keyword, attribute + newline + indent))
    masked, _ = mask_rust(source)
    for match in INCLUDE.finditer(source):
        if not masked[match.start():match.start() + len(match[1])].strip():
            continue
        old_target = Path(os.path.normpath(old.parent / match[2]))
        target = moves.get(old_target, old_target)
        if new != old or target != old_target:
            start, end = match.span(2)
            edits.append((start, end, relative(target, new.parent)))
    for start, end, replacement in sorted(edits, reverse=True):
        source = source[:start] + replacement + source[end:]
    return source


def inventory(files: list[Path], sources: dict[Path, str]) -> tuple[list[tuple[Path, Module]], set[Path], list[dict]]:
    inline: list[tuple[Path, Module]] = []
    moving: set[Path] = set()
    unresolved: list[dict] = []
    for path in files:
        if 'tests' in path.relative_to(ROOT).parts:
            continue
        source = sources[path]
        if not re.search(r'\btest\b|tests', source):
            continue
        seen_inline: list[Module] = []
        for module in modules(source):
            if any(parent.delimiter < module.start < parent.end for parent in seen_inline):
                continue
            if not module.test_only:
                continue
            if module.inline:
                inline.append((path, module))
                seen_inline.append(module)
            else:
                target = resolve_module(path, module)
                if (target is not None and 'tests' not in target.relative_to(ROOT).parts
                        and not is_package_build_script(target)):
                    moving.add(target)
                elif target is None:
                    unresolved.append({'source': relative(path, ROOT), 'module': module.name})
        if path.stem == 'tests' or path.stem.endswith('_tests') or (
            'test_sources' in path.parts and test_count(source)
        ):
            moving.add(path)
    # Child modules and include! helpers inherit their test owner's directory.
    pending = list(moving)
    while pending:
        path = pending.pop()
        for module in modules(sources[path]):
            if module.inline:
                continue
            target = resolve_module(path, module)
            if (target in sources and target not in moving
                    and 'tests' not in target.relative_to(ROOT).parts
                    and not is_package_build_script(target)):
                moving.add(target)
                pending.append(target)
    return inline, moving, unresolved


def migrate(files: list[Path], sources: dict[Path, str], receipt: Path, apply: bool) -> dict:
    inline, moving, unresolved = inventory(files, sources)
    moves = {old: test_destination(old) for old in moving}
    outputs: dict[Path, str] = {}
    extractions: list[dict] = []
    by_source: dict[Path, list[Module]] = {}
    for path, module in inline:
        if path not in moving:
            by_source.setdefault(path, []).append(module)
    for path, children in by_source.items():
        source = sources[path]
        for module in reversed(children):
            leaf = 'cases' if path.stem == 'mod' and module.name == 'tests' else path.stem
            if module.name != 'tests':
                leaf += '_' + module.name
            target = test_destination(path).with_name(f'{leaf}.rs')
            if target.exists() or target in outputs or target in moves.values():
                target = target.with_name(target.stem + '_unit.rs')
            if target.exists() or target in outputs or target in moves.values():
                raise ValueError(f'test destination collision: {target}')
            original_body = source[module.delimiter + 1:module.end - 1]
            body = deindent(original_body)
            if token_hash(original_body) != token_hash(body):
                raise ValueError(f'extraction changed Rust tokens: {path}::{module.name}')
            # A formerly inline test module uses its logical module directory
            # for children, but its containing file for include_* paths.
            body = rewrite_references(body, path, target, moves, module.parents + (module.name,))
            outputs[target] = body
            line_start = source.rfind('\n', 0, module.keyword) + 1
            indent = source[line_start:module.keyword]
            indent = indent if not indent.strip() else ''
            base = module_base(path).joinpath(*module.parents) if module.parents else path.parent
            hook = '#[path = ' + json.dumps(relative(target, base)) + ']\n' + indent
            source = source[:module.keyword] + hook + source[module.keyword:module.delimiter].rstrip() + ';' + source[module.end:]
            extractions.append({'source': relative(path, ROOT), 'test_file': relative(target, ROOT),
                                'module': module.name, 'tests': test_count(body),
                                'tokens_sha256': token_hash(body)})
        outputs[path] = source
    for old, new in moves.items():
        if new.exists() or new in outputs:
            raise ValueError(f'test destination collision: {new}')
        outputs[new] = rewrite_references(sources[old], old, new, moves)
    for path in files:
        if path in moving:
            continue
        source = outputs.get(path, sources[path])
        rewritten = rewrite_references(source, path, path, moves)
        if rewritten != sources[path]:
            outputs[path] = rewritten
    before_count = sum(test_count(source) for source in sources.values())
    after_count = sum(test_count(outputs.get(path, source)) for path, source in sources.items() if path not in moving)
    after_count += sum(test_count(source) for path, source in outputs.items() if path not in sources)
    if before_count != after_count:
        raise ValueError(f'test attributes changed: {before_count} -> {after_count}')
    report = {'schema_version': 1, 'rust_files': len(files), 'inline_modules': len(extractions),
              'moved_test_files': len(moves), 'changed_files': len(outputs),
              'test_attributes_before': before_count, 'test_attributes_after': after_count,
              'by_root': dict(Counter(Path(row['source']).parts[0] for row in extractions)),
              'unresolved_existing_modules': unresolved, 'extractions': extractions,
              'moves': [{'old': relative(old, ROOT), 'new': relative(new, ROOT),
                         'before_sha256': hashlib.sha256(sources[old].encode()).hexdigest()}
                        for old, new in sorted(moves.items())],
              'files': [{'path': relative(path, ROOT), 'sha256': hashlib.sha256(source.encode()).hexdigest()}
                        for path, source in sorted(outputs.items())]}
    if apply:
        permissions = tomllib.loads(read(ROOT / '.codex/config.toml'))['permissions']['zircon-strict']['filesystem']
        grants = [Path(name.replace('/', os.sep)).resolve() for name, access in permissions.items()
                  if access == 'write' and not name.startswith(':')]
        for path in {*outputs, *moving, receipt}:
            physical = path.resolve()
            if not any(physical == grant or physical.is_relative_to(grant) for grant in grants):
                raise ValueError(f'destination is outside the explicit file-write grants: {path}')
        print(f'Preflight passed: {len(extractions)} inline modules, {len(moves)} test files, '
              f'{before_count} unchanged test attributes.', flush=True)
        for path, source in outputs.items():
            if path.exists() and read(path) != sources.get(path):
                raise ValueError(f'concurrent change detected: {path}')
        receipt.parent.mkdir(parents=True, exist_ok=True)
        report['status'] = 'prepared'
        write_canonical(receipt, json.dumps(report, indent=2) + '\n')
        for path, source in outputs.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            write_canonical(path, source)
        for old in moving:
            if read(old) != sources[old]:
                raise ValueError(f'concurrent change detected: {old}')
            old.unlink()
        report['status'] = 'source-migrated'
        write_canonical(receipt, json.dumps(report, indent=2) + '\n')
    return report


def token_hash(source: str) -> str:
    masked, literals = mask_rust(source)
    tokens = [(m.start(), m.group()) for m in re.finditer(r'\w+|[^\s]', masked)]
    tokens += [(start, source[start:end]) for start, end in literals]
    return hashlib.sha256(json.dumps([value for _, value in sorted(tokens)], ensure_ascii=False).encode()).hexdigest()


def main() -> int:
    global KNOWN_FILES
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--check', action='store_true', help='Reject tests outside tests directories')
    parser.add_argument('--receipt', type=Path, default=ROOT / '.codex/outbox/rust-tests-separation/receipt.json')
    options = parser.parse_args()
    files = rust_files()
    KNOWN_FILES = set(files)
    sources = {path: read(path) for path in files}
    if options.check:
        violations = []
        for path, source in sources.items():
            if 'tests' in path.relative_to(ROOT).parts:
                continue
            count = test_count(source)
            inline = [m.name for m in modules(source) if m.inline and m.test_only]
            if count or inline or path.stem == 'tests' or path.stem.endswith('_tests'):
                violations.append({'path': relative(path, ROOT), 'test_attributes': count, 'inline_modules': inline})
        print(json.dumps({'rust_files': len(files), 'violations': violations, 'passed': not violations}, indent=2))
        return int(bool(violations))
    report = migrate(files, sources, options.receipt, options.apply)
    print(json.dumps({key: value for key, value in report.items() if key not in {'extractions', 'moves', 'files'}}, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
