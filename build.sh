#!/bin/bash

source ~/.rccnenv

colcon build \
    --event-handlers console_direct+ \
    --packages-up-to vacuum_test_node \
