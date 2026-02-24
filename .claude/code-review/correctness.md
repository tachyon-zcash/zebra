# Correctness Review Policy

## Scope

Review safety, correctness, and robustness in Rust source files. Focus on code
that could cause unsoundness, panics, or subtle bugs.

## Error Handling

- No `unwrap()` or `expect()` in library code unless the invariant is documented
  and truly impossible to violate. Prefer propagating errors or using `assert!`
  for internal invariants.
- No `unsafe` without a `// SAFETY:` comment explaining why it's sound. The
  comment should address each safety requirement of the unsafe operation.

## Numeric Safety

- Avoid `as` casts between numeric types; prefer `try_into()` or explicit
  conversion functions that handle overflow. `as` casts silently truncate,
  which is almost never what you want in cryptographic code.

## Cryptographic Code

- Cryptographic code must be constant-time where timing side-channels matter.
  Document when constant-time behavior is required and verified.

## Assertions

- Use `assert!`, not `debug_assert!`. If an assertion is cheap enough to keep,
  keep it always. If it's too expensive for production, it's too expensive for
  debugging — write a test instead.
- Assertions guard invariants that correct callers cannot violate through the
  API. An assertion should never fire unless there's a bug in *this* code, not
  the caller's.

## Invariant Design

- Design code so invariants can be checked cheaply. If protecting an invariant
  requires an expensive assertion, restructure the code so the invariant is
  inherent (e.g., types that make invalid states unrepresentable) or move the
  check to a test.
- Expensive assertions are a code smell — they suggest either the invariant
  isn't worth checking at runtime, or the data structure doesn't make the
  invariant easy to verify.
