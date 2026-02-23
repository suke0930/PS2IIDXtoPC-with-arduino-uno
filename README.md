# PS2IIDXtoPC-with-arduino-uno

## 概要

PS2のIIDX向け専コン（エントリーモデル）をPCで利用するために、Arduino Uno/Atmega328pを利用してキーボード入力またはXbox 360コントローラー入力に変換するためのプログラムです。

**主な機能:**
- PS2コントローラー入力をキーボードまたは仮想Xbox 360コントローラーに変換
- IIDX、ポップン、Xbox 360向けプリセットマッピング対応
- カスタムマッピング設定のJSON形式サポート
- インタラクティブランチャーによる簡単なポート/モード選択
- 環境変数（.env）による設定の永続化

**技術スタック:**
- **ハードウェア:** Arduino Uno/Atmega328p + PsxControllerBitBang ライブラリ
- **PC側:** Rust (stable edition 2021)
- **プラットフォーム:** キーボードモード（Windows/Linux/macOS）、X360モード（Windows専用）

## セットアップ

### 1. Arduino の準備

1. Arduino IDEに `PsxControllerBitBang` ライブラリをインポート
2. PS2コントローラーのメスアダプターの各端子を以下のように接続:

```
const byte PIN_PS2_ATT = 9;
const byte PIN_PS2_CMD = 6;
const byte PIN_PS2_DAT = 5;
const byte PIN_PS2_CLK = 8;
```

3. GNDと3.3Vも接続してください
4. `arduino/sketch_dec16a/sketch_dec16a.ino` をArduinoに書き込み

### 2. PC側のセットアップ

```bash
# Rustのインストール（未インストールの場合）
# https://www.rust-lang.org/tools/install

# リポジトリのクローン
git clone https://github.com/suke0930/PS2IIDXtoPC-with-arduino-uno.git
cd PS2IIDXtoPC-with-arduino-uno

# ビルド
cargo build --release

# 環境変数の設定（オプション）
cp .env.example .env
# .envファイルを編集してデフォルト値を設定
```

### 3. Xbox 360モードを使う場合（Windows専用）

Xbox 360コントローラーエミュレーションを使用する場合は、[ViGEmBus ドライバー](https://github.com/ViGEm/ViGEmBus)のインストールが必要です。

## 使い方

### インタラクティブランチャー（推奨）

初めて使う場合や、ポートが分からない場合はこちらが便利です:

```bash
cargo run -- --launcher
```

対話形式でシリアルポート、ボーレート、マッピングモードを選択できます。

### 直接実行

ポートとモードを指定して直接実行:

```bash
# IIDX モード（キーボード）
cargo run -- -p COM10 -m iidx

# ポップン モード（キーボード）
cargo run -- -p COM10 -m popn

# Xbox 360 モード（Windows専用）
cargo run -- -p COM10 -m x360

# カスタムマッピング
cargo run -- -p COM10 --map ./custom-mapping.json

# 入力遅延の調整（ミリ秒）
cargo run -- -p COM10 -m iidx --offset 10

# デバッグモード
cargo run -- -p COM10 -m iidx --debug
```

### リリースビルドの実行

ビルド済みバイナリを直接実行:

```bash
./target/release/ps2iidx_controller -p COM10 -m iidx
```

### 環境変数による設定

`.env` ファイルでデフォルト値を設定できます:

```env
DEFAULT_PORT=COM10
DEFAULT_BAUD=115200
DEFAULT_MODE=iidx
DEFAULT_OFFSET=0
DEFAULT_DEBUG=0
```

設定後は引数なしで実行可能:

```bash
cargo run
```

## コマンドラインオプション

```
-p, --port <port>       シリアルポート（例: COM10）
-b, --baud <rate>       ボーレート（デフォルト: 115200）
-m, --mode <mode>       マッピングモード（iidx, popn, x360）
--map <path>            カスタムマッピングJSONファイルのパス
-o, --offset <time>     入力遅延（ミリ秒）
-d, --debug             デバッグログを有効化
--launcher              インタラクティブランチャーを起動
```

## マッピング設定

### プリセットマッピング

- `iidx` - beatmania IIDX向けキーボードマッピング（`mapping/iidx.keyboard.json`）
- `popn` - ポップンミュージック向けキーボードマッピング（`mapping/popn.keyboard.json`）
- `x360` - Xbox 360コントローラーマッピング（`mapping/x360.pad.json`）

### カスタムマッピングの作成

`mapping/` ディレクトリにJSONファイルを作成することで、独自のマッピングを定義できます。

**キーボードマッピングの例:**

```json
{
  "name": "custom.keyboard",
  "output": "keyboard",
  "buttons": {
    "0": { "key": "F21" },
    "1": { "key": "RightShift" }
  },
  "special": {
    "ignoreKey": "F14",
    "tapKeys": ["F13", "F15"],
    "tapDurationMs": 13
  }
}
```

**Xbox 360マッピングの例:**

```json
{
  "name": "custom.x360",
  "output": "x360",
  "buttons": {
    "0": { "type": "button", "name": "A" },
    "1": { "type": "dpad", "direction": "up" },
    "8": { "type": "trigger", "trigger": "left" }
  }
}
```

## トラブルシューティング

### ポートが開けない

- デバイスマネージャーでCOMポート番号を確認
- 他のアプリケーションがポートを使用していないか確認
- Arduino IDEのシリアルモニタが開いていないか確認

### 入力が反応しない

- デバッグモードで動作確認: `--debug` オプションを追加
- Arduinoのシリアルモニタで `b:` メッセージが送信されているか確認
- ボーレートが正しいか確認（デフォルト: 115200）

### Xbox 360モードが動かない

- ViGEmBus ドライバーがインストールされているか確認
- Windowsでのみ動作します（他のOSではキーボードモードを使用）

## 開発

### ビルド

```bash
cargo build              # デバッグビルド
cargo build --release    # リリースビルド
```

### テスト

```bash
cargo test
```

### 新しいマッピングモードの追加

1. `mapping/` ディレクトリにJSONファイルを作成
2. `src/mapping.rs` の `DEFAULT_MAPS` に追加:
   ```rust
   "custom" => Some("mapping/custom.keyboard.json".to_string()),
   ```

## 技術詳細

プロジェクトのアーキテクチャや開発ガイドラインについては、[CLAUDE.md](./CLAUDE.md) を参照してください。

## ライセンス

このプロジェクトはMITライセンスの下で公開されています。

## 補足

- ターンテーブルは現在ボタンイベント（`b:3` / `b:6`）として処理されます
- TypeScript/Node.js実装は `ts-legacy/` に移動されました（参考用）


