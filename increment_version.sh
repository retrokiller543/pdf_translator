#!/bin/bash
git fetch --tags
tag=$(git describe --tags --abbrev=0)

major=$(echo $tag | cut -d. -f1 | cut -c2-)
echo "major = $major"
minor=$(echo $tag | cut -d. -f2)
echo "minor = $minor"
patch=$(echo $tag | cut -d. -f3)
echo "patch = $patch"

messages=$(git log $tag..HEAD --pretty=format:%s)

versioning_commits=$(echo "$messages" | grep -E "^(feat|major|fix|minor|chore|refactor|patch):")

echo $(git log --oneline -n 5)
echo $(git tag)


if [ -z "$versioning_commits" ]; then
  echo "No versioning commits detected. No version increment needed."
  exit 0
fi

major_count=$(echo "$messages" | grep -cE "^(feat|major):")
minor_count=$(echo "$messages" | grep -cE "^(fix|minor):")
patch_count=$(echo "$messages" | grep -cE "^(chore|refactor|patch):")

if [[ $major_count -ge $minor_count ]] && [[ $major_count -ge $patch_count ]]; then
  # Increment the major number
  echo "Incrementing major version"
  major=$((major + 1))
  minor=0
  patch=0
  echo "New version is $major.$minor.$patch"
elif [[ $minor_count -ge $major_count ]] && [[ $minor_count -ge $patch_count ]]; then
  # Increment the minor number
  echo "Incrementing minor version"
  minor=$((minor + 1))
  patch=0
  echo "New version is $major.$minor.$patch"
else
  # Increment the patch number
  echo "Incrementing patch version"
  patch=$((patch + 1))
  echo "New version is $major.$minor.$patch"
fi

version="$major.$minor.$patch"
echo "Current version is $tag"
echo "New version is $version"

# Update the version in Cargo.toml
echo "Updating version to $version"
echo "messages = $messages"
sed -i "s/^version = \".*\"/version = \"$version\"/" Cargo.toml

# Only try to set the GitHub Actions environment variable if running within GitHub Actions
if [ ! -z "$GITHUB_ENV" ]; then
  echo "NEW_VERSION=$version" >> $GITHUB_ENV
fi