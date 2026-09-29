# JSON出力

すべてのコマンドで `--json` フラグを付けると、すべての標準出力は NDJSON(1行1JSON)形式になります。

## 形式

各行は以下の形式のJSONです。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `event` | string | イベントの種類 |
| `data` | object | イベントごとのデータ(下記参照) |

```json
{"event": "event_type", "data": { /* event_typeに対応したデータ */ }}
```

## 共通イベント

すべてのサブコマンドで出力される可能性があるイベントです。

### log

汎用のログメッセージです。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `level` | string | ログレベル。`"success"` / `"info"` / `"warn"` のいずれか |
| `message` | string | ログの本文 |

```json
{"event":"log","data":{"level":"info","message":"Downloading..."}}
```

## サブコマンド別イベント

### download

| event | 説明 |
|-------|------|
| [`markdown_saved`](#markdown_saved) | Markdownをファイルに保存した |
| [`markdown_downloaded`](#markdown_downloaded) | Markdownを取得した(ファイルには保存していない) |

#### markdown_saved

Markdownをファイルに保存したときに出力されます。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `path` | string | 保存先のパス |

```json
{"event":"markdown_saved","data":{"path":"./output/page.md"}}
```

#### markdown_downloaded

Markdownを取得したときに出力されます。本文は `text` に入ります。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `text` | string | Markdownの本文 |

```json
{"event":"markdown_downloaded","data":{"text":"# Title\n\nBody..."}}
```
