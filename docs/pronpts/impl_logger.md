# Rust Text Logger 実装仕様書の作成・実装指示

## 1. 目的

Rust で簡易的なテキストログライブラリを実装する。

実装する機能は以下のとおり。

- ログレベル管理
- ログレコード生成
- ログファイルへの書き込み
- ログのバッファリング
- ファイルサイズに基づくログローテーション
- 指定世代数を超えた古いログファイルの削除
- Logger 破棄時の自動書き込み

本仕様書に記載された要件を優先すること。

**重要：独自の機能追加、仕様変更、過剰な抽象化を行わないこと。**

仕様に記載されていない事項については、既存プロジェクトの設計・規約に従うこと。判断できない場合は、勝手に実装せずユーザーに質問すること。

---

## 2. 実装前の確認事項

実装を開始する前に、以下を確認する。

1. リポジトリのディレクトリ構成
2. 既存の `Cargo.toml`
3. 既存の `src` ディレクトリ
4. 既存のコーディング規約
5. 既存のテスト構成
6. 既存の依存クレート

既存プロジェクトがある場合は、その構成に合わせて実装する。

既存のコードを無断で削除・変更しないこと。

必要な依存クレートが既に存在する場合は再利用する。

実装に必要な依存クレートがない場合は、追加の必要性を判断する。

---

## 3. 基本仕様

### 3.1. 使用言語

- Rust
- Rust の標準的な所有権・借用ルールに従う。
- 不必要な `unsafe` は使用しない。
- 不必要な外部依存を追加しない。

### 3.2. ログファイル

出力先の例：

`logs/app.log`

ログファイルの親ディレクトリが存在しない場合は、自動的に作成する。

### 3.3. デフォルト設定

| 設定 | デフォルト値 |
|---|---|
| max_file_num | 7 |
| max_file_size | 204800 Bytes |
| realtime | false |

`max_file_size` は 200 KiB（200 × 1024 Bytes）とする。

`max_file_num` は現在のログファイルを含む最大ファイル数とする。

---

## 4. LogLevel

### 4.1. enum 定義

以下のログレベルを定義する。

```rust
pub enum LogLevel {
    CRITICAL,
    ERROR,
    WARNING,
    INFO,
    DEBUG,
}
```

必要に応じて `#[allow(non_camel_case_types)]` を使用し、指定された名称を維持する。

### 4.2. 文字列変換

ログレベルを簡単に文字列へ変換できるようにする。

`Display` トレイトを実装すること。

変換結果：

| enum | 文字列 |
|---|---|
| CRITICAL | CRITICAL |
| ERROR | ERROR |
| WARNING | WARNING |
| INFO | INFO |
| DEBUG | DEBUG |

以下のように利用できること。

```rust
let level = LogLevel::INFO;
println!("{}", level);
```

---

## 5. LogRecode

### 5.1. 構造体定義

ログ1件分の情報を保持する構造体。

```rust
pub struct LogRecode {
    pub timestamp: String,
    pub level: LogLevel,
    pub msg: String,
    pub pid: i32,
}
```

各フィールドの意味：

| フィールド | 型 | 説明 |
|---|---|---|
| timestamp | String | ログ生成時刻 |
| level | LogLevel | ログレベル |
| msg | String | ログメッセージ |
| pid | i32 | プロセスID |

### 5.2. timestamp

ログレコードの生成時に現在時刻を取得する。

以下の形式で保存する。

`2026-11-11T12:13:45.123456`

要件：

- ISO 8601 に準拠した形式とする。
- ローカル時刻を使用する。
- タイムゾーン情報は付与しない。
- マイクロ秒精度とする。
- 形式は `YYYY-MM-DDTHH:MM:SS.ffffff` とする。

UTC に変換しないこと。

### 5.3. pid

ログレコード生成時に、実行中プロセスの PID を取得する。

### 5.4. ログ生成メソッド

以下の5メソッドを実装する。

```rust
impl LogRecode {
    pub fn critical(msg: &str) -> Self;
    pub fn error(msg: &str) -> Self;
    pub fn warning(msg: &str) -> Self;
    pub fn info(msg: &str) -> Self;
    pub fn debug(msg: &str) -> Self;
}
```

各メソッドの処理：

1. 引数 `msg` を受け取る。
2. 現在時刻を取得する。
3. PID を取得する。
4. メソッドに対応する LogLevel を設定する。
5. msg を所有権のある String に変換する。
6. LogRecode インスタンスを生成して返す。

対応関係：

| メソッド | LogLevel |
|---|---|
| critical | CRITICAL |
| error | ERROR |
| warning | WARNING |
| info | INFO |
| debug | DEBUG |

例：

```rust
let record = LogRecode::info("Application started");
```

### 5.5. to_log

ログレコードをテキストに変換するメソッドを実装する。

```rust
pub fn to_log(&self) -> String;
```

出力形式：

`timestamp [level] msg`

例：

```text
2026-11-11T12:13:45.123456 [INFO] Application started
```

要件：

