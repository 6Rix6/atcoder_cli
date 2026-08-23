# atcoder_cli

[English](./README-en.md) | 日本語

AtCoder向けのCLIヘルパー。問題文の取得、サンプルケースの実行、ログインセッションの管理をコマンドラインから行えます。

## インストール

Rust 1.85以上(edition 2024)が必要です。

```sh
cargo build --release
# バイナリ: target/release/acc
```

## サブコマンド

| コマンド | エイリアス | 説明 |
|---|---|---|
| `acc test <TASK_ID>` | `t` | サンプルケースを実行し AC/WA/TLE/RE を判定 |
| `acc auth login` | `a` | ブラウザからコピーした `REVEL_SESSION` クッキーを保存 |
| `acc auth logout` | | クッキーファイルを削除 |
| `acc auth status` | | 保存済みクッキーの有効性を検証 |
| `acc init` | | 対話形式で言語を選び `atcoder.toml` を生成 |
| `acc setup <CONTEST_ID>` | `s` | コンテスト用ディレクトリを自動生成(問題ごとの雛形込み) |
| `acc download <TASK_ID>` | `dl` | 問題文をMarkdownとして標準出力へ取得(`-o` でファイル保存) |

`<TASK_ID>` には `abc350_a` のようなタスクIDか、問題ページURL全体を指定できます。
`<CONTEST_ID>` には `abc350` のようなコンテストIDか、コンテストURL全体を指定できます。

## 使い方の流れ

```sh
# 1. 初回のみ: 設定生成とログイン
acc init
acc auth login   # 表示手順に従い REVEL_SESSION を貼り付け

# 2. コンテスト用ディレクトリの作成(問題ごとに a/ b/ ... を一括生成)
acc setup abc350

# 3. 問題文の取得
acc dl abc350_a > problem.md
# または -o で直接保存
acc dl abc350_a -o problem.md

# 4. 解答を作成しサンプルテスト(atcoder.tomlのある場所から実行)
acc t abc350_a
```

## 設定ファイル(atcoder.toml)

カレントディレクトリの `atcoder.toml` を優先し、無ければグローバル設定(`~/.config/atcoder-cli/atcoder.toml`)を使用します。

```toml
compiler = "g++"                       # 省略可(コンパイル不要言語は省く)
compiler_flags = ["${TASK_ID}.cpp", "-O2", "-o", "${TASK_ID}"]
run = "./${TASK_ID}"
run_flags = []                         # 省略可
```

コマンドとフラグ内では次のテンプレート変数が利用できます。

| 変数 | 展開値 | 例(`abc350_a`) |
|---|---|---|
| `${CONTEST_ID}` | コンテストID | `abc350` |
| `${TASK_ID}` | タスクID | `abc350_a` |
| `${TASK_CHAR}` | 問題文字 | `a` |

## コンテストディレクトリのセットアップ(acc setup)

`acc setup <CONTEST_ID>` は問題一覧を取得し、各問題のソースファイル雛形を生成します。`atcoder.toml` のコピーは行わないため、`atcoder.toml` を置いたディレクトリ(作業ルート)から移動せずに使う想定です。生成パスをテンプレート変数で参照する設定例:

```toml
compiler = "g++"
compiler_flags = ["${CONTEST_ID}/${TASK_CHAR}/main.cpp", "-O2", "-o", "${TASK_ID}"]
run = "./${TASK_ID}"

[setup]
dir = "${TASK_CHAR}"
file = ["main.cpp"]
```

```sh
acc setup abc350          # カレントディレクトリ直下に作成
acc setup abc350 -d ~/cp  # 出力先を指定
acc setup abc473 --md     # 問題文Markdownも同時取得
```

- 開催前のコンテストではタスク一覧ページが存在しないため、トップページの配点表から問題文字を取得します(この場合 `--md` は取得できません)
- **既存ファイルは上書きしません**。再実行すると未作成の問題だけ追加されます

ディレクトリ構成は `[setup]` テーブルでカスタマイズできます。

```toml
[setup]
dir = "${TASK_CHAR}"                    # 各問題ディレクトリのパス(コンテストルート相対)
file = ["main.cpp"]                     # 生成するファイル(問題ディレクトリ相対・複数可)

[setup.template]                        # 雛形を拡張子キーで直接指定(省略可)
cpp = '''
#include <bits/stdc++.h>
using namespace std;

int main() {

}
'''
```

`dir` / `file` にはテンプレート変数(`${CONTEST_ID}` / `${TASK_ID}` / `${TASK_CHAR}`)が使えます。`parent` は先頭の `~` をホームディレクトリに展開します。雛形は `[setup.template]` の拡張子キー → 標準ファイル名のプリセット雛形(`main.cpp` 等) → 空ファイル の順で決まります。

## テスト実行の挙動

- 判定は **AC / WA / TLE / RE**。改行や行頭行末の空白差異は許容して比較します
- 実行時間制限は問題ページから自動取得(取得できない場合は2秒)
- [GNU time](https://www.gnu.org/software/time/)(`/usr/bin/time` または `gtime`)があればメモリ使用量も表示します

## 終了コード

| コード | 意味 |
|---|---|
| `0` | 成功(全サンプル合格を含む) |
| `1` | エラー(ネットワーク失敗、コンパイル不能、設定破損など) |
| `2` | コマンドは完走したが結果が不合格(WA/TLE、クッキー無効など) |

スクリプトからは終了コードで成否を判定できます。

## ログインセッションについて

`acc auth login` で保存するクッキーは `~/.config/atcoder-cli/cookies.txt` に平文で保存され、所有者のみ読み書き可能な権限(0600)に自動設定されます。取り扱いには注意してください。

## ライセンス

MIT LICENSE
