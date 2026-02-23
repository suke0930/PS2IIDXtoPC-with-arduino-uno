# PS2IIDXtoPC-with-arduino-uno
## 概要
PS2のIIDX向け専コン（エントリーモデル）をPCで利用するために、Arduino uno/Atmega328pを利用しキーボード入力に変換するためのコードです。

## 使い方
最初にArduino IDEに``PsxControllerBitBang.h``ライブラリをインポートします。
次に、arduno/sketch_dec16a.inoの記述のとおりにPS2 コントローラのメスアダプターの各端子を接続してください。
```
const byte PIN_PS2_ATT = 9;
const byte PIN_PS2_CMD = 6;
const byte PIN_PS2_DAT = 5;
const byte PIN_PS2_CLK = 8;
```

これ以外にGNDと3.3Vを接続する部分があります。詳しくは調べてみてください。

次にarduinoに当該スケッチを書き込み後、index.jsをシリアル通信のポートを指定し実行します。
### NODEJSに詳しくない場合はRealeasesから実行ファイルをダウンロードし、node index.jsをexeのファイル名に置き換えてください

```node index.js -p COM10```

これでポートがオープンされた後にコントローラのボタン入力がキー入力に変換されます。

また、``-b 115200``のようにボーレートを指定することも可能です。


