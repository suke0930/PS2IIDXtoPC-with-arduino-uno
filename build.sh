#!/bin/bash
# Windows向けビルドをDocker内で行うスクリプト

set -e

# イメージのビルド
docker build -t rust-win-builder .

# コンテナ内でビルドを実行し、現在のディレクトリをマウントして結果を受け取る
docker run --rm \
    -v "$(pwd)":/app \
    -v cargo-registry:/usr/local/cargo/registry \
    rust-win-builder cargo build --release --target x86_64-pc-windows-gnu

echo "ビルド完了: target/x86_64-pc-windows-gnu/release/ps2iidx_controller.exe に出力されました"
