# eijirs

英辞郎のテキストデータ（EDP形式）を高速に検索するためのCLIツールです。
内部データベースに `sled` を使用し、初回インポート後は高速な前方一致検索が可能です。

## インストール

```bash
cargo install --path .

```

## 使い方

### 1. データベースの初期化

初回は英辞郎のテキストデータ（例：`EIJI-1441.TXT`）を指定してインポートを行う必要があります。

```bash
eijirs --init /path/to/EIJI-XXX.TXT

```

*データベースは OS 標準のデータディレクトリ（例: `~/.local/share/eijirs/db`）に作成されます。*

### 2. 単語の検索

引数に検索したいキーワードを入力します。

```bash
eijirs apple

```

### 3. TUI

引数に何も指定しないとTUIが起動します。

```bash
eijirs

```
