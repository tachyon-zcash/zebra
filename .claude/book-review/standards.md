# Book Standards

Master standards that apply to all book reviewers. Every reviewer agent reads
this file in addition to its focus-specific policy.

## Page Roles

The book's introduction (`book/src/introduction.md`) serves as a landing page,
not a narrative chapter. Its purpose is to briefly describe the project and
direct readers to the appropriate sections. When reviewing the introduction:

- Do NOT flag it for missing motivational openings, narrative transitions, or
  closing summaries.
- Do NOT flag jargon that is linked to a page where it is defined. Inline
  expansion of every term is not expected on a landing page.
- DO review it for clarity, accuracy, and link coverage.

## Link Integrity

Any change that alters a page's file path, moves a file, or renames a heading
must include corresponding updates to every reference pointing to that content.
This includes:

- Internal markdown links (`[text](path.md)`) throughout the entire book
- Anchor links (`[text](path.md#heading-slug)`) that reference renamed headings
- Entries in `book/src/SUMMARY.md`

When reviewing changes that move or rename content, verify that all affected
links have been updated — not just links in the changed file, but in every file
that references it. A renamed heading with no updated anchors elsewhere is a
broken link waiting to happen.
- Every heading that is the target of an anchor link must have an explicit
  `{#slug}` attribute (e.g., `## My Heading {#slug}`). Do not rely on
  mdbook's auto-generated slugs — they break silently when heading text
  changes. Slugs should be concise and deliberately chosen.
- Flag any anchor link (`#slug`) whose target heading lacks an explicit
  `{#slug}` attribute.

## Citations

Academic-style citation tags (e.g., `[BGH19]`, `[BCTV14]`) follow these
rules:

- Every citation tag must be a hyperlink to its source (typically an ePrint
  or conference URL). A bare, unlinked citation tag is a must-fix finding.
- Nested brackets are acceptable when a citation tag appears inside link
  text (e.g., `[Halo [BGH19]](url)`). Do not flag this as a style issue.

## Deferred Issues

The file `book/FIXME.md` tracks known issues that were identified during
review but deferred for later resolution. The lifecycle is:

1. **Defer**: During review triage, the user marks a finding as "defer" —
   it is recorded in `book/FIXME.md` as a `###`-level entry.
2. **Suppress**: Reviewers are given the list of deferred issues and must
   not re-raise them as findings.
3. **Resolve**: When a solution is available, use `/book-fixme` to apply
   the fix and remove the entry.

Reviewers should not re-raise deferred issues, but SHOULD note when a
deferred issue appears to have been resolved or become easily resolvable
given surrounding changes.