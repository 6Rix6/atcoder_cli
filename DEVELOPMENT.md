# DEVELOPMENT.md

atcoder_cli の開発者向けドキュメント。アーキテクチャ、ディレクトリ構成、設計方針をまとめる。

## アーキテクチャ概要

バイナリ(`acc`)とライブラリ(`atcoder_cli`)を分離したクレート構成。依存は一方向:

```
main.rs ──> cli ──> atcoder / config / cookie / process / utils
```

- `cli` が唯一の表示層(標準出力/エラー出力・スピナー・対話プロンプトを担う)
- lib層のモジュールは **出力・`unwrap`・プロセス終了を禁止**(clippy lintで強制)し、`Result<T, E>` のみを返す
- エラーは `AppError` まで伝播し、**終了コードの選択と詳細表示は `main.rs` に集約**

## ディレクトリ構成

```
.
├── src/
│   ├── main.rs            # エントリポイント。Outcome→終了コード(0/1/2)変換、エラー表示
│   ├── lib.rs             # クレートルート
│   ├── error.rs           # AppError: 全ドメインエラーの合成(transparent)
│   ├── constants.rs       # URL・ファイル名等の定数
│   ├── languages.rs       # acc init 用の言語プリセット(C++/Python/Rust)
│   │
│   ├── cli/               # コマンドハンドラ + 表示層(出力OKな唯一の層)
│   │   ├── mod.rs         # Cli定義、run()ディスパッチャ、report_error()、ensure_interactive()
│   │   ├── auth.rs        # acc auth login/logout/status
│   │   ├── test.rs        # acc test(コンパイル→サンプル実行→判定)
│   │   ├── dl.rs          # acc download(問題文Markdown取得)
│   │   ├── init.rs        # acc init(対話で設定生成)
│   │   ├── utils/mod.rs   # セッションcookie読み出し(SessionStatus)
│   │   └── styles/
│   │       ├── macros.rs         # print系マクロ(success_ln!/error_ln!/...)
│   │       ├── spinner.rs        # indicatifラッパ(静的テンプレートのみ)
│   │       └── dialog_theme.rs   # dialoguerテーマ改造版(vendored)
│   │
│   ├── atcoder/           # AtCoderとの通信・解析層
│   │   ├── client.rs      # reqwest blockingクライアント(cookie付与、タイムアウト30秒)
│   │   ├── parser.rs      # HTML解析(scraper)。サンプル抽出・問題メタデータ(LazyLockセレクタ)
│   │   ├── markdown/      # 問題文HTML→Markdown変換(html2md)
│   │   ├── models.rs      # TaskInfo / TaskPage / TaskData / SampleInput
│   │   ├── utils.rs       # タスクID/URL → TaskInfo 解析
│   │   └── error.rs       # AtcoderError(Http/Parse/NoSamples)
│   │
│   ├── config/            # atcoder.toml の読み書き
│   │   ├── models.rs      # Config / ConfigRef(serde)
│   │   ├── utils.rs       # ローカル優先→グローバルフォールバックのロード、生成
│   │   └── error.rs       # ConfigError(Read/Parse/Write/Serialize/NoConfigDir)
│   │
│   ├── cookie/            # REVEL_SESSION クッキーの永続化
│   │   ├── storage.rs     # key=valueテキスト形式。保存時に0600へ権限強制
│   │   ├── utils.rs       # パス解決、cookie-octet検証(is_valid_session_value)
│   │   └── error.rs       # CookieError(Read/Write/Remove/NoConfigDir)
│   │
│   ├── process/           # サブプロセス実行
│   │   ├── compiler.rs    # コンパイル実行(CompileError)
│   │   └── runner.rs      # テストケース実行。watchdogスレッドが制限時間超過でkillpg(SIGKILL)
│   │
│   └── utils/
│       └── gnu_time.rs    # GNU time の検出(メモリ計測用)
│
├── README.md              # 利用者ドキュメント(日本語)
└── README-en.md           # 利用者ドキュメント(英語)
```

## 主要な設計方針

### エラー処理

- 各ドメインが thiserror のエラー型を持つ(`AtcoderError` / `CookieError` / `CompileError` / `ConfigError` / `RunError`)
- `src/error.rs` の `AppError` が `#[from]` で合成し、ハンドラは `Result<Outcome, AppError>` を返すだけ
- 「コマンド完走だが結果不合格」(WA/TLE、無効クッキー)はエラーではなく `Outcome::Failed`(exit code 2)
- 詳細なエラー表示は `cli::report_error()` が source チェーン全体を1回だけ stderr へ出力する
- `std::process::exit` はコードベースに存在しない(`main` の戻り値 `ExitCode` のみ)

| 終了コード | 意味 |
|---|---|
| 0 | 成功 |
| 1 | 実行時エラー |
| 2 | 完走したが結果NG |

### 非同期について

tokioは使用せず完全同期。ワークロードが逐次的(fetch→compile→逐次テスト)で並行性の恩恵がないため。

- HTTP: `reqwest::blocking`
- プロセス実行: `std::process` + watchdog スレッド(unix では `setsid` + `killpg` でプロセスグループごと kill)
- 対話プロンプト(dialoguer)は TTY 前提のため、呼び出し前に `cli::ensure_interactive()` で非対話環境を検出して中断する(非TTYでは dialoguer が無限再描画するため)

### Lint 方針

lib層モジュール(atcoder/config/cookie/process/utils)には冒頭で以下を適用:

```rust
#![deny(clippy::unwrap_used, clippy::print_stdout, clippy::print_stderr)]
```

- パニックは起動時不変条件の `expect`(静的セレクタ・正規表現など、固定文字列)のみ許容
- 表示は cli 層に集約
- 静的な `Selector` / `Regex` は `LazyLock` で一度だけ構築する

### テスト

- `cargo test` で単体テストを実行(各モジュール末尾の `#[cfg(test)] mod tests`)
- 重点対象: HTMLパーサ(AtCoderの仕様変更で壊れる境界)、タスクID解析、cookie検証・永続化、テンプレート展開、GNU time出力パース
- HTMLフィクスチャは実構造(`span.lang-ja`, `.part > section > h3/pre`)を再現したインライン定数

## 開発フロー

```sh
cargo build          # ビルド
cargo test           # 単体テスト
cargo clippy --all-targets   # 警告0を維持
cargo fmt --check    # フォーマット確認(rustfmt, edition 2024)
```

コミットメッセージは日本語、1コミット=1論理変更(ビルド可能な状態を保つ)。

## 既知の制限・TODO

- `expand_template`(src/cli/test.rs)は HashMap 反復順序依存の置換。値に `${...}` を含めると非決定的になりうる
- `dialog_theme.rs` は dialoguer `ColorfulTheme` の改造コピー(約450行)。prefix描画ブロックが8箇所重複
- メモリ制限は計測表示のみで強制しない
- Windows は TLE 時のプロセスキル(killpg)未対応
