#!/usr/bin/env python3
"""Find lines exceeding the 80-character width limit in book markdown files."""

import re
import sys
from pathlib import Path


# Maximum allowed line width for prose lines.
MAX_WIDTH = 80

# Patterns for exempt lines.
HEADING_RE = re.compile(r'^#{1,6}\s')
LINK_REF_DEF_RE = re.compile(r'^\[([^\]]+)\]:\s')
FOOTNOTE_DEF_RE = re.compile(r'^\[\^[^\]]+\]:\s')
HTML_TAG_RE = re.compile(r'^</?[a-zA-Z][^>]*>$')
SINGLE_INLINE_MATH_RE = re.compile(r'^\$[^$]+\$$')
FENCE_RE = re.compile(r'^\s*```')
DISPLAY_MATH_RE = re.compile(r'^\$\$')
TABLE_ROW_RE = re.compile(r'^\|.*\|$')

# Patterns for inherent exceptions (over-long but irreducible).
MARKDOWN_LINK_RE = re.compile(r'\[[^\]]*\]\([^)]*\)')
MATH_PAREN_RE = re.compile(r'\([^)]*\$[^$]+\$[^)]*\)')
JOINING_PUNCT_RE = re.compile(r'^[/,.:;!?\s]*$')


def find_book_src() -> Path:
    """Find the book source directory relative to this script."""
    script_dir = Path(__file__).resolve().parent
    book_src = script_dir.parent.parent / "book" / "src"
    if not book_src.exists():
        raise FileNotFoundError(f"Book source directory not found at {book_src}")
    return book_src


def is_exempt(line: str) -> bool:
    """Check whether a single line is exempt from the width limit."""
    stripped = line.strip()

    # Empty or blank lines.
    if not stripped:
        return True

    # Headings.
    if HEADING_RE.match(stripped):
        return True

    # Link reference definitions (may contain long URLs).
    if LINK_REF_DEF_RE.match(stripped):
        return True

    # Footnote definitions (may contain long URLs).
    if FOOTNOTE_DEF_RE.match(stripped):
        return True

    # Single HTML tags (e.g. <details>, </summary>).
    if HTML_TAG_RE.match(stripped):
        return True

    # Lines that are a single inline math expression.
    if SINGLE_INLINE_MATH_RE.match(stripped):
        return True

    # Table rows (pipes at start and end).
    if TABLE_ROW_RE.match(stripped):
        return True

    return False


def is_inherent_exception(line: str) -> bool:
    """Check whether an over-long line is an inherent exception.

    Inherent exceptions are lines that cannot be shortened without breaking
    content (e.g. lines made entirely of markdown links, or parentheticals
    containing inline math).
    """
    stripped = line.strip()

    # Link-only line: after removing all [text](url), only punctuation remains.
    without_links = MARKDOWN_LINK_RE.sub('', stripped)
    if without_links != stripped and JOINING_PUNCT_RE.match(without_links):
        return True

    # Math-parenthetical: line contains (... $...$ ...).
    if MATH_PAREN_RE.search(stripped):
        return True

    return False


def check_file(path: Path) -> tuple[list[tuple[int, int, str]], int]:
    """Check a single markdown file for overlong lines.

    Returns a (violations, inherent_count) tuple where violations is a list
    of (line_number, width, content) tuples.
    """
    violations = []
    inherent_count = 0
    in_fenced_block = False
    in_display_math = False

    lines = path.read_text().split('\n')

    for line_num, line in enumerate(lines, start=1):
        # Toggle fenced code blocks.
        if FENCE_RE.match(line):
            in_fenced_block = not in_fenced_block
            continue

        # Toggle display math blocks.
        if DISPLAY_MATH_RE.match(line.strip()):
            in_display_math = not in_display_math
            continue

        # Skip lines inside fenced code blocks or display math.
        if in_fenced_block or in_display_math:
            continue

        # Skip per-line exemptions.
        if is_exempt(line):
            continue

        width = len(line)
        if width > MAX_WIDTH:
            if is_inherent_exception(line):
                inherent_count += 1
            else:
                violations.append((line_num, width, line))

    return violations, inherent_count


def collect_files(args: list[str], book_src: Path) -> list[Path]:
    """Resolve CLI arguments into a sorted list of markdown files."""
    if not args:
        return sorted(book_src.rglob('*.md'))

    files = []
    for arg in args:
        p = Path(arg)
        if p.is_dir():
            files.extend(p.rglob('*.md'))
        elif p.is_file():
            files.append(p)
        else:
            print(f"Warning: {arg} not found, skipping", file=sys.stderr)
    return sorted(files)


def main() -> int:
    """Main entry point."""
    try:
        book_src = find_book_src()
    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    files = collect_files(sys.argv[1:], book_src)
    if not files:
        print("No markdown files found.")
        return 0

    total_violations = 0
    total_inherent = 0
    files_with_violations = 0

    for path in files:
        violations, inherent_count = check_file(path)
        total_inherent += inherent_count
        if not violations:
            continue

        files_with_violations += 1
        total_violations += len(violations)

        try:
            rel = path.relative_to(book_src)
        except ValueError:
            rel = path

        print(f"\n{rel} ({len(violations)} violation(s)):")
        for line_num, width, content in violations:
            truncated = content[:90] + "..." if len(content) > 90 else content
            print(f"  L{line_num} ({width} chars): {truncated}")

    inherent_note = ""
    if total_inherent > 0:
        s = "s" if total_inherent != 1 else ""
        inherent_note = f" ({total_inherent} inherent exception{s} skipped)"

    if total_violations == 0:
        print(f"No lines exceed the 80-character limit.{inherent_note}")
        return 0

    print(f"\n{total_violations} violation(s) in {files_with_violations} file(s).{inherent_note}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
