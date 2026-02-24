# Math Notation Rules

These rules apply to all mathematical content in the project — book chapters,
rustdoc, module docs, and any other technical writing. Context-specific policies
layer additional rules on top of these.

## KaTeX for All Math

- Use KaTeX (`$...$`) for all mathematical expressions — not backticks or raw
  Unicode. Backticks are for code identifiers; Unicode subscripts and operators
  render inconsistently across platforms. KaTeX provides proper typesetting and
  is searchable.
- Write `$\sum_j c_j \cdot Y^j$` not `Σⱼ cⱼ · Yʲ`.

## LaTeX Conventions

- Use LaTeX conventions within KaTeX: `\cdot` for multiplication, `\sum` for
  summation, `^{}` for superscripts. Avoid ASCII approximations like `*` for
  multiply.
- Standard spacing in math: `$f(x, y)$` not `$f(x,y)$`. Function arguments,
  tuple elements, and parameter lists should have spaces after commas.

## Symbol Consistency

- Variables and symbols must mean the same thing throughout a document. Flag
  reuse of a symbol for a different meaning without explicit redefinition.
- Flag notation used before it is defined.
- Flag notation that conflicts with standard usage in the relevant literature
  (cryptography, algebraic geometry) without explicit redefinition.
