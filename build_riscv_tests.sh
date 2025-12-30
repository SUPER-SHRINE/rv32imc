#!/bin/bash

# Dockerイメージのビルド
echo "Building Docker image..."
docker build -t rv32imc-riscv-tests -f Dockerfile.riscv-tests .

# バイナリの抽出
echo "Extracting binaries..."
mkdir -p tests/bin
docker run --rm rv32imc-riscv-tests | tar xf - -C tests/bin

echo "Done! Binaries are located in tests/bin/"
