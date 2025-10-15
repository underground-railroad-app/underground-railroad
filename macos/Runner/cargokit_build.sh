#!/bin/bash
set -e

# Get the project root (3 levels up from this script)
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

# Run cargokit build
"${PROJECT_ROOT}/cargokit/build_pod.sh"
