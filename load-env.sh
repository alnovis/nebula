#!/bin/bash
# Load .env file into environment variables
# Usage: source load-env.sh  (NOT ./load-env.sh)

if [ ! -f .env ]; then
    echo "No .env file found"
    return 1 2>/dev/null || exit 1
fi

count=0
while IFS= read -r line || [ -n "$line" ]; do
    # Skip comments and empty lines
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${line// }" ]] && continue

    # Export the variable
    export "$line"
    count=$((count + 1))
done < <(grep -v '^\s*#' .env | grep -v '^\s*$')

echo "Loaded $count variables from .env"
