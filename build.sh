#!/bin/bash

source ~/.rccnenv

~/.local/bin/uv pip install git+https://github.com/jdiez17/colcon-ros-cargo

colcon build \
    --event-handlers console_direct+ \
    --packages-up-to vacuum_test_node

