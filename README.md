# tachyon-zcash workspace

Monorepo workspace for coordinating development across tachyon-zcash projects using git subtrees.

## Included projects

| Directory | Upstream |
|-----------|----------|
| `tachyon/` | github.com/tachyon-zcash/tachyon |
| `ragu/` | github.com/tachyon-zcash/ragu |
| `zebra/` | github.com/tachyon-zcash/zebra |
| `librustzcash/` | github.com/tachyon-zcash/librustzcash |
| `zcash-devtool/` | github.com/tachyon-zcash/zcash-devtool |

## Syncing from upstream

```sh
just sync              # all projects
just sync tachyon      # one project
```

## Making changes and opening PRs

Make changes anywhere in the workspace and commit normally. When ready to open a PR, first add your fork as a local remote (one-time setup):

```sh
git remote add my-ragu git@github.com:YOURNAME/ragu.git
```

Then push the relevant subtree to your fork and open a PR:

```sh
just push ragu my-ragu my-feature-branch
# Open PR: YOURNAME/ragu:my-feature-branch → tachyon-zcash/ragu:main
```

The `git subtree push` command filters workspace commits down to only those touching the given prefix, rewrites paths, and pushes — resulting in a normal-looking PR with individual commits.

