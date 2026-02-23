# PS2IIDXtoPC-with-arduino-uno

## 概要
PS2のIIDX向け専コン（エントリーモデル）をPCで利用するために、Arduino uno/Atmega328pを利用しキーボード入力に変換するためのソフトウェアです。
従来のNode.js実装から**Rustによる極小レイテンシのWindowsネイティブ実装**へとアップグレードされました。

## アーキテクチャ構成
- **Arduino側**: PS2コントローラーからBitBangで入力を読み取り、シリアル通信(`115200bps`)でPCへ送信します。
  - 送信フォーマット例: `b:14:1` (ボタン14が押された), `b:14:0` (ボタン14が離された)
- **PC側 (Rust)**: シリアル通信を監視し、送られてきた状態をOSのネイティブ仮想キーボードAPI（WindowsInput等）を用いてエミュレートします。

---

## 使い方 (Windows向け)

### 1. Arduinoのセットアップ
最初にArduino IDEに `PsxControllerBitBang.h` などのライブラリをインポートします。
次に、`arduino/sketch_dec16a.ino` の記述のとおりにPS2 コントローラのメスアダプターの各端子を接続してください。
```cpp
const byte PIN_PS2_ATT = 9;
const byte PIN_PS2_CMD = 6;
const byte PIN_PS2_DAT = 5;
const byte PIN_PS2_CLK = 8;
```
これ以外にGNDと3.3Vを接続する部分があります。
Arduinoにスケッチを書き込んでください。

### 2. PC側ソフトの実行
Releasesなどから `ps2iidx_controller.exe` をダウンロードするか、後述の手順でビルドしてください。

コマンドプロンプトまたはPowerShellを開き、Arduinoが接続されているCOMポート（例: `COM10`）を指定して実行します。

```bat
ps2iidx_controller.exe -p COM10
```
またはボーレートを指定する場合:
```bat
ps2iidx_controller.exe -p COM10 -b 115200
```
これでポートがオープンされた後にコントローラの入力がPCのキー入力に爆速で変換されます。

---

## ビルド方法 (開発者向け)
このリポジトリには、ホスト環境を汚さずにRustのWindows向けクロスコンパイルを行うためのDocker環境が含まれています。

**要件:** Dockerがインストールされていること(Linux/WSL推奨)

```bash
# ビルドスクリプトを実行するだけです
./build.sh
```

ビルドが完了すると `target/x86_64-pc-windows-gnu/release/ps2iidx_controller.exe` が生成されます。
