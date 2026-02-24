# Sync all subtrees from upstream
sync project="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{project}}" ]; then
        git subtree pull --prefix={{project}} {{project}} main --squash
    else
        for p in tachyon ragu zebra librustzcash zcash-devtool; do
            echo "==> Syncing $p..."
            git subtree pull --prefix=$p $p main --squash
        done
    fi

# Push a subtree to a fork remote for PRing
# Usage: just push <prefix> <fork-remote> <branch>
push prefix fork branch:
    git subtree push --prefix={{prefix}} {{fork}} {{branch}}
