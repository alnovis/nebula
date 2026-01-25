#!/bin/bash
# Load .env file into environment variables
# Usage: source load-env.sh

set -a  # automatically export all variables
source .env
set +a

echo "Environment variables loaded from .env"
