#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo -e "${GREEN}Nebula Release Script${NC}"
echo "Current version: $CURRENT_VERSION"
echo ""

# Parse arguments
if [ -z "$1" ]; then
    echo "Usage: $0 <version> [--dry-run]"
    echo ""
    echo "Examples:"
    echo "  $0 0.2.20           # Release version 0.2.20"
    echo "  $0 patch            # Bump patch version (0.2.19 -> 0.2.20)"
    echo "  $0 minor            # Bump minor version (0.2.19 -> 0.3.0)"
    echo "  $0 major            # Bump major version (0.2.19 -> 1.0.0)"
    echo "  $0 0.2.20 --dry-run # Preview changes without applying"
    exit 1
fi

DRY_RUN=false
if [ "$2" = "--dry-run" ]; then
    DRY_RUN=true
    echo -e "${YELLOW}DRY RUN MODE - no changes will be made${NC}"
    echo ""
fi

# Calculate new version
VERSION=$1
if [ "$VERSION" = "patch" ]; then
    IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
    VERSION="$major.$minor.$((patch + 1))"
elif [ "$VERSION" = "minor" ]; then
    IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
    VERSION="$major.$((minor + 1)).0"
elif [ "$VERSION" = "major" ]; then
    IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
    VERSION="$((major + 1)).0.0"
fi

echo "New version: $VERSION"
echo ""

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Invalid version format. Use X.Y.Z${NC}"
    exit 1
fi

# Check for uncommitted changes
if [ "$DRY_RUN" = false ] && ! git diff --quiet; then
    echo -e "${RED}Error: You have uncommitted changes. Please commit or stash them first.${NC}"
    exit 1
fi

DATE=$(date +%Y-%m-%d)

echo "Steps to perform:"
echo "  1. Update version in Cargo.toml"
echo "  2. Update CHANGELOG.md"
echo "  3. Update RELEASE_NOTES.md"
echo "  4. Run cargo check"
echo "  5. Create git commit"
echo "  6. Create git tag v$VERSION"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}Dry run complete. Run without --dry-run to apply changes.${NC}"
    exit 0
fi

read -p "Continue? [y/N] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo -e "${GREEN}Step 1: Updating Cargo.toml${NC}"
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$VERSION\"/" Cargo.toml

echo -e "${GREEN}Step 2: Updating CHANGELOG.md${NC}"
# Add new version section after [Unreleased]
sed -i "/^## \[Unreleased\]$/a\\
\\
## [$VERSION] - $DATE\\
\\
### Added\\
\\
### Changed\\
\\
### Fixed\\
" CHANGELOG.md

# Update links at bottom
sed -i "s|\[Unreleased\]: \(.*\)/compare/v$CURRENT_VERSION...HEAD|[Unreleased]: \1/compare/v$VERSION...HEAD\n[$VERSION]: \1/compare/v$CURRENT_VERSION...v$VERSION|" CHANGELOG.md

echo -e "${GREEN}Step 3: Updating RELEASE_NOTES.md${NC}"
cat > RELEASE_NOTES.md << EOF
# Release Notes

## v$VERSION

**Release Date:** $DATE

### Highlights

<!-- Add release highlights here -->

### New Features

<!-- Add new features here -->

### Improvements

<!-- Add improvements here -->

### Bug Fixes

<!-- Add bug fixes here -->

### Migration

No breaking changes.

---

## Previous Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history.
EOF

echo -e "${GREEN}Step 4: Running cargo check${NC}"
cargo check

echo -e "${GREEN}Step 5: Creating git commit${NC}"
git add Cargo.toml Cargo.lock CHANGELOG.md RELEASE_NOTES.md
git commit -m "chore: release v$VERSION"

echo -e "${GREEN}Step 6: Creating git tag${NC}"
git tag -a "v$VERSION" -m "Release v$VERSION"

echo ""
echo -e "${GREEN}Release v$VERSION prepared successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. Edit CHANGELOG.md and RELEASE_NOTES.md with actual changes"
echo "  2. Amend the commit: git commit --amend"
echo "  3. Push to remote: git push && git push --tags"
echo ""
echo "Or to undo:"
echo "  git reset --soft HEAD~1 && git tag -d v$VERSION"
