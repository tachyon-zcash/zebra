# Mathematical Accuracy Review Policy

## Scope

Review mathematical content in book chapter markdown files (`book/src/`). Focus
on correctness, consistency, and clarity of mathematical exposition.

In addition to `.claude/review-shared/math.md` (shared math notation rules that apply
to all project content), the following book-specific rules apply.

## Notation

- All notation should be consistent with the KaTeX macros in `book/macros.txt`.
- When a LaTeX command name is verbose or unclear, define a short macro in
  `book/macros.txt` (e.g., `\nil` for `\bot`). The only exception is contexts
  where KaTeX cannot render, such as mermaid diagram labels.

## Definitions and Claims

- Every definition should be precise enough to be unambiguous.
- Claims should be clearly distinguishable as definitions, theorems, lemmas,
  or informal observations. Flag ambiguous "this means that..." statements that
  could be either a definition or a derived fact.
- Flag hand-wavy justifications ("it's easy to see", "clearly", "obviously") if
  the claim is non-trivial.

## Correctness

- Verify that mathematical statements are internally consistent within the
  chapter.
- Check that protocol descriptions match the formal definitions they claim to
  implement.
- Flag any step in a construction or proof sketch that appears to be missing or
  glossed over.

## Accessibility

- Complex formulas should have verbal explanations nearby.
- Flag passages where adding geometric or algebraic intuition would make the
  content more approachable.
- Flag jargon not defined in the chapter or in `book/src/appendix/terminology.md`.
