#!/usr/bin/env fish
# Load .env file into environment variables for fish shell
# Usage: source load-env.fish

for line in (cat .env | grep -v '^#' | grep -v '^$')
    set -l key (echo $line | cut -d '=' -f 1)
    set -l value (echo $line | cut -d '=' -f 2-)
    set -gx $key $value
end

echo "Environment variables loaded from .env"
