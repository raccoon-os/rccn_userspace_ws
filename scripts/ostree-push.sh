#!/usr/bin/bash

PACKAGE_NAME="rccn_usr_ws"
PLATFORM="$1"
BRANCH="$2"

ostree init \
    --repo=repo \
    --mode=archive

# Create the desired filesystem structure
mkdir -p ./ostree_root/usr/lib/$PACKAGE_NAME/
cp -r ./install ./ostree_root/usr/lib/$PACKAGE_NAME/

# Add the build output files
REF=$(ostree commit \
    --repo=repo \
    --branch="rccn_usr_ws/$PLATFORM/$BRANCH" \
    --tree=dir=./ostree_root)

# Push to server
ostree-push \
    --repo repo \
    builder@localhost:/home/builder/meta-raccoon-mission/build/tmp/deploy/images/phyboard-pollux-imx8mp-3/ostree_repo \
    $REF
