#!/usr/bin/env fish
# Load .env file into environment variables for fish shell
# Usage: source load-env.fish  (NOT ./load-env.fish)

if not test -f .env
    echo "No .env file found"
    return 1
end

set -l count 0
while read -l line
    # Skip empty lines and comments
    string match -q -r '^\s*#' -- $line; and continue
    string match -q -r '^\s*$' -- $line; and continue

    # Split on first '='
    set -l key (string split -m 1 '=' -- $line)[1]
    set -l value (string split -m 1 '=' -- $line)[2]

    # Strip surrounding quotes from value
    set value (string trim -- $value)
    set value (string replace -r '^"(.*)"$' '$1' -- $value)
    set value (string replace -r "^'(.*)'\$" '$1' -- $value)

    if test -n "$key"
        set -gx $key $value
        set count (math $count + 1)
    end
end < .env

echo "Loaded $count variables from .env"
