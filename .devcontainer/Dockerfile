# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.202.5/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends mingw-w64

RUN rustup target add x86_64-pc-windows-gnu

RUN cargo install cargo-tarpaulin

RUN chown -R vscode /usr/local/cargo

RUN mkdir -p /workspaces/target
RUN chown -R vscode /workspaces/target
