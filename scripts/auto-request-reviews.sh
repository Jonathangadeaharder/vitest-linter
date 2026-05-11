#!/usr/bin/env bash
set -euo pipefail

STATE_DIR=".git/auto-review-state"

log() { echo "[auto-review] $*"; }
die() { echo "[auto-review] ERROR: $*" >&2; exit 1; }

require_cmds() {
    for cmd in git gh python3; do
        command -v "$cmd" &>/dev/null || die "Required command not found: $cmd"
    done
}

get_current_branch() { git branch --show-current; }

get_repo() {
    git remote get-url origin 2>/dev/null \
        | sed -E 's|git@github.com:||;s|https://github.com/||;s|\.git$||'
}

find_open_pr() {
    local branch="$1" repo="$2"
    gh pr list --repo "$repo" --head "$branch" --state open --json number -q '.[0].number' 2>/dev/null || echo ""
}

get_state() {
    local branch="$1"
    local safe_branch="${branch//\//_}"
    local state_file="$STATE_DIR/${safe_branch}.state"
    if [ -f "$state_file" ]; then cat "$state_file"; else echo ""; fi
}

save_state() {
    local branch="$1"
    local head_sha="$2"
    local ts
    ts=$(python3 -c "from datetime import datetime,timezone;print(datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'))")
    local safe_branch="${branch//\//_}"
    mkdir -p "$STATE_DIR"
    echo "${head_sha}|${ts}" > "$STATE_DIR/${safe_branch}.state"
}

fetch_and_check_reviews() {
    local repo="$1" pr="$2" since="$3"
    local tmpdir
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" RETURN

    gh api --paginate "repos/$repo/pulls/$pr/reviews?per_page=100" > "$tmpdir/reviews.json" 2>/dev/null || echo "[]" > "$tmpdir/reviews.json"
    gh api --paginate "repos/$repo/pulls/$pr/comments?per_page=100" > "$tmpdir/comments.json" 2>/dev/null || echo "[]" > "$tmpdir/comments.json"

    python3 "$tmpdir/../check.py" "$since" "$tmpdir/reviews.json" "$tmpdir/comments.json" 2>/dev/null || \
    python3 - "$since" "$tmpdir/reviews.json" "$tmpdir/comments.json" << 'PYEOF'
import json, sys

since = sys.argv[1][:19]
with open(sys.argv[2]) as f:
    reviews = json.load(f)
with open(sys.argv[3]) as f:
    comments = json.load(f)

major_markers = [
    "high-priority",
    "potential issue",
    "🟠 major",
    "⚠️ potential issue",
    "![high]",
    "critical",
]

major_found = []
for r in reviews:
    ts = (r.get("submitted_at") or "")[:19]
    if ts < since:
        continue
    user = r.get("user", {}).get("login", "")
    if "github-actions" in user or "renovate" in user:
        continue
    body = (r.get("body") or "").lower()
    state = r.get("state", "")
    for marker in major_markers:
        if marker in body:
            major_found.append({"user": user, "marker": marker.strip(), "type": "review"})
            break

for c in comments:
    ts = (c.get("created_at") or "")[:19]
    if ts < since:
        continue
    user = c.get("user", {}).get("login", "")
    if "github-actions" in user or "renovate" in user:
        continue
    body = (c.get("body") or "").lower()
    for marker in major_markers:
        if marker in body:
            major_found.append({"user": user, "marker": marker.strip(), "type": "comment", "path": c.get("path", "")})
            break

print(f"ITEMS:{len(reviews)+len(comments)}")
if major_found:
    for m in major_found:
        print(f"MAJOR:{m['user']}|{m['marker']}|{m.get('path','')}")
else:
    print("NONE")
PYEOF
}

request_reviews() {
    local repo="$1" pr="$2"
    log "Requesting reviews on PR #$pr..."
    gh pr comment "$pr" --repo "$repo" --body "/gemini review" 2>/dev/null && log "  Requested Gemini review"
    gh pr comment "$pr" --repo "$repo" --body "@coderabbitai full review" 2>/dev/null && log "  Requested CodeRabbit full review"
    log "  Copilot auto-triggers on push (no manual request needed)"
}

main() {
    require_cmds

    local branch repo pr state current_head saved_head saved_ts

    branch=$(get_current_branch)
    [ -z "$branch" ] && die "Not on a branch"
    [[ "$branch" == "main" || "$branch" == "master" ]] && { log "Skipping main/master branch"; exit 0; }

    repo=$(get_repo)
    [ -z "$repo" ] && die "Could not detect repo from remote"

    pr=$(find_open_pr "$branch" "$repo")
    [ -z "$pr" ] && { log "No open PR for branch $branch"; exit 0; }
    log "Found PR #$pr for branch $branch"

    state=$(get_state "$branch")
    current_head=$(gh pr view "$pr" --repo "$repo" --json headRefOid -q '.headRefOid' 2>/dev/null)
    [ -z "$current_head" ] && current_head=$(git rev-parse HEAD)

    if [ -z "$state" ]; then
        log "First run on this branch — tracking state for next push"
        save_state "$branch" "$current_head"
        exit 0
    fi

    saved_head=$(echo "$state" | cut -d'|' -f1)
    saved_ts=$(echo "$state" | cut -d'|' -f2)

    if [ "$saved_head" == "$current_head" ]; then
        log "No new commits since last run. Skipping review requests."
        exit 0
    fi

    log "New commits detected. Checking for major findings since $saved_ts..."
    local result
    result=$(fetch_and_check_reviews "$repo" "$pr" "$saved_ts")

    local item_count
    item_count=$(echo "$result" | grep "^ITEMS:" | head -1 | cut -d: -f2)
    log "Found ${item_count:-?} review items since last push"

    if echo "$result" | grep -q "^NONE$"; then
        save_state "$branch" "$current_head"
        log "No major findings detected — skipping review requests"
        exit 0
    fi

    local majors
    majors=$(echo "$result" | grep "^MAJOR:" | sed 's/^MAJOR://')
    log "Major findings detected:"
    echo "$majors" | while IFS= read -r line; do
        log "  - $line"
    done

    request_reviews "$repo" "$pr"
    save_state "$branch" "$current_head"
}

main "$@"
