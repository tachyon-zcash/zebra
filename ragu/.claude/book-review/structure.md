# Structural Review Policy

## Scope

Review organization and flow in book chapter markdown files (`book/src/`). All
rules in `.claude/book-review/standards.md` also apply.

## Chapter-Level

- Each chapter should open with motivation: why does this topic matter to the
  reader right now?
- Sections should progress logically. Flag sections that feel out of order or
  that could be reordered for clarity.
- Chapters should close with either a summary, a forward-looking transition, or
  both.

## Section Categories

Not every section in the book is a standalone page. Some sections are
**category headings** — they group related subpages under a label but have no
meaningful content of their own (e.g. "Preliminaries", "Part I: User Guide").
Category headings:

- Should NOT have an `index.md` file.
- Should use an empty `()` link in `SUMMARY.md` so they are not linkable.
- Should NOT be flagged for missing motivation, transitions, or summaries —
  the rules in "Chapter-Level" apply only to pages with real content.

Flag an `index.md` that contains only a title or a sentence or two of generic
framing and suggest converting the section to a category heading. Conversely,
if a section's `index.md` has substantive content worth reading on its own, it
should remain a linked page.

## Section-Level

- Headings should form a clear hierarchy. Don't skip levels.
- Each section should have a single focus. Flag sections trying to cover too
  much ground.
- Long sections (roughly >100 lines) should usually be broken into subsections.
- Headings in `SUMMARY.md` (and the corresponding page titles) should not
  redundantly restate their parent section. When a page title repeats a word
  from its parent and the remaining words still clearly identify the page's
  role, drop the repeated word (e.g., "Protocol Overview" under "Protocol
  Design" should be just "Overview"). Do NOT shorten when the compound phrase
  is a term of art ("Cryptographic Assumptions", "Simple Gadgets"), when the
  apparently-repeated word adds meaning beyond what the parent provides
  ("Architecture Overview" under "Implementation"), or when shortening would
  leave the title too vague to stand alone.

## Progressive Disclosure

- Concepts should build on each other. Flag complexity jumps where the reader
  needs to absorb too much at once.
- Concrete examples should precede or accompany abstract definitions, not follow
  them as afterthoughts.
- Flag places where the reader needs knowledge from a later section or chapter.
- Within a section, pages must be ordered so that each page depends only on
  concepts, notation, and conventions introduced by preceding pages in the
  same section. Flag any page that uses notation, terminology, or concepts
  defined by a later sibling page. This is especially important in
  foundational/reference sections (like Preliminaries) where the pages are a
  curated collection rather than a narrative arc.

## Lists and Formatting

- Prefer prose over bullet lists for narrative content. Lists work best for
  enumerations, requirements, and reference material.
- Code blocks should have surrounding context: what the code does, what the
  reader should pay attention to.

## Cross-References

- Flag broken or misleading references to other chapters or sections.
- Verify that "as we saw in..." and "we'll see in..." references are accurate.
- When the reviewed content moves or renames pages or headings, verify that all
  inbound links from other pages have been updated (see standards.md "Link
  Integrity").