org := "tachyon-zcash"
projects := "tachyon ragu zebra librustzcash zcash-devtool"

_default:
    @just --list

# Push a project to its fork remote, for the current branch
[arg('project', pattern='tachyon|ragu|zebra|librustzcash|zcash-devtool')]
push project *args:
    git subtree push --prefix={{ project }} fork/{{ project }} $(git branch --show-current) {{ args }}

# Pull a project from its fork remote, for the current branch
[arg('project', pattern='tachyon|ragu|zebra|librustzcash|zcash-devtool')]
pull project *args:
    git subtree pull --squash --gpg-sign --prefix={{ project }} fork/{{ project }} $(git branch --show-current) {{ args }}

# Pull all projects from fork remotes, for the current branch
all-pull *args:
    for p in {{ projects }} ; do git subtree pull --squash --gpg-sign --prefix=$p fork/$p $(git branch --show-current) {{ args }} ; done

# Push all projects to fork remotes for the current branch
all-push *args:
    for p in {{ projects }} ; do git subtree push --prefix=$p fork/$p $(git branch --show-current) {{ args }} ; done

# Pull a project from its org remote, optionally specifying a rev.
# Use this after a project PR merges to the org repo.
[arg('project', pattern='tachyon|ragu|zebra|librustzcash|zcash-devtool')]
land project rev="main" *args:
    git subtree pull --squash --gpg-sign --prefix={{ project }} {{ org }}/{{ project }} {{ rev }} {{ args }}

# Open a PR on the org project for the current branch (you should push first)
[arg('project', pattern='tachyon|ragu|zebra|librustzcash|zcash-devtool')]
pr project *args:
    #!/usr/bin/env sh
    set -ue
    pr_source="$(gh api user --jq .login):$(git branch --show-current)"
    read -p " $(tput bold)PR {{ project }} $pr_source to {{ org }}/{{ project }}?$(tput sgr0) [y/N] " yesno
    expr "$yesno" : '^[yY]$' > /dev/null
    gh pr create -R {{ org }}/{{ project }} --head $pr_source {{ args }}

# Add remotes for all projects
subtree-setup:
    #!/usr/bin/env sh
    set -ue
    gh_user=$(gh api user --jq .login)
    reset -Q
    echo " Github user "$(tput bold)$gh_user$(tput sgr0)" may fork {{ org }} projects:" {{ projects }}
    echo " If you have a colliding name, you probably want to do this manually."
    read -p " $(tput bold)OK TO CREATE GITHUB REPOS?$(tput sgr0) [y/N] " yesno
    expr "$yesno" : '^[yY]$' > /dev/null
    set -x
    [ $(gh config get git_protocol) = "ssh" ] && url_field="sshUrl" || url_field="url"
    for p in {{ projects }}; do
        gh repo fork "{{ org }}/$p" --clone=false || true # don't fail if fork already exists
        upstream_url=$(gh repo view "{{ org }}/$p" --json "$url_field" --jq ".$url_field")
        git remote add "{{ org }}/$p" "$upstream_url"
        fork_url=$(gh repo view "$gh_user/$p" --json "$url_field" --jq ".$url_field")
        git remote add "fork/$p" "$fork_url"
    done
