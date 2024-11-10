#!/usr/bin/bash

PACKAGE_NAME="rccn_usr_ws"
PLATFORM="$1"
BRANCH="$2"

# Always start from a fresh ostree repo.
# We only push one commit at a time, the receiver deduplicates.
rm -rf repo

ostree init \
    --repo=repo \
    --mode=archive

# Create the desired filesystem structure
mkdir -p ./ostree_root/usr/lib/$PACKAGE_NAME/
cp -r ./install ./ostree_root/usr/lib/$PACKAGE_NAME/

# Add the build output files
ostree commit \
    --repo=repo \
    --branch="$PACKAGE_NAME/$PLATFORM/$BRANCH" \
    --tree=dir=./ostree_root

# Push to server
ostree-push \
    --repo repo \
    deploy@deploy.rccn.space:repos/raccoon-mission \
    -p 2233 \
    $REF
