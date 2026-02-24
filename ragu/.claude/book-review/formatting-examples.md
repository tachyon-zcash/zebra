# Formatting Examples Catalog

Concrete before/after examples of rewrapping overlong lines. Each entry
demonstrates a distinct pattern. When suggesting rewraps during formatting
review, prefer the patterns shown here.

This catalog grows over time as user corrections are captured via `/book-refine`.

---

## Plain Prose

### plain-clause-break

> One-line rationale: fill to ~80 columns, breaking at a subordinate-clause boundary ("that ...").

<!-- <BEFORE> -->
When building a PCD application with Ragu, you configure three key parameters that determine the system's capacity and behavior.
<!-- </BEFORE> -->

<!-- <AFTER> -->
When building a PCD application with Ragu, you configure three key parameters
that determine the system's capacity and behavior.
<!-- </AFTER> -->

**Notes:** Break falls between "parameters" and "that determine...". Fill lines close to 80 characters — don't break short just because a clause boundary exists earlier in the line.

### plain-trailing-demotion

> One-line rationale: demote trailing word(s) to the next line when a line barely exceeds 80 columns.

<!-- <BEFORE> -->
Ragu operates with a single proof structure that can exist in two different modes:
<!-- </BEFORE> -->

<!-- <AFTER> -->
Ragu operates with a single proof structure that can exist in two different
modes:
<!-- </AFTER> -->

**Notes:** Only 2 characters over the limit, but still rewrap. The fix is minimal: move "modes:" down. Don't leave lines barely over 80.

## Inline Math

### math-keep-expression-intact

> One-line rationale: never split `$...$` across lines; break the surrounding prose to keep the expression whole.

<!-- <BEFORE> -->
Recall that in the standard R1CS, there exists a vector $\v{z}=(1, \v{x}, \v{w})$
<!-- </BEFORE> -->

<!-- <AFTER> -->
Recall that in the standard R1CS, there exists a vector
$\v{z}=(1, \v{x}, \v{w})$
<!-- </AFTER> -->

**Notes:** The inline math `$\v{z}=(1, \v{x}, \v{w})$` must stay on one line. Move it to its own line when keeping it inline would exceed the limit. A lone inline-math expression on its own line is exempt from the 80-char limit.

### math-demote-long-expression

> One-line rationale: short math stays inline like a regular word; a long math expression that would overflow gets its own line.

<!-- <BEFORE> -->
we need to further enforce the consistency between its commitment $\mathring{A}$
and the overall trace polynomial commitment $R\in\G_{nested}=\com(r(X)\in\F_q[X])$.
<!-- </BEFORE> -->

<!-- <AFTER> -->
we need to further enforce the consistency between its commitment $\mathring{A}$
and the overall trace polynomial commitment
$R\in\G_{nested}=\com(r(X)\in\F_q[X])$.
<!-- </AFTER> -->

**Notes:** `$\mathring{A}$` is short enough to stay inline on its line. `$R\in\G_{nested}=\com(r(X)\in\F_q[X])$` is long and gets its own standalone line. Contrast with `math-keep-expression-intact`: the principle is the same (never split `$...$`), but here the passage has a mix of short and long math — only the long one needs demotion.

### math-break-between-expressions

> One-line rationale: when two math expressions appear near each other on an overlong line, break between them, keeping each intact.

<!-- <BEFORE> -->
If we want to operate on the original commitment $A\in\G_{host}$ (say $s\cdot A$),
<!-- </BEFORE> -->

<!-- <AFTER> -->
If we want to operate on the original commitment $A\in\G_{host}$
(say $s\cdot A$), we can't constrain such operation natively in $\F_p$ circuit
<!-- </AFTER> -->

**Notes:** The break falls after `$A\in\G_{host}$` and before the parenthetical `(say $s\cdot A$)`. The gap between two math expressions is a natural break point — each expression stays whole on its respective line.

### math-dense-paragraph

> One-line rationale: fill-wrap a paragraph dense with inline math, treating short math as regular words and demoting long expressions.

<!-- <BEFORE> -->
Recall from above that the $n - 1$ degree coefficient, $c_{n-1}$, equals the revdot product of the vectors, $\revdot{\v{a}}{\v{b}}$. To construct $p$, we take the lower half of $c$ and reverse its coefficients, so the $c_{n-1}$ coefficient becomes the constant term (and thus, $p(0) = c = \revdot{\v{a}}{\v{b}}$):
<!-- </BEFORE> -->

<!-- <AFTER> -->
Recall from above that the $n - 1$ degree coefficient, $c_{n-1}$, equals the
revdot product of the vectors, $\revdot{\v{a}}{\v{b}}$. To construct $p$, we
take the lower half of $c$ and reverse its coefficients, so the $c_{n-1}$
coefficient becomes the constant term (and thus,
$p(0) = c = \revdot{\v{a}}{\v{b}}$):
<!-- </AFTER> -->

