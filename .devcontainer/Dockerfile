FROM docker.io/ros:humble-ros-core

ARG USERNAME=rosdev
ARG UID=1000
ARG GID=1000

# Install some dependencies packages
RUN apt update -q \
    && apt upgrade -q -y \
    && apt install -y --no-install-recommends \
    software-properties-common \
    python3-pip \
    git \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN apt-get update && apt-get install -y \
    ros-humble-tf2 ros-humble-rosbag2 \
    ros-humble-tf2-msgs-dbgsym ros-humble-rosbag2-interfaces-dbgsym && \
    rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y \
    curl libclang-dev build-essential sudo git && \
    rm -rf /var/lib/apt/lists/*

# Create and switch to user
RUN groupadd -g $GID $USERNAME \
    && useradd -lm -u $UID -g $USERNAME -s /bin/bash $USERNAME \
    && echo "$USERNAME ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers
USER $USERNAME

# Install colcon to user-local pip
RUN pip install git+https://github.com/colcon/colcon-cargo.git
RUN pip install git+https://github.com/colcon/colcon-ros-cargo.git
RUN pip install colcon-common-extensions

RUN mkdir -p /home/$USERNAME

# Create workspace so that user own this directory
RUN mkdir -p /home/$USERNAME/ros2_ws/src
WORKDIR /home/$USERNAME/ros2_ws

# Install rust
RUN curl https://sh.rustup.rs -sSf | (bash -s -- -y)

# Install cargo-ament-build
ENV PATH="/home/$USERNAME/.cargo/bin:${PATH}"
RUN cargo install cargo-ament-build

# TODO: install cargo bitbake
RUN echo 'source /opt/ros/'$ROS_DISTRO'/setup.bash' >> /home/$USERNAME/.bashrc \
    && echo 'source /home/'$USERNAME'/ros2_ws/install/setup.bash' >> /home/$USERNAME/.bashrc \
    && echo 'source /home/'$USERNAME'/.cargo/env' >> /home/$USERNAME/.bashrc

# Setup entrypoint
COPY ./ros_entrypoint.sh /
ENTRYPOINT ["/ros_entrypoint.sh"]
CMD ["bash"]
