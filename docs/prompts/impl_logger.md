# Rust Text Logger 実装仕様書

## 1. 目的

Rust で簡易的なテキストログライブラリを実装する。

実装する機能は以下のとおり。

- ログレベル管理
- ログレコード生成
- ログレベルによる出力フィルタリング
- 複数ログレコードの一括登録
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
| log_level | INFO |

`max_file_size` は 200 KiB（200 × 1024 Bytes）とする。

`max_file_num` は現在のログファイルを含む最大ファイル数とする。

`log_level` はログ出力フィルタリングに使用する。

---

## 4. LogLevel

### 4.1. enum 定義

以下のログレベルを定義する。

```rust
#[allow(non_camel_case_types)]
pub enum LogLevel {
    CRITICAL,
    ERROR,
    WARNING,
    INFO,
    DEBUG,
}
```

### 4.2. ログレベルの重要度

ログレベルの重要度は以下の順番とする。

```text
CRITICAL > ERROR > WARNING > INFO > DEBUG
```

この順番をログ出力フィルタリングに使用する。

実装ではログレベルの重要度を数値として比較できるようにすること。

重要度の値は以下とする。

| LogLevel | 重要度 |
|---|---:|
| CRITICAL | 5 |
| ERROR | 4 |
| WARNING | 3 |
| INFO | 2 |
| DEBUG | 1 |

