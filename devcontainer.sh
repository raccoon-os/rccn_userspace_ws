#!/bin/bash
set -e

echo $1

BASE_IMAGE_NAME="rccn-usr-base"
DEV_IMAGE_NAME="rccn-usr-dev"
CONTAINER_NAME="rccn-usr-devcontainer"

WS_MOUNT="/home/rosdev/ros2_ws/"

# Build the base image first
echo "Building base image..."
docker build -t $BASE_IMAGE_NAME -f Dockerfile.base .

if [ $1 == "--build-dev" ]; then
    echo "Building development container..."
    docker build -t $DEV_IMAGE_NAME -f Dockerfile.dev .
    exit 0
fi

# Run the devcontainer
echo "Starting development container..."
docker run -it --rm \
    --name $CONTAINER_NAME \
    -v "$(pwd)":$WS_MOUNT \
    -w $WS_MOUNT \
    --net=host \
    $DEV_IMAGE_NAME
    "$@"
