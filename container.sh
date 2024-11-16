#!/usr/bin/env bash
set -euo pipefail
set -x

# Load environment variables
source .devenv

# Parse command line arguments
PLATFORM="${RCCN_USR_DEV_PLATFORM}"  # Default from .devenv
IMAGE="${RCCN_USR_DEV_CONTAINER_IMAGE}"
CONTAINER_ENGINE="docker"
while [[ $# -gt 0 ]]; do
    case $1 in
        --platform=*)
            PLATFORM="${1#*=}"
            shift
            ;;
        --image=*)
            IMAGE="${1#*=}"
            shift
            ;;
        --engine=*)
            CONTAINER_ENGINE="${1#*=}"
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
    $CONTAINER_ENGINE build -t local-dev-container -f "$RCCN_USR_DEV_CONTAINER_FILE" .
    IMAGE="local-dev-container"
fi

# Check if we're running interactively
if [ -t 0 ]; then
    INTERACTIVE_FLAGS="-it"
else
    INTERACTIVE_FLAGS=""
fi

# Container-engine-specific arguments
if [[ "$CONTAINER_ENGINE" == "podman" ]]; then
    CONTAINER_ENGINE_ARGS = "--userns=keep-id"
fi

# Run the command in container
$CONTAINER_ENGINE run --rm $INTERACTIVE_FLAGS \
    --platform="$PLATFORM" \
    --net=host \
    -v "$(pwd):$WORKSPACE" \
    -w "$WORKSPACE" \
    -u "$USERNAME" \
    $CONTAINER_ENGINE_ARGS \
    "$IMAGE" \
    bash -c "$*"
