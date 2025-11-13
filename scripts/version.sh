#!/bin/bash
set -e

VERSION=""
DRY_RUN=false

show_usage() {
    echo "Usage: $0 <version> [--dry-run]"
    echo ""
    echo "Update version across Rust CLI and TypeScript SDK"
    echo ""
    echo "Arguments:"
    echo "  <version>    Version number in semver format (e.g., 0.2.0)"
    echo "  --dry-run    Show what would be changed without making changes"
    echo ""
    echo "Example:"
    echo "  $0 0.2.0"
    echo "  $0 0.2.0 --dry-run"
}

validate_semver() {
    local version=$1
    if ! [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        echo "Error: Invalid semver format: $version"
        echo "Expected format: X.Y.Z or X.Y.Z-prerelease"
        exit 1
    fi
}

check_tag_exists() {
    local version=$1
    if git rev-parse "v$version" >/dev/null 2>&1; then
        echo "Error: Tag v$version already exists"
        exit 1
    fi
}

update_cargo_toml() {
    local version=$1
    local file="Cargo.toml"
    
    if [ ! -f "$file" ]; then
        echo "Error: $file not found"
        exit 1
    fi
    
    if [ "$DRY_RUN" = true ]; then
        echo "[DRY RUN] Would update $file: version = \"$version\""
    else
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/^version = \".*\"/version = \"$version\"/" "$file"
        else
            sed -i "s/^version = \".*\"/version = \"$version\"/" "$file"
        fi
        echo "✅ Updated $file"
    fi
}

update_package_json() {
    local version=$1
    local file="packages/client/package.json"
    
    if [ ! -f "$file" ]; then
        echo "Error: $file not found"
        exit 1
    fi
    
    if [ "$DRY_RUN" = true ]; then
        echo "[DRY RUN] Would update $file: \"version\": \"$version\""
    else
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"version\": \".*\"/\"version\": \"$version\"/" "$file"
        else
            sed -i "s/\"version\": \".*\"/\"version\": \"$version\"/" "$file"
        fi
        echo "✅ Updated $file"
    fi
}

create_git_tag() {
    local version=$1
    
    if [ "$DRY_RUN" = true ]; then
        echo "[DRY RUN] Would create git tag: v$version"
    else
        git tag "v$version"
        echo "✅ Created git tag v$version"
    fi
}

show_next_steps() {
    local version=$1
    
    echo ""
    echo "Version updated to $version"
    echo ""
    echo "Next steps:"
    echo "  1. Review the changes:"
    echo "     git diff"
    echo ""
    echo "  2. Commit the changes:"
    echo "     git commit -am 'Bump version to $version'"
    echo ""
    echo "  3. Push the commit and tag:"
    echo "     git push && git push --tags"
    echo ""
    echo "  4. Create a release on GitHub (if applicable)"
    echo "     https://github.com/plyght/birch/releases/new"
}

main() {
    if [ $# -lt 1 ]; then
        show_usage
        exit 1
    fi
    
    VERSION=$1
    shift
    
    while [ $# -gt 0 ]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                echo "Error: Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    echo "Birch Version Update Script"
    echo "============================"
    echo ""
    
    if [ "$DRY_RUN" = true ]; then
        echo "DRY RUN MODE - No changes will be made"
        echo ""
    fi
    
    validate_semver "$VERSION"
    
    if [ "$DRY_RUN" = false ]; then
        check_tag_exists "$VERSION"
    fi
    
    echo "Updating version to: $VERSION"
    echo ""
    
    update_cargo_toml "$VERSION"
    update_package_json "$VERSION"
    
    if [ "$DRY_RUN" = false ]; then
        create_git_tag "$VERSION"
        show_next_steps "$VERSION"
    else
        echo ""
        echo "[DRY RUN] No changes were made"
        echo "Run without --dry-run to apply changes"
    fi
}

main "$@"

