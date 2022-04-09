FROM ubuntu:20.04

# Unminimize the system
RUN yes | unminimize

# Install system utilities and programming languages
# nano: Git merge editor
RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    sudo man curl unzip git git-lfs nano zsh apt-transport-https ca-certificates gnupg2 lsb-release \
    libssl-dev pkg-config language-pack-en dnsutils build-essential

# Create user
ARG USERNAME=user
# On Linux, if 1000 is not your user's ID, don't worry. VSCode will update this for you automatically.
# See `updateRemoteUserUID` on: https://code.visualstudio.com/docs/remote/devcontainerjson-reference
# See also: https://code.visualstudio.com/docs/remote/containers-advanced#_adding-a-nonroot-user-to-your-dev-container
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID $USERNAME && \
    useradd --uid $USER_UID --gid $USER_GID --shell /usr/bin/zsh -m $USERNAME && \
    echo "$USERNAME ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/$USERNAME && \
    chmod 0440 /etc/sudoers.d/$USERNAME

# Install Docker and Docker Compose
RUN curl https://raw.githubusercontent.com/microsoft/vscode-dev-containers/master/containers/docker-in-docker/.devcontainer/library-scripts/docker-in-docker-debian.sh \
    | bash -s -- true $USERNAME false
ENV DOCKER_BUILDKIT=1

# Setting the ENTRYPOINT to docker-init.sh will configure non-root access to 
# the Docker socket if "overrideCommand": false is set in devcontainer.json. 
# The script will also execute CMD if you need to alter startup behaviors.
ENTRYPOINT [ "/usr/local/share/docker-init.sh" ]
CMD [ "sleep", "infinity" ]

# Run as user
USER $USERNAME

# Install default ZSH theme
RUN cp /etc/zsh/newuser.zshrc.recommended /home/$USERNAME/.zshrc

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain 1.56 \
    # VSCode Rust plugin dependencies
    --component "rust-src,rust-analysis,rls"
ENV PATH=$PATH:/home/$USERNAME/.cargo/bin
RUN cargo install cargo-audit

# Install NVM
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | zsh
ENV NVM_DIR="/home/$USERNAME/.nvm"

# Install Node v16
RUN . "$NVM_DIR/nvm.sh" && nvm install 16
