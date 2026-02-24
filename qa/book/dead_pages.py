#!/usr/bin/env python3
"""Find markdown files in the book that are not rendered (not in SUMMARY.md)."""

import re
import sys
from pathlib import Path


def find_book_src() -> Path:
    """Find the book source directory relative to this script."""
    script_dir = Path(__file__).resolve().parent
    book_src = script_dir.parent.parent / "book" / "src"
    if not book_src.exists():
        raise FileNotFoundError(f"Book source directory not found at {book_src}")
    return book_src


def parse_summary(summary_path: Path) -> set[Path]:
    """Extract all .md file paths listed in SUMMARY.md."""
    files = set()
    content = summary_path.read_text()

    # Match markdown links: [text](path.md) or [text](path.md#anchor)
    link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')

    for match in link_pattern.finditer(content):
        target = match.group(2)
        # Strip anchor if present
        if '#' in target:
            target = target.split('#')[0]
        # Skip external URLs
        if target.startswith(('http://', 'https://', 'mailto:')):
            continue
        # Skip empty targets (anchor-only links)
        if not target:
            continue

        resolved = (summary_path.parent / target).resolve()
        files.add(resolved)

    return files


def find_dead_pages(book_src: Path, summary_files: set[Path]) -> list[Path]:
    """Find markdown files that exist but are not in SUMMARY.md."""
    dead = []

    for md_file in book_src.rglob('*.md'):
        resolved = md_file.resolve()
        # Skip SUMMARY.md itself
        if md_file.name == 'SUMMARY.md':
            continue
        if resolved not in summary_files:
            dead.append(resolved)

    return sorted(dead)


def main() -> int:
    """Main entry point."""
    try:
        book_src = find_book_src()
    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    summary_path = book_src / "SUMMARY.md"
    if not summary_path.exists():
        print(f"Error: SUMMARY.md not found at {summary_path}", file=sys.stderr)
        return 1

    print(f"Checking for dead pages in: {book_src}")

    summary_files = parse_summary(summary_path)
    print(f"Found {len(summary_files)} files in SUMMARY.md")

    dead_pages = find_dead_pages(book_src, summary_files)

    if not dead_pages:
        print("\nNo dead pages found!")
        return 0

    print(f"\nFound {len(dead_pages)} dead page(s) (not in SUMMARY.md):\n")
    for page in dead_pages:
        rel_path = page.relative_to(book_src)
        print(f"  {rel_path}")

    return 1


if __name__ == "__main__":
    sys.exit(main())