**Notes:** Short math tokens like `$c_{n-1}$` and `$p$` flow inline with prose as if they were ordinary words. The long expression `$p(0) = c = \revdot{\v{a}}{\v{b}}$` gets its own line. This is the general case for math-heavy prose: fill-wrap normally, only giving standalone treatment to expressions that would overflow.

### math-parenthetical-unit

> One-line rationale: a parenthetical containing math is kept as an indivisible unit, tolerating an overlong line to avoid an awkward break.

<!-- <BEFORE> -->
and so we can encode pairwise product constraints (of the form $\v{a}_i \cdot \v{b}_i = \v{c}_i$)
into claims about such vectors, using verifier challenges to keep the claims linearly independent.
<!-- </BEFORE> -->

<!-- <AFTER> -->
and so we can encode pairwise product constraints (of the form $\v{a}_i \cdot \v{b}_i = \v{c}_i$)
into claims about such vectors, using verifier challenges to keep the claims
linearly independent.
<!-- </AFTER> -->

**Notes:** `(of the form $\v{a}_i \cdot \v{b}_i = \v{c}_i$)` stays attached to "constraints" as a unit even though the line reaches ~95 chars. Breaking before `(of the form` would leave a short first line; breaking inside the parenthetical is not allowed. Tolerate the overlong line rather than produce an awkward split.

## Markdown Links

### link-list-item-with-description

> One-line rationale: keep `[text](url)` intact; fill the line to ~80 cols then indent continuation by 2 spaces.

<!-- <BEFORE> -->
- **[Writing Circuits](writing_circuits.md)**: Detailed explanation of Steps, Headers, and circuit logic implementation
<!-- </BEFORE> -->

<!-- <AFTER> -->
- **[Writing Circuits](writing_circuits.md)**: Detailed explanation of Steps,
  Headers, and circuit logic implementation
<!-- </AFTER> -->

**Notes:** The bold markdown link stays on the first line with as much description text as fits within 80 chars. Continuation is indented 2 spaces to align with content after `- `.

## List Items

### list-continuation-indent

> One-line rationale: wrap list items at ~80 columns with 2-space continuation indent.

<!-- <BEFORE> -->
- Read [Getting Started](getting_started.md) for a complete example using these configurations
<!-- </BEFORE> -->

<!-- <AFTER> -->
- Read [Getting Started](getting_started.md) for a complete example using these
  configurations
<!-- </AFTER> -->

**Notes:** The markdown link stays intact on the first line. Continuation indented 2 spaces (aligned with content after `- `). For numbered lists (`1. `), use 3 spaces.

## Bold/Emphasis Headers

### bold-header-long-definition

> One-line rationale: rewrap a `**Term:** definition` paragraph to ~80 columns, keeping the bold term inline on the first line.

<!-- <BEFORE> -->
**Memoization**: `SXY` driver can cache routine synthesis if the same routine is invoked multiple times, such as a Poseidon implementation that uses routines for its permutation rounds to avoid recomputing it each time. The Routine trait marks locations in circuit code where drivers can identify structurally identical invocations. This allows drivers to memoize the polynomial construction, making subsequent circuit synthesis significantly faster when the same routine is called multiple times.
<!-- </BEFORE> -->

<!-- <AFTER> -->
**Memoization**: `SXY` driver can cache routine synthesis if the same routine is
invoked multiple times, such as a Poseidon implementation that uses routines for
its permutation rounds to avoid recomputing it each time. The Routine trait
marks locations in circuit code where drivers can identify structurally
identical invocations. This allows drivers to memoize the polynomial
construction, making subsequent circuit synthesis significantly faster when the
same routine is called multiple times.
<!-- </AFTER> -->

**Notes:** `**Memoization**:` stays at the start of the first line. Fill-wrap the rest as normal prose with no extra indent on continuation lines. Don't insert a line break between `**Term**:` and the first word of the definition.

## Mega-Paragraphs

### mega-multi-sentence

> One-line rationale: fill-wrap a long multi-sentence paragraph to ~80 columns without forcing breaks at sentence boundaries.

<!-- <BEFORE> -->
The **driver** abstraction provides a unified interface that enables the same circuit code to work across different execution contexts. A driver is a compile-time specialized backend interpreter that determines how circuit operations are executed at runtime.
<!-- </BEFORE> -->

<!-- <AFTER> -->
The **driver** abstraction provides a unified interface that enables the same
circuit code to work across different execution contexts. A driver is a
compile-time specialized backend interpreter that determines how circuit
operations are executed at runtime.
<!-- </AFTER> -->

**Notes:** Sentences flow together continuously — don't force a line break between sentences, just word-wrap at the column limit. Inline bold markup like `**driver**` stays intact.

### mega-bold-prefix

> One-line rationale: fill-wrap a bold-prefixed multi-sentence paragraph to ~80 columns.

