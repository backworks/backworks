#!/bin/bash
# Release script for Backworks

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Check if version is provided
if [[ $# -eq 0 ]]; then
    echo -e "${RED}❌ Usage: $0 <version>${NC}"
    echo -e "${BLUE}   Example: $0 v1.0.0${NC}"
    exit 1
fi

VERSION=$1

# Ensure version starts with 'v'
if [[ ! $VERSION =~ ^v ]]; then
    VERSION="v$VERSION"
fi

echo -e "${BLUE}🚀 Preparing release ${VERSION}...${NC}"

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "${YELLOW}⚠️  Warning: Not on main branch (currently on: $CURRENT_BRANCH)${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    echo -e "${RED}❌ Working directory is not clean. Please commit or stash changes.${NC}"
    git status --short
    exit 1
fi

# Update Cargo.toml version
echo -e "${BLUE}📝 Updating version in Cargo.toml...${NC}"
CLEAN_VERSION=${VERSION#v}  # Remove 'v' prefix for Cargo.toml
sed -i.bak "s/^version = .*/version = \"$CLEAN_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update lock file
cargo update

# Run tests one more time
echo -e "${BLUE}🧪 Running final tests...${NC}"
cargo test

# Build release
echo -e "${BLUE}🔨 Building release...${NC}"
cargo build --release

# Create changelog entry if CHANGELOG.md exists
if [[ -f "CHANGELOG.md" ]]; then
    echo -e "${BLUE}📋 Please update CHANGELOG.md for version ${VERSION}${NC}"
    echo -e "${YELLOW}   Press any key when ready to continue...${NC}"
    read -n 1 -s
fi

# Commit version bump
echo -e "${BLUE}📦 Committing version bump...${NC}"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to ${VERSION}"

# Create and push tag
echo -e "${BLUE}🏷️  Creating tag ${VERSION}...${NC}"
git tag -a "$VERSION" -m "Release $VERSION"

echo -e "${BLUE}📤 Pushing to origin...${NC}"
git push origin main
git push origin "$VERSION"

echo -e "${GREEN}🎉 Release ${VERSION} has been created!${NC}"
echo -e "${BLUE}📋 Next steps:${NC}"
echo -e "${BLUE}   1. Check GitHub Actions for automated builds${NC}"
echo -e "${BLUE}   2. Monitor the release workflow${NC}"
echo -e "${BLUE}   3. Update documentation if needed${NC}"
echo -e "${BLUE}   4. Announce the release${NC}"

# Show release URL
REPO_URL=$(git config --get remote.origin.url | sed 's/\.git$//')
if [[ $REPO_URL =~ github.com ]]; then
    RELEASE_URL="${REPO_URL/git@github.com:/https://github.com/}/releases/tag/$VERSION"
    echo -e "${BLUE}🔗 Release URL: ${RELEASE_URL}${NC}"
fi
