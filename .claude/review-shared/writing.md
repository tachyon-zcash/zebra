# Writing Rules

These rules apply to all prose in the project — book chapters, rustdoc, module
docs, and any other written content. Context-specific policies (book-review,
code-review) layer additional rules on top of these.

## Voice and Tone

- Prefer active voice, but accept passive in mathematical definitions and
  protocol descriptions where the agent is irrelevant. "The polynomial is
  evaluated at $x$" is fine; "We can see that the polynomial is evaluated" is
  not — the passive is correct, but the hedging ("we can see") is not.
- Maintain a direct, confident tone. Avoid hedging ("perhaps", "it might be")
  unless genuine uncertainty is being communicated.

## Weasel Words

- Avoid "simply", "just", "obviously", "clearly". If something is obvious, it
  doesn't need a comment. If it isn't obvious, these words are dishonest.
- "Note that" is almost always filler. Delete it and the sentence usually
  improves.

## Sentence Structure

- Vary sentence length. A long explanatory sentence should be followed by a
  short, punchy one. Monotonous rhythm puts readers to sleep.
- Avoid long parenthetical asides mid-sentence. Use a separate sentence instead.
  If the aside is important enough to include, it deserves its own sentence.
- Technical terms should be introduced before use. Flag forward references to
  undefined terms.

## Word Repetition

- Avoid repeating the same word within a sentence. Rephrase with a synonym or
  restructure. Example: "Developed for use with the Pasta curves used in Zcash"
  → "Developed for the Pasta curves employed in Zcash".
- Within a paragraph, vary word choice when natural alternatives exist (e.g.,
  "verify" / "check" / "confirm"; "construct" / "build" / "create").
- **Exempt**: technical terms, proper nouns, acronyms, and domain-specific
  vocabulary. Terminological consistency takes precedence over variety — do not
  replace a defined term with a synonym for the sake of variety.

## Punctuation Density

- Watch for em dash overuse at the page/module level. Em dashes are effective
  for asides and interjections, but overuse makes the writing monotonous and
  the dashes lose their punch.
- When flagging density, identify the least impactful usages and suggest
  rephrasing so the dash becomes unnecessary. Leave the strongest usages
  intact — a page with 2–3 well-placed em dashes reads better than a page
  with zero.
- Preferred alternatives: commas, parentheticals, semicolons, subordinate
  clauses, or restructuring the sentence to eliminate the aside entirely.

## Terminology

- Once a term is chosen for a concept, use it consistently throughout. Flag
  synonyms meaning the same thing within a document or module.
- Defined terms (from a glossary or terminology appendix) take precedence.

## Capitalization

- Lowercase for technical descriptive phrases: "proof-carrying data", not
  "Proof-Carrying Data". Capitalize only the first word at sentence start:
  "Proof-carrying data is...".
- Proper nouns (Halo, Zcash, Pasta, Poseidon) and acronyms (SNARK, PCD,
  ECDLP) stay capitalized.
- Flag title-cased descriptive phrases that aren't proper nouns.
