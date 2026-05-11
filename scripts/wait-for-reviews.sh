#!/usr/bin/env bash
set -euo pipefail

branch="${GITHUB_HEAD_REF:-$(git branch --show-current)}"
if [ -z "$branch" ]; then
    echo "Cannot determine branch name (detached HEAD). Set GITHUB_HEAD_REF." >&2
    exit 1
fi
repo=$(git remote get-url origin 2>/dev/null | sed -E 's|git@github.com:||;s|https://github.com/||;s|\.git$||')
pr=$(gh pr list --repo "$repo" --head "$branch" --state open --json number -q '.[0].number' 2>/dev/null || echo "")

if [ -z "$pr" ]; then
    echo "No open PR found for branch $branch"
    exit 1
fi

current_head=$(git rev-parse HEAD)

echo "Waiting for new reviews on PR #$pr for commit $current_head..."

seen_coderabbit=false
seen_gemini=false

max_wait_seconds="${MAX_WAIT_SECONDS:-1800}"
deadline_epoch=$(( $(date +%s) + max_wait_seconds ))
api_failures=0

while true; do
    # Fetch reviews for the specific commit
    if ! reviews=$(gh api "repos/$repo/pulls/$pr/reviews" \
      -q ".[] | select(.commit_id == \"$current_head\") | .user.login" 2>/dev/null); then
        api_failures=$((api_failures + 1))
        if [ "$api_failures" -ge 3 ]; then
            echo -e "\nFailed to fetch PR reviews after 3 attempts" >&2
            exit 2
        fi
        sleep 30
        continue
    fi
    api_failures=0
    
    if echo "$reviews" | grep -q -E "coderabbitai"; then
        seen_coderabbit=true
    fi
    if echo "$reviews" | grep -q -E "gemini"; then
        seen_gemini=true
    fi
    
    # Copilot does not reliably review every push (it typically runs only on PR open),
    # so we do not block on it. We only strictly require the two main bots.
    if [ "$seen_coderabbit" = true ] && [ "$seen_gemini" = true ]; then
        echo -e "\nCodeRabbit and Gemini reviews detected for the current commit!"
        exit 0
    fi
    
    echo -n "."
    if [ "$(date +%s)" -ge "$deadline_epoch" ]; then
        echo -e "\nTimed out waiting for bot reviews" >&2
        exit 2
    fi
    sleep 30
done
