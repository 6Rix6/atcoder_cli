# atcoder_cli

## subcommands

### acc auth
- acc auth login
atcoderにログイン。実際にはatcoderに誘導しクッキーをコピペ

- acc auth logout
クッキーファイルを削除

- acc auth status
クッキーが有効かどうか検証

### acc test
- acc test <task_id>
指定されたタスクのテストケースを実行。atcoder.tomlまたは引数でコンパイル・実行コマンドを指定

### acc init
- acc init
対話形式で使用言語を選択し、atcoder.tomlを生成

### acc download (dl)
- acc download <task_id>
問題文をMarkdownで標準出力に取得
