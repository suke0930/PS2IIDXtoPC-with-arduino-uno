# 改良計画（ドラフト）

この計画は、現状の動作を維持しつつ、設定の外部化と構成整理を進めるためのロードマップです。

## 前提（現状の挙動）
- ターンテーブルは `t:` を使わず、`b:` の入力として扱う。
- 現在の割り当てではターンテーブル相当が `F13` / `F15` に対応している。
- Arduino側は `t:` を送信しない（`b:3` / `b:6` で出ている扱い）。
- この挙動は当面変更しない。

## 目標
- マッピングとモードをJSON化して切り替え可能にする。
- IIDX / Popn などのコードを統合し、共通入出力層を設計する。
- pad.js をTypeScript化し、同様にJSON設定を利用する。
- CUIランチャー＋環境変数で起動を簡単にする。

## ステップ案
1. JSON設計
   - `mapping/` にボタン→キー/ボタンのマッピングJSONを置く。
   - 例: `mapping/iidx.keyboard.json`, `mapping/popn.keyboard.json`, `mapping/x360.pad.json`
2. 共通入出力層の分離
   - 入力: `serial -> event`（`b:id:state` のみ対応）
   - 出力: `keyboard` / `x360` を差し替え可能にする
3. index.ts / popen.ts を統合
   - `--mode` または `--map` で切り替える
4. pad.js をTypeScriptへ移行
   - 既存の挙動を維持したまま移植
5. pad.jsのマッピングをJSON化
   - キーボードと同じ構成で `x360` 向けマッピングを作成
6. CUIランチャー作成
   - モード、ボーレート、COMポートを対話選択
   - ランチャーなしでもCLI起動できるように `--mode/--map` を維持
7. env導入
   - `DEFAULT_PORT`, `DEFAULT_BAUD`, `DEFAULT_MODE` などを想定

## 追加で入れておくと良いこと
- JSONのスキーマ検証（必須項目やID範囲チェック）
- `--debug` で入力イベントと出力先のログを切り替え表示
- 依存分離（keyboard系/ViGEm系）でexe化時のサイズと依存性を軽くする

