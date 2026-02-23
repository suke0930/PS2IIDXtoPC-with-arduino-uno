#!/bin/bash
# Windows向けビルドをDocker内で行うスクリプト

set -e

# イメージのビルド
docker build -t rust-win-builder .

MODE="${1:-release}"

if [ "$MODE" = "check" ]; then
    echo "=== Linux向けチェックビルド ==="
    docker run --rm \
        -v "$(pwd)":/app \
        -v cargo-registry:/usr/local/cargo/registry \
        rust-win-builder cargo check 2>&1
    echo "チェック完了"
elif [ "$MODE" = "test" ]; then
    echo "=== テスト実行 ==="
    docker run --rm \
        -v "$(pwd)":/app \
        -v cargo-registry:/usr/local/cargo/registry \
        rust-win-builder cargo test 2>&1
    echo "テスト完了"
else
    echo "=== Windows向けリリースビルド ==="
    docker run --rm \
        -v "$(pwd)":/app \
        -v cargo-registry:/usr/local/cargo/registry \
        rust-win-builder cargo build --release --target x86_64-pc-windows-gnu
    echo "ビルド完了: target/x86_64-pc-windows-gnu/release/ps2iidx_controller.exe に出力されました"
fi
