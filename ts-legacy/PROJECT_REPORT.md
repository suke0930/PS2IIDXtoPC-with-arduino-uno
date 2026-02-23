# PS2IIDXtoPC-with-arduino-uno 調査レポート / 概要ドキュメント

このリポジトリは、PS2用IIDX専用コントローラ（PS2コントローラ互換）からの入力をArduinoで読み取り、シリアル経由でPC側に送信し、PC側でキーボード入力または仮想ゲームパッド入力に変換するための一連のコード群です。

## 1. 全体構成（調査結果）
- Arduino側（`arduino/sketch_dec16a/sketch_dec16a.ino`）
  - PS2コントローラのボタン状態を読み取り、変化があったときにシリアル送信。
  - 送信フォーマットは `b:<id>:<state>` 形式（例: `b:14:1`）。
  - ターンテーブル位置用に `t:<position>` を送る関数は存在するが、現状 `scr_pos` を更新していないため実質未使用。
- PC側（Node.js）
  - `index.js`: 旧実装/ビルド済みJavaScript。Nut.jsでキーボード入力に変換。
  - `indexV2.ts`: TypeScript版。キーマッピングが変更され、遅延（offset）オプションが追加。
  - `popen.ts`: `indexV2.ts`とほぼ同じロジックで別のキーマッピング。
  - `pad.js`: ViGEmを使って仮想Xbox 360コントローラとして入力を出力。
- README（`README.md`）
  - 使用方法の説明があるが、一部文字化けが見られる。

## 2. シリアル通信仕様（Arduino -> PC）
- ボタンイベント: `b:<id>:<state>`
  - `<id>`: 0-15 のボタンID
  - `<state>`: `1` = 押下, `0` = 解放
- ターンテーブルイベント: `t:<position>`
  - 送信関数はあるが、PC側で未処理。

## 3. PC側の変換方式（用途別）
### キーボード入力（Nut.js）
- `index.js`
  - ボタンIDを `A,S,D,W,R,T,F,G,B,N,M,J,U,K,P,L` に変換。
  - 特定キー押下時の例外処理（ignoreフラグ）がある。
- `indexV2.ts`
  - ボタンIDを `F13〜F24` や `RightShift/RightControl` などに変換。
  - `-o, --offset` で送信遅延を指定可能。
  - 方向入力（F13/F15）に対する特殊処理（ignoreフラグ）あり。
- `popen.ts`
  - `indexV2.ts`と同等のロジックだが、キーマッピングが `Q/W/A/S` 等に変更されている。

### 仮想ゲームパッド（ViGEm）
- `pad.js`
  - ViGEmBus経由で仮想Xbox 360コントローラを生成。
  - D-Padは軸（dpadHorz/dpadVert）として扱う。
  - L2/R2（ID 8/9）はトリガー入力として別処理。
  - それ以外はボタンマッピングに従って押下/解放。

## 4. 主要ファイル一覧と役割
- `arduino/sketch_dec16a/sketch_dec16a.ino`: PS2コントローラ読み取りとシリアル送信。
- `index.js`: 旧実装（Nut.jsキーボード変換、JS）。
- `indexV2.ts`: TypeScript版キーボード変換。
- `popen.ts`: 代替キーマッピング版。
- `pad.js`: ViGEmによる仮想ゲームパッド出力。
- `package.json`: 依存関係（serialport, nut-js, vigemclient など）。
- `tsconfig.json`: TypeScript設定。

## 5. 実行方法（例）
Arduino側:
1. Arduino IDEに `PsxControllerBitBang.h` を導入。
2. `arduino/sketch_dec16a/sketch_dec16a.ino` を書き込み。
3. 配線は `PIN_PS2_ATT = 9`, `PIN_PS2_CMD = 6`, `PIN_PS2_DAT = 5`, `PIN_PS2_CLK = 8`。

PC側（キーボード出力）:
```powershell
node index.js -p COM10
node indexV2.ts -p COM10 -b 115200 -o 0
```

PC側（仮想パッド出力）:
```powershell
node pad.js -p COM10 -b 115200
```

## 6. 依存と前提
- `serialport`: Arduinoからのシリアル通信。
- `@nut-tree-fork/nut-js`: キーボード入力のエミュレーション。
- `vigemclient`: 仮想Xbox 360コントローラ出力（Windows + ViGEmBusが必要）。

## 7. 既知の未実装/注意点
- ターンテーブル（`t:`）はArduino側で送信関数があるが、PC側で未処理。
- `index.js`はビルド済みJSで、`indexV2.ts`/`popen.ts`とはマッピングや挙動が異なる。
- READMEの文字化けがあるため、今後整理が必要。

## 8. 次の整理候補（提案）
- ターンテーブル入力の仕様を決め、PC側で `t:` を処理する実装を追加。
- どのPC側実装を採用するか（キーボード or 仮想パッド）を明確化。
- READMEを最新構成に合わせて整理し、日本語の文字化けを修正。
