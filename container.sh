#!/usr/bin/env bash
set -euo pipefail

# Load environment variables
source .devenv

# Parse command line arguments
PLATFORM="${RCCN_USR_DEV_PLATFORM}"  # Default from .devenv
while [[ $# -gt 0 ]]; do
    case $1 in
        --platform)
            PLATFORM="$2"
            shift 2
            ;;
        --platform=*)
            PLATFORM="${1#*=}"
            shift
            ;;
        *)
            break
            ;;
    esac
done

# Default values
USERNAME="rosdev"
WORKSPACE="/home/rosdev/ros2_ws"

# Check if we should build from Dockerfile or use existing image
if [ -n "${RCCN_USR_DEV_CONTAINER_FILE:-}" ]; then
    echo "Building container from $RCCN_USR_DEV_CONTAINER_FILE..."
    docker build -t local-dev-container -f "$RCCN_USR_DEV_CONTAINER_FILE" .
    CONTAINER_IMAGE="local-dev-container"
else
    CONTAINER_IMAGE="$RCCN_USR_DEV_CONTAINER_IMAGE"
fi

# Run the command in container
docker run --rm -it \
    --platform="${PLATFORM}" \
    --net=host \
    -v "$(pwd):$WORKSPACE" \
    -w "$WORKSPACE" \
    -u "$USERNAME" \
    "$CONTAINER_IMAGE" \
    "$@"
