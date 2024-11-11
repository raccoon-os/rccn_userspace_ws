#!/bin/bash

set -x

source /opt/raccoon/0.4/environment-setup-cortexa53-crypto-poky-linux

export ROS_VERSION=2
export ROS_PYTHON_VERSION=3
export ROS_DISTRO=${ROS_DISTRO:-humble}
export PYTHONPATH=$OECORE_NATIVE_SYSROOT/usr/lib/python3.12/site-packages:$OECORE_TARGET_SYSROOT/usr/opt/ros/$ROS_DISTRO/lib/python3.12/site-packages
export AMENT_PREFIX_PATH="$OECORE_TARGET_SYSROOT/usr/opt/ros/$ROS_DISTRO:$OECORE_TARGET_SYSROOT/usr/opt/ros/$ROS_DISTRO/share:$OECORE_TARGET_SYSROOT/usr/opt/ros/$ROS_DISTRO/lib/cmake"


export PWD="$(pwd)"

# Architecture-specific args
# Not sure why we have to pass this, but we get the following error otherwise:
#    Generate bindings in '/home/rosdev/ros2_ws/target/debug/build/r2r_msg_gen-x/out/y'
#    /opt/raccoon/0.4/sysroots/cortexa53-crypto-poky-linux/usr/include/bits/timesize.h:23:10: fatal error: 'bits/timesize-32.h' file not found
export ARCH_FLAGS="-D__WORDSIZE=64 -D__LP64__=1 -D__aarch64__"

# Rust-specific
export IDL_PACKAGE_FILTER="std_msgs;rccn_usr_msgs;thermal_test_msgs"
export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$OECORE_TARGET_SYSROOT $ARCH_FLAGS"

colcon build \
    --merge-install \
    --install-base "$PWD/install" \
    --cmake-args \
    " -DCMAKE_TOOLCHAIN_FILE=$CMAKE_TOOLCHAIN_FILE" \
    " -DCMAKE_STAGING_PREFIX=$PWD/install" \
    " -DBUILD_TESTING=OFF" \
    " -DPython3_NumPy_INCLUDE_DIR=$OECORE_TARGET_SYSROOT/usr/lib/python3.12/site-packages/numpy/core/include" \
    " $ARCH_FLAGS"

# Now we compile the Rust nodes.
# First, "pseudo-source-install-setup.bash.
export AMENT_PREFIX_PATH="$PWD/install:$AMENT_PREFIX_PATH"

# TODO this does not work currently.
# We get a linker error when trying to build the build script for r2r (https://github.com/sequenceplanner/r2r/blob/master/r2r/build.rs).
# This seems to happen because the build script has to be built for the build-machine architecture instead of the target (makes sense).

#cargo build --target aarch64-poky-linux-gnu