#!/bin/bash
set -euo pipefail

FILES=$(find . -type f \( -name "*.py" -o -name "*.rs" -o -name "*.ts" -o -name "*.svelte" -o -name "*.js" -o -name "*.tsx" -o -name "*.jsx" -o -name "*.go" -o -name "*.rb" -o -name "*.java" -o -name "*.c" -o -name "*.h" -o -name "*.toml" -o -name "*.yaml" -o -name "*.yml" -o -name "*.json" -o -name "*.md" \) ! -path "*/node_modules/*" ! -path "*/target/*" ! -path "*/.git/*" ! -path "*/dist/*" ! -path "*/.venv/*" ! -path "*/__pycache__/*" ! -path "*/vendor/*" | head -100)

PAYLOAD="# Repository: $REPO
# Branch: $BRANCH
# Commit: $COMMIT

## Directory Structure
$(find . -type d ! -path '*/node_modules/*' ! -path '*/target/*' ! -path '*/.git/*' ! -path '*/dist/*' ! -path '*/.venv/*' ! -path '*/__pycache__/*' ! -path '*/vendor/*' | head -50)

## Source Files
"

while IFS= read -r file; do
  size=$(wc -c < "$file" 2>/dev/null || echo 0)
  if [ "$size" -gt 0 ] && [ "$size" -lt 50000 ]; then
    PAYLOAD="$PAYLOAD
### $file
$(head -200 "$file")
"
  fi
done <<< "$FILES"

echo "$PAYLOAD" > /tmp/review-input.txt
echo "Input size: $(wc -c < /tmp/review-input.txt) bytes"

RESPONSE=$(curl -s https://crof.ai/v1/chat/completions \
  -H "Authorization: Bearer $CROFAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"openai/glm-5.1\",
    \"messages\": [
      {
        \"role\": \"system\",
        \"content\": \"You are an expert code reviewer. Perform a thorough codebase review covering: 1) Architecture and design patterns, 2) Security issues, 3) Code quality and maintainability, 4) Potential bugs, 5) Performance concerns, 6) Testing coverage gaps. Be specific with file paths and line numbers. Output as markdown with clear sections.\"
      },
      {
        \"role\": \"user\",
        \"content\": $(cat /tmp/review-input.txt | jq -Rs .)
      }
    ],
    \"max_tokens\": 8192
  }")

echo "## Codebase Review Results" >> "$GITHUB_STEP_SUMMARY"
echo "$RESPONSE" | jq -r '.choices[0].message.content // "Error: " + (.error.message // "Unknown error")' >> "$GITHUB_STEP_SUMMARY"
echo "Review complete. Check GITHUB_STEP_SUMMARY for results."