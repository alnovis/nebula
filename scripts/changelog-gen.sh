#!/bin/bash
# Generate changelog entries from git commits since last tag

# Get last tag
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null)

if [ -z "$LAST_TAG" ]; then
    echo "No tags found. Showing all commits."
    RANGE="HEAD"
else
    echo "Changes since $LAST_TAG:"
    echo ""
    RANGE="$LAST_TAG..HEAD"
fi

# Categorize commits by conventional commit type
echo "### Added"
git log $RANGE --oneline --grep="^feat" --format="- %s" | sed 's/^- feat[:(]/- /' | sed 's/):/:/g'

echo ""
echo "### Changed"
git log $RANGE --oneline --grep="^refactor\|^perf\|^style" --format="- %s" | sed 's/^- \(refactor\|perf\|style\)[:(]/- /' | sed 's/):/:/g'

echo ""
echo "### Fixed"
git log $RANGE --oneline --grep="^fix" --format="- %s" | sed 's/^- fix[:(]/- /' | sed 's/):/:/g'

echo ""
echo "### Other"
git log $RANGE --oneline --invert-grep --grep="^feat\|^fix\|^refactor\|^perf\|^style\|^chore\|^docs\|^test\|^ci" --format="- %s" 2>/dev/null | head -10

echo ""
echo "---"
echo "Total commits: $(git rev-list --count $RANGE)"