- timestamp を先頭にする。
- LogLevel を `[]` で囲む。
- msg を末尾にする。
- pid はログ文字列に含めない。
- 戻り値は所有権のある String とする。
- 改行文字は含めない。改行はファイル書き込み時に追加する。

---

## 6. Logger

### 6.1. 構造体定義

Logger 本体を以下のフィールドで構成する。

```rust
pub struct Logger {
    log_path: PathBuf,
    recodes: Vec<LogRecode>,
    max_file_num: u32,
    max_file_size: u32,
    realtime: bool,
    now_size: u32,
}
```

各フィールドの意味：

| フィールド | 型 | 説明 |
|---|---|---|
| log_path | PathBuf | ログ出力先 |
| recodes | Vec<LogRecode> | 未出力ログ |
| max_file_num | u32 | 最大ログファイル数 |
| max_file_size | u32 | 最大ログファイルサイズ（Bytes） |
| realtime | bool | 即時書き込み設定 |
| now_size | u32 | 現在のログファイルサイズ（Bytes） |

### 6.2. 初期化メソッド

以下の2種類を実装する。

#### new

```rust
pub fn new<P: AsRef<Path>>(log_path: P) -> std::io::Result<Self>;
```

ファイルパスだけを指定する初期化メソッド。

以下のデフォルト設定を使用する。

```rust
max_file_num = 7;
max_file_size = 200 * 1024;
realtime = false;
```

#### new_with

```rust
pub fn new_with<P: AsRef<Path>>(
    log_path: P,
    max_file_num: u32,
    max_file_size: u32,
    realtime: bool,
) -> std::io::Result<Self>;
```

全オプションを指定して初期化する。

#### 初期化処理

両メソッドは同じ初期化処理を利用する。

1. log_path を PathBuf に変換する。
2. log_path の親ディレクトリを取得する。
3. 親ディレクトリが存在しない場合は作成する。
4. ログファイルが存在するか確認する。
5. 存在する場合はファイルサイズを取得し、now_size に設定する。
6. 存在しない場合は now_size を 0 にする。
7. recodes を空の Vec で初期化する。
8. 指定された設定値を各フィールドに保存する。

ファイルサイズは実際のファイルサイズを使用すること。

ログファイル自体が存在しない場合は、初期化時点では空のバッファ状態として扱う。

---

## 7. Logger のログメソッド

Logger に以下の5メソッドを実装する。

```rust
pub fn critical(&mut self, msg: &str) -> std::io::Result<()>;
pub fn error(&mut self, msg: &str) -> std::io::Result<()>;
pub fn warning(&mut self, msg: &str) -> std::io::Result<()>;
pub fn info(&mut self, msg: &str) -> std::io::Result<()>;
pub fn debug(&mut self, msg: &str) -> std::io::Result<()>;
```

### 7.1. 共通処理

各メソッドは以下の処理を行う。

1. 対応する LogRecode を生成する。
2. recodes に追加する。
3. realtime が true の場合は out() を実行する。
4. realtime が false の場合は書き込みを行わず、バッファに保持する。

### 7.2. realtime の動作

| realtime | 動作 |
|---|---|
| true | ログ追加のたびに out() を実行 |
| false | ログをバッファに蓄積 |

out() がエラーになった場合は、そのエラーを呼び出し元に返す。

---

## 8. out メソッド

### 8.1. メソッド定義

```rust
pub fn out(&mut self) -> std::io::Result<()>;
```

recodes に保持されているログをログファイルに書き出す。

### 8.2. 基本処理

1. recodes が空の場合は何もせず Ok(()) を返す。
2. recodes の各 LogRecode を to_log() で文字列に変換する。
3. 各ログの末尾に改行を追加する。
4. UTF-8 でログファイルに追記する。
5. 書き込んだバイト数を now_size に加算する。
6. 全レコードの書き込みが完了したら recodes を空にする。
7. ファイルサイズが max_file_size を超えた場合はローテーションを実行する。

### 8.3. 書き込み方式

既存ファイルを上書きしてはならない。

必ず追記モードで書き込む。

新しいログファイルが存在しない場合は作成する。

### 8.4. バッファの保持

重要なルール：

- 書き込みに成功した場合のみ recodes を空にする。
- 書き込みに失敗した場合は recodes を保持する。
- エラーを握りつぶさず、呼び出し元に返す。
- 書き込み失敗時にログデータを勝手に破棄しない。

ファイル書き込み処理では、部分書き込みや失敗時の扱いを考慮すること。

---

## 9. ログローテーション

### 9.1. 発動条件

out() によってすべてのバッファを書き込んだ後、以下を判定する。

```rust
now_size > max_file_size
```

最大サイズと等しい場合はローテーションしない。

最大サイズを超えた場合にのみ実行する。

### 9.2. ファイル構成

例えば max_file_num = 3 の場合：

ローテーション前：

```text
logs/
    app.log
    app.log.1
    app.log.2
```

ローテーション後：

```text
logs/
    app.log
    app.log.1
    app.log.2
```

世代の意味：

| ファイル | 意味 |
|---|---|
| app.log | 現在のログ |
| app.log.1

