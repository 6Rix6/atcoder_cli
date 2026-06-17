# atcoder_cli

## subcommands

### acc auth
- acc auth login
atcoderにログイン。実際にはatcoderに誘導しクッキーをコピペ

- acc auth logout
クッキーファイルを削除

- acc auth statuc
クッキーが有効かどうか検証

### acc test
- acc test <task_id>
指定されたタスクのテストケースを実行。atcoder.tomlまたは引数でコンパイル・実行コマンドを指定

### acc set
- acc test <contest_id>
そのセッション内でコンテストを設定。設定するとtask_idが問題番号の指定だけでよくなる