### 4.3. 文字列変換

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
- UTC に変換しないこと。

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
    log_level: LogLevel,
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
| log_level | LogLevel | ログ出力フィルタリング設定 |

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
log_level = LogLevel::INFO;
```

#### new_with

```rust
pub fn new_with<P: AsRef<Path>>(
    log_path: P,
    max_file_num: u32,
    max_file_size: u32,
    realtime: bool,
    log_level: LogLevel,
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

## 7. ログレベルフィルタリング

### 7.1. 基本仕様

Logger は `log_level` に設定されたレベルを基準に、ログを出力するかどうかを判定する。

設定されたレベルより重要度が低いログは無視する。

設定されたレベルと同じ重要度、または重要度が高いログは出力対象とする。

判定条件：

```text
record.level の重要度 >= logger.log_level の重要度
```

### 7.2. 判定例

| Logger の log_level | critical | error | warning | info | debug |
|---|---|---|---|---|---|
| CRITICAL | 出力 | 無視 | 無視 | 無視 | 無視 |
| ERROR | 出力 | 出力 | 無視 | 無視 | 無視 |
| WARNING | 出力 | 出力 | 出力 | 無視 | 無視 |
| INFO | 出力 | 出力 | 出力 | 出力 | 無視 |
| DEBUG | 出力 | 出力 | 出力 | 出力 | 出力 |

### 7.3. フィルタリングの適用対象

フィルタリングは Logger にログを追加する時点で実施する。

対象メソッド：

- critical
- error
- warning
- info
- debug
- write_recs

フィルタリング条件を満たさないログは recodes に追加しない。

また、フィルタリング対象のログだけが渡された場合は、ファイルへの書き込みを行わない。

### 7.4. フィルタリングの実装

LogLevel の重要度を比較する処理を共通化すること。

各ログメソッドに個別の比較ロジックを実装しないこと。

---

## 8. Logger のログメソッド

Logger に以下の5メソッドを実装する。

```rust
pub fn critical(&mut self, msg: &str) -> std::io::Result<()>;
pub fn error(&mut self, msg: &str) -> std::io::Result<()>;
pub fn warning(&mut self, msg: &str) -> std::io::Result<()>;
pub fn info(&mut self, msg: &str) -> std::io::Result<()>;
pub fn debug(&mut self, msg: &str) -> std::io::Result<()>;
```

### 8.1. 共通処理

各メソッドは以下の処理を行う。

1. 対応する LogRecode を生成する。
2. Logger の log_level とレコードの level を比較する。
3. 出力対象の場合のみ recodes に追加する。
4. 出力対象でない場合は何もせず Ok(()) を返す。
5. realtime が true の場合は out() を実行する。
6. realtime が false の場合は書き込みを行わず、バッファに保持する。

### 8.2. realtime の動作

| realtime | 動作 |
|---|---|
| true | 出力対象ログの追加後に out() を実行 |
| false | 出力対象ログをバッファに蓄積 |

out() がエラーになった場合は、そのエラーを呼び出し元に返す。

### 8.3. フィルタリングされたログ

フィルタリングによってログが無視された場合：

- recodes に追加しない。
- out() を実行しない。
- エラーを返さない。
- Ok(()) を返す。

---

## 9. write_recs メソッド

### 9.1. メソッド定義

複数の LogRecode を一括で Logger に追加するメソッドを実装する。

```rust
pub fn write_recs(&mut self, recs: &[LogRecode]) -> std::io::Result<()>;
```

引数：

| 引数 | 型 | 説明 |
|---|---|---|
| recs | &[LogRecode] | 追加するログレコードの配列 |

### 9.2. 基本処理

以下の順番で処理する。

1. 引数 recs の各 LogRecode を確認する。
2. Logger の log_level と各レコードの level を比較する。
3. 出力対象のレコードだけを内部 recodes に追加する。
4. 出力対象外のレコードは無視する。
5. realtime が true の場合は out() を実行する。
6. realtime が false の場合はバッファに保持する。

### 9.3. フィルタリング

write_recs にも通常のログメソッドと同じフィルタリングを適用する。

例えば Logger の log_level が INFO の場合：

入力：

```rust
let recs = vec![
    LogRecode::critical("critical message"),
    LogRecode::info("info message"),
    LogRecode::debug("debug message"),
];
```

内部 recodes に追加されるレコード：

```text
CRITICAL critical message
INFO     info message
```

DEBUG は追加しない。

### 9.4. realtime

| realtime | 動作 |
|---|---|
| true | フィルタリング後のレコードを追加し、out() を実行 |
| false | フィルタリング後のレコードを追加して終了 |

### 9.5. 空配列

引数 recs が空の場合は何もせず Ok(()) を返す。

### 9.6. 注意事項

- 入力 recs の内容を変更しない。
- 入力 recs の所有権を移動しない。
- 入力 recs のうち出力対象のレコードをコピーして内部 recodes に追加する。
- 既存の内部 recodes を削除しない。
- write_recs 専用の別のログ出力処理を実装しない。
- 通常のログメソッドと同じフィルタリング・書き込み処理を使用する。

---

## 10. out メソッド

### 10.1. メソッド定義

```rust
pub fn out(&mut self) -> std::io::Result<()>;
```

recodes に保持されているログをログファイルに書き出す。

### 10.2. 基本処理

1. recodes が空の場合は何もせず Ok(()) を返す。
2. recodes の各 LogRecode を to_log() で文字列に変換する。
3. 各ログの末尾に改行を追加する。
4. UTF-8 でログファイルに追記する。
5. 書き込んだバイト数を now_size に加算する。
6. 全レコードの書き込みが完了したら recodes を空にする。
7. ファイルサイズが max_file_size を超えた場合はローテーションを実行する。

### 10.3. 書き込み方式

既存ファイルを上書きしてはならない。

必ず追記モードで書き込む。

新しいログファイルが存在しない場合は作成する。

### 10.4. バッファの保持

重要なルール：

- 書き込みに成功した場合のみ recodes を空にする。
- 書き込みに失敗した場合は recodes を保持する。
- エラーを握りつぶさず、呼び出し元に返す。
- 書き込み失敗時にログデータを勝手に破棄しない。

ファイル書き込み処理では、部分書き込みや失敗時の扱いを考慮すること。

書き込み済みデータと未書き込みデータを適切に管理し、再実行時の重複出力を防ぐこと。

### 10.5. ローテーションとの関係

ログデータの書き込みとローテーションを区別して管理する。

ログデータの書き込みが完了した後にローテーションが失敗した場合、書き込み済みデータを recodes に残してはならない。

この場合はローテーションエラーを返し、書き込み済みログを再出力しないこと。

---

## 11. ログローテーション

### 11.1. 発動条件

out() によってすべてのバッファを書き込んだ後、以下を判定する。

```rust
now_size > max_file_size
```

最大サイズと等しい場合はローテーションしない。

最大サイズを超えた場合にのみ実行する。

### 11.2. ファイル構成

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
| app.log.1 | 1世代前 |
| app.log.2 | 2世代前 |

最も古いファイルは削除する。

### 11.3. ローテーション手順

max_file_num = 3 の場合を例にする。

1. app.log.2 が存在すれば削除する。
2. app.log.1 が存在すれば app.log.2 にリネームする。
3. app.log が存在すれば app.log.1 にリネームする。
4. 新しい app.log を作成する。
5. now_size を 0 に設定する。

### 11.4. 一般化した処理

max_file_num の値に応じて同じ処理を行う。

重要：

- 現行ファイルを含めた総数を max_file_num 以下にする。
- 古いファイルから削除する。
- 世代番号の大きいファイルから順番に処理する。
- ファイルのリネーム順序を間違えない。
- 存在しないファイルをリネームしようとしない。
- max_file_num = 1 の場合は旧 app.log を削除し、新しい app.log を作成する。

max_file_num = 0 は無効な設定として扱い、new_with() で std::io::Error を返す。

### 11.5. ローテーション後

ローテーション完了後：

```rust
now_size = 0;
```

新しいログファイルへの書き込みに備える。

ローテーションに失敗した場合はエラーを返す。

---

## 12. Drop トレイト

Logger の破棄時に未出力のログをファイルへ書き込む。

```rust
impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.out();
    }
}
```

要件：

- Drop 時に必ず out() を呼び出す。
- エラーは握りつぶす。
- panic を発生させない。
- Logger の破棄によってアプリケーションの終了処理を妨げない。
- Drop からエラーを返そうとしない。

---

## 13. エラー処理

ファイル操作に関するエラーは `std::io::Result<()>` を利用する。

対象：

- ディレクトリ作成失敗
- ファイル情報取得失敗
- ファイルオープン失敗
- ファイル書き込み失敗
- ファイル削除失敗
- ファイルリネーム失敗
- ローテーション失敗

エラーを不必要に握りつぶさない。

ただし Drop 内の out() のエラーだけは握りつぶす。

エラー発生時に未書き込みの recodes の内容を失わないこと。

---

## 14. 必須テスト

テストには一時ディレクトリを使用し、実際の logs ディレクトリやユーザーのファイルを変更しないこと。

### 14.1. LogLevel

- 各 enum が正しい文字列になること。
- Display が正常に動作すること。
- 重要度の大小関係が正しいこと。

### 14.2. LogRecode

- critical が CRITICAL を設定すること。
- error が ERROR を設定すること。
- warning が WARNING を設定すること。
- info が INFO を設定すること。
- debug が DEBUG を設定すること。
- msg が正しく保存されること。
- PID が現在のプロセス ID と一致すること。
- timestamp が指定形式になっていること。
- to_log() が正しい形式を返すこと。

### 14.3. Logger 初期化

- new() がデフォルト値を設定すること。
- log_level のデフォルトが INFO であること。
- new_with() が指定値を設定すること。
- 親ディレクトリが存在しない場合に作成されること。
- 既存ログファイルのサイズを now_size に設定すること。
- 未作成のログファイルの場合 now_size が 0 になること。
- max_file_num = 0 を拒否すること。

### 14.4. ログレベルフィルタリング

以下をすべてテストする。

- log_level = CRITICAL の場合、CRITICAL だけが出力される。
- log_level = ERROR の場合、CRITICAL と ERROR が出力される。
- log_level = WARNING の場合、CRITICAL、ERROR、WARNING が出力される。
- log_level = INFO の場合、CRITICAL、ERROR、WARNING、INFO が出力される。
- log_level = DEBUG の場合、すべてのログが出力される。
- フィルタリングされたログは recodes に追加されない。
- フィルタリングされたログによって out() が呼び出されない。
- フィルタリングされたログはエラーを返さない。

### 14.5. write_recs

- 複数の LogRecode を一括追加できること。
- log_level によるフィルタリングが適用されること。
- フィルタリング対象外のレコードだけが追加されること。
- 入力 recs の内容が変更されないこと。
- 入力 recs の所有権が維持されること。
- 既存の内部 recodes が保持されること。
- realtime = false の場合、バッファに蓄積されること。
- realtime = true の場合、追加後に out() が実行されること。
- 空配列の場合は何もせず Ok(()) を返すこと。
- フィルタリング後に追加対象が0件の場合、不要なファイル書き込みを行わないこと。

### 14.6. バッファリング

- realtime = false ではログメソッド呼び出し時にファイルへ書き込まれないこと。
- recodes にログが追加されること。
- out() でログが書き込まれること。
- 書き込み成功後 recodes が空になること。
- out() を複数回呼び出しても不要な重複書き込みが発生しないこと。

### 14.7. realtime

- realtime = true ではログメソッド呼び出し時にファイルへ書き込まれること。
- 書き込み後 recodes が空になること。

### 14.8. ローテーション

- max_file_size 以下ではローテーションされないこと。
- max_file_size を超えるとローテーションされること。
- app.log が app.log.1 になること。
- app.log.1 が app.log.2 になること。
- 最大世代数を超えたファイルが削除されること。
- max_file_num = 1 が正常に動作すること。
- ローテーション後 now_size が 0 になること。

### 14.9. Drop

- Logger のスコープ終了時にバッファが書き込まれること。
- バッファが空の場合でも安全に破棄できること。

### 14.10. エラー処理

- 無効なパスを指定した場合にエラーが返されること。
- 書き込み失敗時に未書き込みのバッファが失われないこと。
- ローテーション失敗時にエラーが返されること。
- 書き込み済みログが再出力されないこと。

---

## 15. 実装上の注意事項

以下の事項を必ず守ること。

1. 仕様にない機能を勝手に追加しない。
2. ログ形式を変更しない。
3. timestamp にタイムゾーンを追加しない。
4. pid をログ文字列に追加しない。
5. max_file_size の単位を変更しない。
6. max_file_num の数え方を変更しない。
7. ログを上書きしない。追記する。
8. 書き込み失敗時に未書き込みバッファを破棄しない。
9. Drop 内で panic を発生させない。
10. 不必要な外部クレートを追加しない。
11. 不必要な unsafe を使用しない。
12. 既存プロジェクトのコードを無断で破壊しない。
13. ログレベルの重要度比較を共通化する。
14. 通常のログメソッドと write_recs で異なるフィルタリング処理を実装しない。
15. ログレベルフィルタリングは recodes への追加前に行う。

---

## 16. 実装完了条件

以下をすべて満たした場合のみ実装完了とする。

- [ ] LogLevel が実装されている。
- [ ] LogLevel の Display が実装されている。
- [ ] LogLevel の重要度比較が実装されている。
- [ ] LogRecode が実装されている。
- [ ] 5種類のログ生成メソッドが実装されている。
- [ ] timestamp がローカル時刻・マイクロ秒精度である。
- [ ] PID が取得される。
- [ ] to_log() が実装されている。
- [ ] Logger が実装されている。
- [ ] new() が実装されている。
- [ ] new_with() が実装されている。
- [ ] log_level のデフォルトが INFO である。
- [ ] 5種類の Logger ログメソッドが実装されている。
- [ ] ログレベルフィルタリングが実装されている。
- [ ] write_recs() が実装されている。
- [ ] write_recs() にもログレベルフィルタリングが適用される。
- [ ] out() が実装されている。
- [ ] realtime が動作する。
- [ ] ログファイルの追記が動作する。
- [ ] ログローテーションが動作する。
- [ ] 古いログファイルが削除される。
- [ ] Drop による自動出力が動作する。
- [ ] エラー処理が仕様どおりである。
- [ ] 必須テストがすべて実装されている。

---

## 17. 品質確認

実装後、以下を実行する。

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

すべて成功すること。

失敗した場合は原因を調査して修正し、再実行する。

テストを通すためだけにテストを削除したり、警告を不適切に抑制したりしてはならない。

---

## 18. 作業完了時の報告

実装完了後、以下を報告する。

1. 作成・変更したファイル
2. 実装した機能
3. 追加した依存クレートとその理由
4. 実施したテスト
5. cargo fmt / clippy / test の実行結果
6. 未対応事項
7. 仕様から変更した箇所がある場合は、その理由

仕様どおりに実装できなかった項目がある場合、完了したように報告してはならない。

必ず未完了項目と理由を明記する。