<!-- <BEFORE> -->
**When to compress:** The key is to operate in uncompressed mode during recursion and only compress at specific boundary conditions. For example, when broadcasting a proof on-chain, you compress to optimize for bandwidth. During intermediate computation steps where you'll continue folding proofs together, keep them uncompressed.
<!-- </BEFORE> -->

<!-- <AFTER> -->
**When to compress:** The key is to operate in uncompressed mode during
recursion and only compress at specific boundary conditions. For example, when
broadcasting a proof on-chain, you compress to optimize for bandwidth. During
intermediate computation steps where you'll continue folding proofs together,
keep them uncompressed.
<!-- </AFTER> -->

**Notes:** The `**When to compress:**` bold prefix stays inline on the first line. Then wrap normally as with mega-multi-sentence. The bold prefix is structural (like a sub-heading within prose), not a term definition.

## Inherent Exceptions

These lines exceed 80 characters and **cannot** be fixed by rewrapping. They
are inherent exceptions — not violations. See "Inherent Exceptions" in
`formatting.md` for the general policy.

### link-irreducible-standalone

> One-line rationale: a long `[text](url)` already isolated on its own line is irreducible — no rewrap can shorten it.

<!-- <BEFORE> -->
... Developed for use with the [Pasta curves](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/) used in ...
<!-- </BEFORE> -->

<!-- <AFTER> -->
... Developed for use with the
[Pasta curves](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/)
used in ...
<!-- </AFTER> -->

**Notes:** The link is 84 characters on its own line. The surrounding prose has already been rewrapped to give the link its own line — no further action is possible. This is an inherent exception.

### link-irreducible-trailing-punctuation

> One-line rationale: a long link with trailing punctuation (`,` or `.`) already on its own line is irreducible.

<!-- <AFTER> -->
[LICENSE-APACHE](https://github.com/tachyon-zcash/ragu/blob/main/LICENSE-APACHE),
<!-- </AFTER> -->

**Notes:** 81 characters including the trailing comma. The comma is part of the sentence grammar and cannot be separated from the link. The link is already isolated — inherent exception.

### link-compound-reference

> One-line rationale: two links joined by `/` form an indivisible compound unit — tolerate the overlong line.

<!-- <AFTER> -->
[accumulation](https://eprint.iacr.org/2020/499)/[folding](https://eprint.iacr.org/2021/370)
<!-- </AFTER> -->

**Notes:** 92 characters. The `[A](url)/[B](url)` compound is a single semantic unit — splitting it across lines would break the visual relationship between the two links. Inherent exception.

### link-irreducible-nested-indent

> One-line rationale: a long link inside a nested list item is irreducible even though indentation adds to the width.

<!-- <AFTER> -->
      [Pasta curve cycle](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/)
<!-- </AFTER> -->

**Notes:** 95 characters (6-space indent + irreducible link). The list indentation is structurally required and the link cannot be shortened. Inherent exception.

### mega-mixed-markup

> One-line rationale: fill-wrap a paragraph containing bold definitions, inline links, and mid-paragraph emphasis, treating all markup except math and links as splittable.

<!-- <BEFORE> -->
**Proof-carrying data (PCD)** is a cryptographic primitive in which data
carries a proof of the entire computational history that produced it. In
traditional [verifiable computation], a [SNARK](../appendix/snarks.md) is
attached to a piece of data as a one-shot attestation that some computation
was performed correctly. PCD is qualitatively different: it is organized
around a **transition predicate** that must hold over old and new data. At
each step of a computation, one or more prior PCD instances are consumed and
a new instance is produced, certifying that the predicate was satisfied and
that the consumed instances were themselves valid — which, by induction,
establishes the validity of the entire preceding history. The result is not
a one-shot proof but incremental verification: the proof remains
constant-size regardless of how much computation it captures.
<!-- </BEFORE> -->

<!-- <AFTER> -->
**Proof-carrying data (PCD)** is a cryptographic primitive in which data carries
a proof of the entire computational history that produced it. In traditional
[verifiable computation], a [SNARK](../appendix/snarks.md) is attached to a
piece of data as a one-shot attestation that some computation was performed
correctly. PCD is qualitatively different: it is organized around a **transition
predicate** that must hold over old and new data. At each step of a computation,
one or more prior PCD instances are consumed and a new instance is produced,
certifying that the predicate was satisfied and that the consumed instances were
themselves valid — which, by induction, establishes the validity of the entire
preceding history. The result is not a one-shot proof but incremental
verification: the proof remains constant-size regardless of how much computation
it captures.
<!-- </AFTER> -->

**Notes:** Bold and emphasis markup (`**...**`, `*...*`) split freely across lines — they are ordinary prose during fill-wrap, unlike math and links. Here `**transition predicate**` breaks between lines 5–6. Inline links like `[SNARK](../appendix/snarks.md)` stay intact on one line (links are indivisible). A `**Term (ABBR)**` opening without a colon is treated the same as `**Term:**` — keep it inline on the first line and fill-wrap the rest.
