# History & Context Review Policy

## Scope

Use git history to find context-dependent bugs, regressions, and patterns that
a static read of the code would miss.

## Procedure

1. Identify the files and functions being reviewed.
2. Run `git log --oneline -20 -- {file}` for each file to see recent evolution.
3. Run `git blame -L {start},{end} -- {file}` on the specific lines under review
   to understand when and why they were written.
4. If the reviewed code modifies or extends an existing function, check what the
   function looked like before to understand whether the change preserves
   original intent.

## What to Look For

- **Regressions**: Does the new code revert or contradict a previous deliberate
  fix? Check if a recent commit message explains *why* something was done a
  certain way.
- **Incomplete migrations**: If a pattern was recently changed elsewhere in the
  file (e.g., error handling style, naming convention), does the reviewed code
  follow the new pattern or accidentally use the old one?
- **Lost invariants**: If `git blame` shows that a safety check or assertion was
  added in a targeted fix, does the reviewed code preserve that check?
- **Copy-paste drift**: If the reviewed code is structurally similar to another
  block in the same file, check whether both blocks have been kept in sync with
  recent changes.

## What NOT to Look For

- General code quality unrelated to history context.
- Issues on lines that haven't been modified and aren't affected by the change.
- Style or formatting — other reviewers and automated checks handle that.
