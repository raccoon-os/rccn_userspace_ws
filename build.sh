#!/usr/bin/env bash
set -euo pipefail

# Execute colcon build in container
./container.sh bash -c ". ~/.cargo/env && \
    . /opt/ros/humble/setup.sh && \
    . /home/rosdev/.venv/bin/activate && \
    colcon build \
        --event-handlers console_direct+ \
        --packages-up-to vacuum_test_node \
        --cargo-args --verbose"
