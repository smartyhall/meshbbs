#!/usr/bin/env python3
"""Validate UTF-8 byte length budgets for mesh-ready message payloads.

The tool walks files or directories (recursively) and verifies that each
non-empty line stays within the specified byte budget.  It is designed to
cover TinyMUSH transcripts, help text fixtures, and developer documentation
snippets that are expected to fit inside the 200-byte Meshtastic envelope.
"""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Sequence

DEFAULT_EXTENSIONS = [".txt", ".out", ".yaml", ".yml", ".md"]
DEFAULT_IGNORE_PREFIXES = ["//", "#"]


@dataclass
class Failure:
    path: Path | str
    line_no: int
    byte_len: int
    limit: int
    preview: str

    def format(self) -> str:
        location = f"{self.path}:{self.line_no}" if isinstance(self.path, Path) else "stdin"
        return (
            f"{location} exceeds limit ({self.byte_len} > {self.limit} bytes): "
            f"{self.preview}"
        )


def iter_paths(
    inputs: Sequence[str],
    extensions: Sequence[str],
) -> Iterable[Path | str]:
    """Yield files (and optional stdin sentinel) that should be checked."""

    normalized_exts = {ext.lower() for ext in extensions}
    for raw in inputs:
        if raw == "-":
            yield raw
            continue
        path = Path(raw)
        if not path.exists():
            raise FileNotFoundError(raw)
        if path.is_file():
            if not normalized_exts or path.suffix.lower() in normalized_exts:
                yield path
            continue
        for file_path in sorted(path.rglob("*")):
            if file_path.is_file() and (
                not normalized_exts or file_path.suffix.lower() in normalized_exts
            ):
                yield file_path


def check_stream(
    text: Iterable[str],
    limit: int,
    ignore_prefixes: Sequence[str],
    source: Path | str,
) -> List[Failure]:
    failures: List[Failure] = []
    for idx, raw_line in enumerate(text, start=1):
        line = raw_line.rstrip("\n")
        stripped = line.strip()
        if not stripped:
            continue
        if any(stripped.startswith(prefix) for prefix in ignore_prefixes):
            continue
        byte_len = len(line.encode("utf-8"))
        if byte_len > limit:
            preview = stripped
            if len(preview) > 60:
                preview = preview[:57] + "..."
            failures.append(Failure(source, idx, byte_len, limit, preview))
    return failures


def run(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "paths",
        nargs="*",
        default=["tests/test-data-int", "docs/qa"],
        help="Files or directories to scan; use '-' to read from stdin",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=200,
        help="Maximum allowed UTF-8 byte length per line (default: 200)",
    )
    parser.add_argument(
        "--extensions",
        default=",".join(DEFAULT_EXTENSIONS),
        help=(
            "Comma-separated list of file extensions to include. "
            "Use an empty string to scan every file."
        ),
    )
    parser.add_argument(
        "--ignore-prefix",
        default=",".join(DEFAULT_IGNORE_PREFIXES),
        help="Comma-separated prefixes to ignore (default: //,#)",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress non-error output",
    )

    args = parser.parse_args(argv)

    extensions = [ext.strip() for ext in args.extensions.split(",") if ext.strip()]
    ignore_prefixes = [p.strip() for p in args.ignore_prefix.split(",") if p.strip()]

    failures: List[Failure] = []
    checked = 0
    for entry in iter_paths(args.paths, extensions):
        if entry == "-":
            failures.extend(
                check_stream(sys.stdin, args.limit, ignore_prefixes, source="stdin")
            )
            checked += 1
            continue
        with Path(entry).open("r", encoding="utf-8") as handle:
            failures.extend(
                check_stream(handle, args.limit, ignore_prefixes, source=Path(entry))
            )
        checked += 1

    if not args.quiet:
        print(
            f"Checked {checked} {'file' if checked == 1 else 'files'}"
            f" with {len(failures)} violation(s)."
        )

    if failures:
        for failure in failures:
            print(failure.format(), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(run())
