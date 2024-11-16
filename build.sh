#!/bin/bash

source ~/.rccnenv

~/.local/bin/uv pip install git+https://github.com/jdiez17/colcon-ros-cargo

export IDL_PACKAGE_FILTER="std_msgs;rccn_usr_msgs;thermal_test_msgs"

colcon build \
    --event-handlers console_direct+ \
    --packages-up-to rccn_usr_example_app
