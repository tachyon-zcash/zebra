---
name: book-fixme
description: Resolve deferred book issues from FIXME.md
user-invocable: true
---

# Book FIXME

Resolve deferred issues tracked in `book/FIXME.md`. The user's `$ARGUMENTS`
determine what to work on.

## Step 1: Read Deferred Issues

Read `book/FIXME.md` and parse the `###`-level entries under
`## Deferred Issues`. Build a list of all tracked issues with their headings
and descriptions.

## Step 2: Match Target Issues

| Arguments | What to do |
|-----------|-----------|
| *(empty)* | Present the full list of deferred issues to the user. Use AskUserQuestion to let them pick which to work on. |
| A specific FIXME heading or partial match | Fuzzy-match against `###` headings. Select matching entries. |
| A theme that spans multiple entries | Collect all entries whose headings or descriptions match the theme. |
| A description that includes the user's proposed solution | Match the issue AND extract the solution to use as the starting point. |

Fuzzy-match against both the `###` headings and the description paragraphs
beneath them. If the match is ambiguous (multiple plausible interpretations),
ask the user to clarify.

## Step 3: Assess Current State

For each target issue:

1. Read the referenced file(s) and surrounding context.
2. Check whether the issue has already been partially or fully addressed —
   the file may have changed since the FIXME was recorded.
3. Present the current state to the user: what the FIXME says, what the file
   currently looks like, and whether the issue still exists.

If an issue appears to already be resolved, say so — the user may just want
to clear the FIXME entry.

## Step 4: Propose Fixes

For each target issue that still needs work:

- **If the user provided a solution** in `$ARGUMENTS`, use it as the basis.
  Do not invent an alternative approach — the user knows the book better than
  the reviewer does.
- **If no solution was provided**, propose one based on the FIXME description
  and the current file content.

Present the plan to the user for approval before making any edits. Show what
will change in each file with enough context to evaluate the fix.

## Step 5: Validate Fixes

Before applying, validate the proposed fixes against review policies:

1. Use Glob to find all `.claude/book-review/*.md` files.
2. Read `.claude/book-review/standards.md` (master standards).
3. For EACH policy file (except `standards.md`), launch a `general-purpose`
   Task agent (model `sonnet`) with this prompt:

   > You are validating a set of proposed book edits against review policies.
   >
   > Read these files:
   > - `.claude/book-review/standards.md` (master standards)
   > - `.claude/book-review/{focus}.md` (your policy)
   >
   > Here are the proposed changes:
   > {numbered list of proposed changes with locations and suggested rewrites}
   >
   > For each proposed change, check whether applying it would **introduce** a
   > violation of any rule in your policy or the master standards. Only flag
   > real conflicts — do not restate rules that are already satisfied.
   >
   > For each conflict found:
   > - **Change #**: which proposed change
   > - **Rule violated**: quote the relevant policy text
   > - **Conflict**: explain specifically how the suggestion violates the rule
   > - **Resolution**: suggest how to fix the suggestion to comply
   >
   > If no proposed changes conflict with your policy, say so.

   Launch ALL agents in parallel.

4. If validators flag conflicts, adjust the proposed fixes accordingly and
   note the adjustments to the user.

## Step 6: Apply Fixes

For each approved fix:

1. Read the target file to get the current content.
2. Apply the change using the Edit tool.

## Step 7: Clear Resolved FIXME Entries

After applying fixes:

1. Read `book/FIXME.md`.
2. Remove the `###`-level entries for each resolved issue.
3. If all issues under `## Deferred Issues` are resolved, leave the section
   header with no entries beneath it.
4. Write the updated file.

## Step 8: Run QA Scripts

Run QA scripts on affected files:

- `python3 qa/book/line_width.py` — check line width compliance
- `python3 qa/book/broken_links.py` — check for broken internal links

If a script fails to run, note the failure but do not block.

## Step 9: Report

Tell the user:

1. **What was fixed** — summarize the changes made to each file
2. **What FIXME entries were cleared** — list the removed headings
3. **QA results** — any issues found by the QA scripts
4. **Remaining FIXMEs** — how many deferred issues are still tracked
