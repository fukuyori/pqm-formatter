# pqm-formatter

Microsoft Excel Power Query および Power BI で使用される **Power Query M** 言語のコードフォーマッターです。

[English README](README.md)

## 特徴

- **自動インデント**: 一貫した4スペースインデント
- **インテリジェントな改行**: 式の複雑さと設定可能な行長に基づく改行
- **コメント保持**: 行コメント（`//`）とブロックコメント（`/* */`）を保持
- **複数のフォーマットモード**: デフォルト、コンパクト、展開
- **キーワードのフィールド名使用**: `type`、`error`、`each` などの予約語をレコードフィールド名として使用可能
- **Unicode対応**: 日本語の変数名など非ASCII識別子を完全サポート
- **クリップボード連携**: Windows、macOS、Linux でクリップボードから直接フォーマット

## インストール

### ソースからビルド

```bash
git clone https://github.com/fukuyori/pqm-formatter.git
cd pqm-formatter
cargo build --release
```

実行ファイルは `target/release/pqmfmt`（Windows では `pqmfmt.exe`）に生成されます。

### ビルド済みバイナリ

[Releases](https://github.com/fukuyori/pqm-formatter/releases) ページからダウンロードできます。

## 使い方

### コマンドライン

```bash
# ファイルをフォーマットして標準出力に出力
pqmfmt input.pq

# ファイルをフォーマットして上書き保存
pqmfmt -w input.pq

# フォーマット結果を別ファイルに出力
pqmfmt -o output.pq input.pq

# 標準入力から読み込み
cat input.pq | pqmfmt --stdin

# フォーマット済みかチェック（未フォーマットなら終了コード1）
pqmfmt -c input.pq

# コンパクトモードを使用
pqmfmt --compact input.pq

# 展開モードを使用
pqmfmt --expanded input.pq

# インデントサイズを指定
pqmfmt --indent 2 input.pq
```

### クリップボードモード（デフォルト）

引数なしで実行すると、クリップボードからコードを読み取り、フォーマットして結果をクリップボードに書き戻します：

```bash
# Power Query M コードをクリップボードにコピーしてから実行:
pqmfmt

# フォーマット済みコードがクリップボードに入ります
```

### ライブラリとして使用

```rust
use pqm_formatter::{format, Config};

let code = r#"let x=1,y=2,z=x+y in z"#;

// デフォルトフォーマット
let formatted = format(code, Config::default()).unwrap();
println!("{}", formatted);

// コンパクトフォーマット
let compact = format(code, Config::compact()).unwrap();
println!("{}", compact);
```

## フォーマットモード

### デフォルトモード

適度な改行と4スペースインデントによる標準的なフォーマット。

**入力:**
```m
let ソース=Excel.CurrentWorkbook(){[Name="テーブル1"]}[Content],変更された型=Table.TransformColumnTypes(ソース,{{"地区",type text},{"売上",Int64.Type}}) in 変更された型
```

**出力:**
```m
let
    ソース = Excel.CurrentWorkbook(){[Name = "テーブル1"]}[Content],
    変更された型 = 
        Table.TransformColumnTypes(
            ソース,
            {
                {"地区", type text},
                {"売上", Int64.Type}
            }
        )
in
    変更された型
```

### コンパクトモード (`--compact`)

改行を最小化。行長制限内であれば単純な式は1行にまとめます。

**出力:**
```m
let ソース = Excel.CurrentWorkbook(){[Name = "テーブル1"]}[Content], 変更された型 = Table.TransformColumnTypes(ソース, {{"地区", type text}, {"売上", Int64.Type}}) in 変更された型
```

### 展開モード (`--expanded`)

すべてのリスト、レコード、関数呼び出しを展開して可読性を最大化。

**出力:**
```m
let
    ソース = Excel.CurrentWorkbook(){[Name = "テーブル1"]}[Content],
    変更された型 = 
        Table.TransformColumnTypes(
            ソース,
            {
                {
                    "地区",
                    type text
                },
                {
                    "売上",
                    Int64.Type
                }
            }
        )
in
    変更された型
```

## オプション

| オプション | 説明 |
|--------|-------------|
| `-c, --check` | フォーマット済みかチェック（未フォーマットなら終了コード1） |
| `-w, --write` | フォーマット結果を入力ファイルに上書き |
| `-o, --output FILE` | 指定したファイルに出力 |
| `--stdin` | 標準入力から読み込み |
| `--compact` | コンパクトモードを使用 |
| `--expanded` | 展開モードを使用 |
| `--indent SIZE` | インデントサイズを指定（デフォルト: 4） |
| `--tabs` | スペースの代わりにタブを使用 |
| `-h, --help` | ヘルプを表示 |
| `-V, --version` | バージョンを表示 |

## 対応構文

- let 式
- if-then-else 式
- try-catch 式（`otherwise` 対応）
- 関数定義と呼び出し
- レコードとリスト
- フィールドアクセス（`[field]`）とアイテムアクセス（`{index}`）
- 型注釈（`as type`）
- 型式（`type table [Column = text]`）
- 二項演算子と単項演算子
- each 式
- section 式
- メタデータ（`meta`）
- すべての Power Query M キーワードをフィールド名として使用可能

## エラー処理

構文エラーが発生した場合、行と列の情報を含むエラーメッセージを表示します：

```
Error in input.pq:
Line 5: Expected identifier, found RightParen
```

クリップボードモードでは、エラーメッセージが元のコードにコメントとして追加されます。

## 統合

### Visual Studio Code

`.vscode/tasks.json` にタスクを作成：

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Format Power Query M",
            "type": "shell",
            "command": "pqmfmt",
            "args": ["-w", "${file}"],
            "problemMatcher": []
        }
    ]
}
```

### エディタショートカット (Windows)

AutoHotkey などを使用して、クリップボードフォーマット用のキーボードショートカットを設定できます。

## ソースからのビルド

必要要件:
- Rust 1.70 以降

```bash
# デバッグビルド
cargo build

# リリースビルド
cargo build --release

# テスト実行
cargo test
```

## ライセンス

MIT License

## コントリビューション

Issue や Pull Request は歓迎します！

## 変更履歴

### v0.4.0

- キーワードをレコードフィールド名として使用可能に（`type`、`error` など）
- シンプルな要素（数値、文字列、型）を含むリストのフォーマット改善
- 入れ子関数のフォーマット修正
- クリップボードモードで関数式を受け入れるよう修正
- Windows での Unicode（UTF-8）クリップボード対応
- コンパクトモードの動作改善
