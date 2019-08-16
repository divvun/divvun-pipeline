FROM debian:stretch

RUN apt update
RUN apt install -y wget curl gnupg

RUN wget -q -O - https://apertium.projectjj.com/apt/install-nightly.sh | bash

# uptodate clang
RUN curl https://apt.llvm.org/llvm-snapshot.gpg.key -sSf | apt-key add -
RUN echo "deb http://apt.llvm.org/stretch/ llvm-toolchain-stretch-7 main" >> /etc/apt/sources.list.d/llvm.list
RUN apt update
RUN apt install -y clang-7 clang-7-doc libclang-common-7-dev libclang-7-dev libclang1-7 clang-format-7

RUN apt install -y libhfst-dev capnproto build-essential
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

# WORKDIR /opt
# ADD . .