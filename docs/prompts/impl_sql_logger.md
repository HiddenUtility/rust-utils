# Codex Implementation Specification: SqlLogger (Rust)

## 1. Project Overview

RustでSQLiteを利用した簡易ログライブラリ `SqlLogger` を実装してください。

目的は、アプリケーションから簡単にログを記録し、SQLiteデータベースに蓄積・検索できるようにすることです。

実装では、以下を重視してください。

1. ログ書き込み速度
2. メモリ上でのログバッファリング
3. SQLiteへの一括書き込み
4. 安全なトランザクション管理
5. ログレベルによる出力制御
6. 大量レコードに対する検索性能
7. データベースの肥大化防止
8. エラーによるアプリケーションへの影響を最小限にすること

この仕様書に記載されていない機能を勝手に追加しないでください。

仕様に矛盾がある場合は、実装を開始する前に質問してください。

---

## 2. 開発環境・使用ライブラリ

### 2.1 開発言語

- Rust
- Editionはプロジェクトの既存設定に従う。
- 既存プロジェクトがない場合は、安定版Rustを対象とする。

### 2.2 外部クレートの選定方針

外部クレートを使用する場合、以下の条件を満たしてください。

- MIT、Apache-2.0、BSDなどの利用可能なライセンスであること。
- 可能な限りPure Rust製を優先すること。
- Pure Rust製でない場合は、採用理由を明確にすること。
- 有名で利用実績があり、継続的にメンテナンスされているものを優先すること。
- 不必要な依存関係を増やさないこと。
- 各クレートのライセンスを確認すること。

### 2.3 使用するクレート

以下を基本構成とします。

| Crate | 用途 |
|---|---|
| rusqlite | SQLite操作 |
| chrono | Unix timeからローカル時刻への変換 |
| serde | JSON変換への対応 |

#### rusqlite

SQLiteの操作には `rusqlite` を使用してください。

- ライセンスを確認すること。
- SQLiteのC実装を利用することは許容する。
- 必要以上にSQLiteの機能を抽象化しないこと。
- SQLiteへの接続はインスタンス内で維持すること。

SQLite本体のビルド方法は、既存環境との互換性を優先してください。

`bundled` featureを採用する場合は、SQLite本体のライセンスも確認してください。

#### chrono

ローカル時刻の表示に使用してください。

#### serde

将来的にJSONへ変換できるように、必要な構造体とenumにSerialize、Deserializeを実装してください。

---

## 3. SqlLogLevel

### 3.1 定義

ログレベルを表すenumを定義してください。

```rust
#[repr(u8)]
pub enum SqlLogLevel {
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
}
```

Rustの命名規則に合わせて、enumの内部名はPascalCaseとしてください。

外部に公開するログレベルの文字列表現は、以下に統一してください。

| Enum | String | 数値 |
|---|---|---:|
| Critical | CRITICAL | 5 |
| Error | ERROR | 4 |
| Warning | WARNING | 3 |
| Info | INFO | 2 |
| Debug | DEBUG | 1 |

数値が大きいほど優先順位が高いものとします。

### 3.2 必須実装

以下を実装してください。

- Clone
- Copy
- Debug
- PartialEq
- Eq
- PartialOrd
- Ord
- Serialize
- Deserialize
- Display

Displayではログレベルの文字列を返してください。

例：

```rust
format!("{}", SqlLogLevel::Info)
```

結果：

```text
INFO
```

Serialize、Deserializeでは、ログレベルを大文字の文字列として扱えるようにしてください。

例えば、INFOはJSON上でも `"INFO"` として表現してください。

serdeの属性を利用する場合は、`SCREAMING_SNAKE_CASE` に対応する設定を使用してください。

### 3.3 ログレベルの優先順位

ログレベルの優先順位は以下とします。

```text
CRITICAL > ERROR > WARNING > INFO > DEBUG
```

数値による優先順位は、必ず以下の対応にしてください。

```text
DEBUG    = 1
INFO     = 2
WARNING  = 3
ERROR    = 4
CRITICAL = 5
```

**数値が大きいほど優先順位が高くなります。**

ログ出力の判定条件は、必ず以下を使用してください。

```rust
record_level >= configured_log_level
```

これは「記録しようとしているログの優先順位が、設定された最低ログレベル以上である場合に記録する」という意味です。

以下の条件にしてはいけません。

```rust
record_level <= configured_log_level
```

### 3.4 ログレベルの比較例

| 設定レベル | CRITICAL | ERROR | WARNING | INFO | DEBUG |
|---|---|---|---|---|---|
| CRITICAL | 出力 | 無視 | 無視 | 無視 | 無視 |
| ERROR | 出力 | 出力 | 無視 | 無視 | 無視 |
| WARNING | 出力 | 出力 | 出力 | 無視 | 無視 |
| INFO | 出力 | 出力 | 出力 | 出力 | 無視 |
| DEBUG | 出力 | 出力 | 出力 | 出力 | 出力 |

例えば、`log_level = INFO` の場合、INFOより優先順位が低いDEBUGは無視し、INFO以上のログはすべて記録してください。

### 3.5 比較ロジックの実装

ログレベルの比較処理は一箇所に集約してください。

enumの数値を明示的に取得できるメソッドを実装し、その数値を使って比較してください。

実装例：

```rust
impl SqlLogLevel {
    pub const fn priority(self) -> u8 {
        self as u8
    }
}
```

判定処理の例：

```rust
fn should_log(
    record_level: SqlLogLevel,
    configured_log_level: SqlLogLevel,
) -> bool {
    record_level.priority() >= configured_log_level.priority()
}
```

各ログメソッドに個別の比較処理を実装してはいけません。

すべてのログメソッドが共通の判定処理を利用するようにしてください。

また、ログレベルを文字列に変換して大小比較する実装は禁止します。

---

## 4. SqlLogRecode

### 4.1 定義

ログレコードを保持する構造体を定義してください。

```rust
pub struct SqlLogRecode {
    pub timestamp: f64,
    pub level: SqlLogLevel,
    pub name: String,
    pub msg: String,
    pub pid: usize,
}
```

### 4.2 フィールド仕様

| Field | Type | Description |
|---|---|---|
| timestamp | f64 | Unix time（秒単位） |
| level | SqlLogLevel | ログレベル |
| name | String | ログの名前 |
| msg | String | ログメッセージ |
| pid | usize | プロセスID |

timestampはUnix Epochからの経過秒数を浮動小数点数で保持してください。

小数部分にはマイクロ秒相当の精度を保持できるようにしてください。

時刻の取得には `SystemTime::now()` を使用してください。

Unix time自体はタイムゾーンに依存しない値とします。

### 4.3 ログ生成メソッド

以下のメソッドを実装してください。

```rust
impl SqlLogRecode {
    pub fn critical(name: &str, msg: &str) -> Self;
    pub fn error(name: &str, msg: &str) -> Self;
    pub fn warning(name: &str, msg: &str) -> Self;
    pub fn info(name: &str, msg: &str) -> Self;
    pub fn debug(name: &str, msg: &str) -> Self;
}
```

#### 動作仕様

各メソッドは以下を実行してください。

1. 呼び出し時点のUnix timeを取得する。
2. 引数のnameをStringに変換する。
3. 引数のmsgをStringに変換する。
4. 現在のプロセスIDを取得する。
5. メソッドに対応したSqlLogLevelを設定する。
6. SqlLogRecodeを生成して返す。

pidの取得には標準ライブラリの `std::process::id()` を使用してください。

### 4.4 DisplayとDebug

ログレコードを表示する場合、timestampをISO 8601形式に変換してください。

表示形式：

```text
2026-11-11T12:13:45.123456
```

要件：

- ローカルタイムゾーンで表示する。
- マイクロ秒まで表示する。
- タイムゾーンのオフセットは表示しない。
- timestampの内部値は変更しない。

DisplayとDebugの両方で、timestampがISO 8601形式で表示されるようにしてください。

Debugは構造体の各フィールドを確認できる形式にしてください。

必要に応じてDebugを手動実装してください。単純なderive(Debug)によってtimestampがUnix timeの数値のまま表示される状態にしてはいけません。

時刻変換にはchronoを使用してください。

### 4.5 JSON対応

将来的なJSON変換を考慮し、serdeのSerializeとDeserializeを実装してください。

JSONに変換する場合、以下の形式で扱える構造にしてください。

- timestampはf64。
- levelは大文字のログレベル文字列。
- nameは文字列。
- msgは文字列。
- pidは数値。

JSON変換時にtimestampをISO 8601文字列へ変更しないでください。

---

## 5. SqlLogger

### 5.1 構造体定義

Logger本体を以下の構造体として実装してください。

```rust
pub struct SqlLogger {
    db_path: PathBuf,
    recodes: Vec<SqlLogRecode>,
    realtime: bool,
    log_level: SqlLogLevel,
    now_row_num: u64,
    max_rec_num: u64,
    conn: Option<rusqlite::Connection>,
}
```

追加フィールドは必要最小限にしてください。

### 5.2 フィールド仕様

| Field | Type | Description |
|---|---|---|
| db_path | PathBuf | SQLiteファイルの出力先 |
| recodes | Vec<SqlLogRecode> | 書き込み待ちログ |
| realtime | bool | ログ追加時の即時書き込み |
| log_level | SqlLogLevel | 出力する最低ログレベル |
| now_row_num | u64 | DB内の現在のレコード数 |
| max_rec_num | u64 | 最大レコード数 |
| conn | Option<Connection> | SQLite接続 |

### 5.3 初期値

以下をデフォルト値としてください。

| Field | Default |
|---|---|
| realtime | false |
| log_level | INFO |
| now_row_num | DBの実際の行数 |
| max_rec_num | 10,000,000 |
| recodes | 空のVec |

### 5.4 Loggerのname

SqlLoggerのログメソッドはmsgのみを受け取ります。

そのため、SqlLoggerから生成するSqlLogRecodeのnameは固定値 `"logger"` としてください。

SqlLogRecode単体の生成メソッドでは、引き続きnameを引数で指定できるようにしてください。

---

## 6. コンストラクタ

以下のコンストラクタを実装してください。

### 6.1 new

```rust
pub fn new(db_path: impl AsRef<Path>) -> Result<Self, SqlLoggerError>;
```

デフォルト設定でSqlLoggerを生成します。

### 6.2 new_with

```rust
pub fn new_with(
    db_path: impl AsRef<Path>,
    realtime: bool,
    log_level: SqlLogLevel,
    max_rec_num: u64,
) -> Result<Self, SqlLoggerError>;
```

オプションを指定してSqlLoggerを生成します。

### 6.3 初期化処理

コンストラクタでは以下の順序で処理してください。

1. db_pathをPathBufに変換する。
2. db_pathの親ディレクトリが存在しない場合は作成する。
3. SQLiteデータベースを開く。
4. 必要なテーブルが存在しない場合は作成する。
5. 必要なインデックスが存在しない場合は作成する。
6. DB内の実際のレコード数をCOUNT(*)で取得する。
7. now_row_numに取得した値を設定する。
8. recodesを空にする。
9. SQLite接続を維持する。

db_pathの例：

```text
logs/logs.db
```

親ディレクトリの作成には `std::fs::create_dir_all()` を使用してください。

DBが存在しない場合は新規作成してください。

DBが存在する場合は、既存データを保持してください。

通常のコンストラクタで既存DBを削除してはいけません。

---

## 7. SQLiteテーブル設計

### 7.1 テーブル名

テーブル名は以下に統一してください。

```sql
sql_logs
```

### 7.2 カラム定義

以下のカラムを作成してください。

| Column | SQLite Type | Description |
|---|---|---|
| id | INTEGER PRIMARY KEY AUTOINCREMENT | 内部識別子 |
| timestamp | REAL NOT NULL | Unix time |
| log_level | TEXT NOT NULL | ログレベル文字列 |
| name | TEXT NOT NULL | ログ名 |
| pid | INTEGER NOT NULL | プロセスID |
| msg | TEXT NOT NULL | ログメッセージ |

idは取得順序を安定させるための内部識別子です。

SqlLogRecodeにはidを追加しないでください。

### 7.3 インデックス

以下のインデックスを作成してください。

```sql
CREATE INDEX IF NOT EXISTS idx_sql_logs_timestamp
ON sql_logs(timestamp DESC);
```

```sql
CREATE INDEX IF NOT EXISTS idx_sql_logs_log_level
ON sql_logs(log_level);
```

```sql
CREATE INDEX IF NOT EXISTS idx_sql_logs_name
ON sql_logs(name);
```

また、検索時にWHEREとORDER BYを効率よく処理できるよう、以下の複合インデックスも検討してください。

```sql
CREATE INDEX IF NOT EXISTS idx_sql_logs_log_level_timestamp
ON sql_logs(log_level, timestamp DESC);
```

```sql
CREATE INDEX IF NOT EXISTS idx_sql_logs_name_timestamp
ON sql_logs(name, timestamp DESC);
```

インデックスは重複して不要なものを作成しないでください。

検索性能とINSERT性能のバランスを考慮してください。

### 7.4 ログレベルの保存形式

SQLiteのlog_levelには、SqlLogLevelの大文字文字列表現を保存してください。

例：

```text
CRITICAL
ERROR
WARNING
INFO
DEBUG
```

ログレベルの数値をSQLiteに保存してはいけません。

SQLiteから読み込む際は、保存された文字列を対応するSqlLogLevelに変換してください。

---

## 8. get_recs

### 8.1 メソッド定義

```rust
pub fn get_recs(&self) -> Result<Vec<SqlLogRecode>, SqlLoggerError>;
```

### 8.2 動作仕様

SQLiteから最新の1000件を取得してください。

SQL：

```sql
SELECT timestamp, log_level, name, pid, msg
FROM sql_logs
ORDER BY timestamp DESC, id DESC
LIMIT 1000;
```

取得したレコードはSqlLogRecodeに変換してください。

戻り値のVecは新しいレコードから順番に格納してください。

DBの内容を変更してはいけません。

---

## 9. get_recs_with_limit

### 9.1 メソッド定義

```rust
pub fn get_recs_with_limit(
    &self,
    limit: u64,
) -> Result<Vec<SqlLogRecode>, SqlLoggerError>;
```

### 9.2 動作仕様

get_recsと同じSELECT処理を実行してください。

LIMITに指定された最大件数を取得してください。

SQL：

```sql
SELECT timestamp, log_level, name, pid, msg
FROM sql_logs
ORDER BY timestamp DESC, id DESC
LIMIT ?1;
```

limitはSQLパラメータとしてバインドしてください。

SQL文字列への直接埋め込みは禁止します。

limitが0の場合は空のVecを返してください。

---

## 10. get_recs_with

### 10.1 メソッド定義

```rust
pub fn get_recs_with(
    &self,
    limit: u64,
    name: Option<&str>,
    log_level: Option<SqlLogLevel>,
) -> Result<Vec<SqlLogRecode>, SqlLoggerError>;
```

### 10.2 動作仕様

get_recsと同じSELECT処理を実行します。

オプションに応じてWHERE句を追加してください。

条件：

- nameがSomeの場合はnameで検索する。
- log_levelがSomeの場合はlog_levelで検索する。
- 両方Noneの場合は全件検索する。
- 両方Someの場合はAND条件で検索する。
- limitは最大取得件数とする。
- 最新のレコードから取得する。

SQLの基本形：

```sql
SELECT timestamp, log_level, name, pid, msg
FROM sql_logs
WHERE ...
ORDER BY timestamp DESC, id DESC
LIMIT ?;
```

条件がない場合はWHERE句を付けないでください。

SQLパラメータは必ずバインドしてください。

文字列連結によるSQLの直接構築や、SQLインジェクションが発生する実装は禁止します。

検索条件に指定するlog_levelは、SQLiteに保存されている大文字文字列と一致させてください。

---

## 11. reduce

### 11.1 メソッド定義

```rust
pub fn reduce(
    &mut self,
    days: usize,
) -> Result<(), SqlLoggerError>;
```

### 11.2 動作仕様

DBに保存されているレコード数がmax_rec_numを超えている場合に、古いレコードを削除してください。

条件：

```text
now_row_num > max_rec_num
```

条件を満たさない場合は何もせず終了してください。

### 11.3 削除対象

現在時刻からdays日前より古いレコードを削除してください。

削除条件：

```sql
DELETE FROM sql_logs
WHERE timestamp < ?1;
```

削除基準時刻は以下で求めてください。

```text
現在のUnix time - days * 86400
```

注意事項：

- timestampはUnix timeなので、タイムゾーン変換せずに比較する。
- daysが0の場合は削除対象が現在時刻より古いレコードとなる。
- usizeからの変換でオーバーフローしないようにする。
- 不要なDELETEを実行しない。

### 11.4 削除後の処理

削除が成功したら、now_row_numを実際のDBの行数に更新してください。

更新にはCOUNT(*)を使用してください。

削除処理でエラーが発生した場合はエラーを返してください。

### 11.5 実行タイミング

reduceは自動実行しないでください。

利用者が明示的にreduceを呼び出した場合のみ実行してください。

outの内部からreduceを呼び出してはいけません。

---

## 12. out

### 12.1 メソッド定義

```rust
pub fn out(&mut self) -> Result<(), SqlLoggerError>;
```

### 12.2 動作仕様

メモリ上に蓄積されているrecodesをSQLiteに書き込んでください。

書き込み速度を重視し、複数レコードを1回のトランザクションで処理してください。

### 12.3 処理手順

以下の順番で処理してください。

1. recodesが空の場合は何もせず終了する。
2. SQLiteのトランザクションを開始する。
3. INSERT文を準備する。
4. recodes内の全レコードをINSERTする。
5. 全INSERTが成功した場合にCOMMITする。
6. COMMITが成功した場合のみrecodesを空にする。
7. now_row_numを書き込んだ件数分だけ増加させる。

### 12.4 INSERT文

```sql
INSERT INTO sql_logs (
    timestamp,
    log_level,
    name,
    pid,
    msg
)
VALUES (?1, ?2, ?3, ?4, ?5);
```

### 12.5 パフォーマンス要件

- 1回のoutにつき1回のトランザクションを使用する。
- トランザクション内でPrepared Statementを再利用する。
- レコードごとにトランザクションを開始しない。
- 不必要なDB接続の開閉をしない。
- INSERT時に不要なSELECTを実行しない。

### 12.6 エラー処理

重要なルール：

**DBへの書き込みが失敗した場合、recodesの内容を消去してはいけません。**

以下の場合はrecodesを保持してください。

- INSERT失敗
- COMMIT失敗
- トランザクション失敗

失敗した場合はエラーを呼び出し元に返してください。

トランザクションは適切にROLLBACKされるようにしてください。

now_row_numはCOMMIT成功後にのみ更新してください。

INSERT途中で失敗した場合、トランザクション内で実行済みのINSERTもDBに反映されないようにしてください。

COMMITが成功するまでは、メモリ上のrecodesを変更してはいけません。

---

## 13. SqlLoggerのログメソッド

### 13.1 メソッド定義

以下のメソッドを実装してください。

```rust
pub fn critical(&mut self, msg: &str) -> Result<(), SqlLoggerError>;

pub fn error(&mut self, msg: &str) -> Result<(), SqlLoggerError>;

pub fn warning(&mut self, msg: &str) -> Result<(), SqlLoggerError>;

pub fn info(&mut self, msg: &str) -> Result<(), SqlLoggerError>;

pub fn debug(&mut self, msg: &str) -> Result<(), SqlLoggerError>;
```

### 13.2 動作仕様

各メソッドは以下の処理を行ってください。

1. 設定されたlog_levelとメソッドのログレベルを比較する。
2. 設定されたlog_levelより低い優先度の場合は何もせず終了する。
3. 対応するSqlLogRecodeを生成する。
4. nameには `"logger"` を設定する。
5. recodesに追加する。
6. realtimeがtrueの場合はoutを呼び出す。
7. realtimeがfalseの場合はDBへ書き込まず、メモリに保持する。

### 13.3 ログレベル判定の厳守事項

ログレベルの判定は、必ず以下の条件を使用してください。

```rust
record_level.priority() >= self.log_level.priority()
```

**大小比較の向きを逆にしてはいけません。**

例えば、以下の設定の場合：

```text
log_level = INFO
```

動作は以下のとおりです。

| Method | Record Level | Result |
|---|---|---|
| critical | CRITICAL | 記録 |
| error | ERROR | 記録 |
| warning | WARNING | 記録 |
| info | INFO | 記録 |
| debug | DEBUG | 無視 |

以下の設定の場合：

```text
log_level = ERROR
```

動作は以下のとおりです。

| Method | Record Level | Result |
|---|---|---|
| critical | CRITICAL | 記録 |
| error | ERROR | 記録 |
| warning | WARNING | 無視 |
| info | INFO | 無視 |
| debug | DEBUG | 無視 |

### 13.4 フィルタリングのタイミング

ログレベルによるフィルタリングは、SqlLogRecodeを生成する前に実行してください。

無視されるログについては、以下の処理を行ってはいけません。

- SqlLogRecodeの生成
- recodesへの追加
- SQLiteへの書き込み
- now_row_numの変更

無視されるログは、何も記録せず正常終了してください。

### 13.5 realtime

realtimeの動作は以下のとおりです。

```text
realtime = false
```

ログメソッドを呼び出してもDBへ書き込まず、recodesに蓄積します。

```text
realtime = true
```

ログメソッドでレコードを追加した直後にoutを呼び出し、DBへ書き込みます。

realtime=trueでoutに失敗した場合は、エラーを呼び出し元へ返してください。

outが失敗した場合、追加したレコードを含むrecodes全体を保持してください。

---

## 14. initialize

### 14.1 メソッド定義

```rust
pub fn initialize(&mut self) -> Result<(), SqlLoggerError>;
```

### 14.2 動作仕様

既存のSQLiteデータベースを削除し、新しいデータベースを再作成してください。

以下の順番で実行してください。

1. メモリ上のrecodesを破棄する。
2. 現在のSQLite接続を安全に閉じる。
3. db_pathのSQLiteファイルを削除する。
4. 必要に応じてSQLite関連ファイル（WAL、SHM）も適切に処理する。
5. 新しいSQLiteデータベースを作成する。
6. テーブルを作成する。
7. インデックスを作成する。
8. SQLite接続を再確立する。
9. now_row_numを0に設定する。

initializeは既存のDBデータをすべて破棄する操作です。

処理中にエラーが発生した場合はエラーを返してください。

接続が失われたままになるなど、SqlLoggerが不整合な状態にならないようにしてください。

DBファイルの削除と再作成に失敗した場合も、接続状態を適切に管理してください。

---

## 15. Drop

### 15.1 動作仕様

SqlLoggerのインスタンスが破棄されるとき、必ずoutを呼び出してください。

実装例：

```rust
impl Drop for SqlLogger {
    fn drop(&mut self) {
        let _ = self.out();
    }
}
```

### 15.2 エラー処理

Drop内で発生したエラーは握りつぶしてください。

Dropからpanicを発生させてはいけません。

Drop内でログメソッドを呼び出してはいけません。

outが失敗した場合でも、Dropを安全に終了させてください。

---

## 16. エラー設計

SqlLoggerのメソッドは、適切にエラーを呼び出し元へ返してください。

独自エラー型を定義してください。

```rust
pub enum SqlLoggerError {
    Database(rusqlite::Error),
    Io(std::io::Error),
    TimeConversion,
}
```

実際のエラー型は必要に応じて調整してください。

要件：

- rusqliteのエラーを適切に変換する。
- ファイル操作のエラーを適切に変換する。
- 時刻変換に失敗した場合は適切にエラーを返す。
- エラー内容をDisplayで確認できるようにする。
- 元のエラー原因を可能な限り保持する。
- エラーを不用意に握りつぶさない。

例外としてDrop内ではエラーを握りつぶしてください。

---

## 17. SQLite接続管理

SQLiteへの接続はSqlLoggerのインスタンス生成時に確立し、インスタンスが破棄されるまで維持してください。

以下は禁止します。

- ログメソッドを呼ぶたびに接続する。
- outのたびに接続を開閉する。
- SELECTのたびに接続を開閉する。

ConnectionはSqlLogger内で管理してください。

initialize時のみ、安全な接続の再作成を許可します。

rusqliteのConnectionが持つスレッド安全性の制約を尊重してください。

SqlLoggerを無理にSendやSyncにしないでください。

---

## 18. 実装上の禁止事項

以下の実装は禁止します。

1. SQL文への直接的な文字列埋め込みによるユーザー入力の利用。
2. INSERTごとのトランザクション開始。
3. INSERT失敗時のrecodes消去。
4. COMMIT前のnow_row_num更新。
5. ログレベル判定を省略すること。
6. realtime=falseでの自動DB書き込み。
7. reduceの自動実行。
8. Drop内でのpanic。
9. 不必要な外部クレートの追加。
10. 仕様にない独自のログ出力やファイル出力機能の追加。
11. DBの既存データを通常のコンストラクタで削除すること。
12. timestampをISO 8601文字列としてDBに保存すること。
13. ログレベルの大小比較に文字列比較を使用すること。
14. ログレベルの比較条件を `record_level <= configured_log_level` とすること。
15. ログレベルのフィルタリング後にSqlLogRecodeを生成すること。

---

## 19. テスト仕様

実装後、必ずユニットテストと必要な統合テストを作成してください。

テストには一時ディレクトリを使用し、実際のユーザーデータを変更しないでください。

### 19.1 SqlLogLevel

- 各ログレベルが正しい文字列になること。
- Displayが正しいこと。
- 各ログレベルの数値が仕様どおりであること。
- ログレベルの大小比較が正しいこと。
- Serialize、Deserializeが正常に動作すること。
- JSON上でログレベルが大文字文字列になること。

優先順位のテストでは、以下を必ず確認してください。

```text
DEBUG < INFO < WARNING < ERROR < CRITICAL
```

また、判定関数が以下の結果になることを確認してください。

| Record Level | Configured Level | Result |
|---|---|---|
| CRITICAL | CRITICAL | true |
| ERROR | CRITICAL | false |
| CRITICAL | ERROR | true |
| ERROR | ERROR | true |
| WARNING | ERROR | false |
| INFO | WARNING | false |
| WARNING | WARNING | true |
| DEBUG | INFO | false |
| INFO | INFO | true |
| DEBUG | DEBUG | true |
| CRITICAL | DEBUG | true |

特に、比較条件を誤って `<=` にした場合にテストが失敗することを確認してください。

### 19.2 SqlLogRecode

- criticalがCRITICALになること。
- errorがERRORになること。
- warningがWARNINGになること。
- infoがINFOになること。
- debugがDEBUGになること。
- nameが正しく設定されること。
- msgが正しく設定されること。
- pidが現在のプロセスIDになること。
- timestampが有効なUnix timeになること。
- DisplayがISO 8601形式になること。
- DebugがISO 8601形式のtimestampを表示すること。
- JSON変換時にtimestampが数値になること。
- JSON変換時にlevelが大文字文字列になること。

### 19.3 SqlLoggerの生成

- 存在しない親ディレクトリが作成されること。
- DBファイルが作成されること。
- テーブルが作成されること。
- インデックスが作成されること。
- 新規DBのnow_row_numが0になること。
- 既存DBのnow_row_numが実際の行数になること。
- 既存DBのデータが保持されること。

### 19.4 ログレベル

- INFO設定時にDEBUGが無視されること。
- INFO設定時にINFOが記録されること。
- INFO設定時にWARNINGが記録されること。
- CRITICAL設定時にERRORが無視されること。
- DEBUG設定時にすべてのレベルが記録されること。
- ERROR設定時にWARNING、INFO、DEBUGが無視されること。
- 無視されたログがrecodesに追加されないこと。
- 無視されたログによってDBの行数が増加しないこと。
- 無視されたログによってnow_row_numが変更されないこと。

**比較条件が逆になっていないことを必ずテストしてください。**

### 19.5 realtime

realtime=falseの場合：

- ログメソッドを呼び出してもDBに書き込まれないこと。
- recodesにレコードが蓄積されること。

realtime=trueの場合：

- ログメソッドの呼び出し後にDBへ書き込まれること。
- 成功後にrecodesが空になること。
- now_row_numが正しく増加すること。

### 19.6 out

- 複数レコードを一括INSERTできること。
- 1回のトランザクションで書き込まれること。
- 書き込み成功後にrecodesが空になること。
- 書き込み成功後にnow_row_numが増えること。
- 書き込み失敗時にrecodesが保持されること。
- 書き込み失敗時にnow_row_numが変更されないこと。
- INSERT途中の失敗で一部のレコードだけがDBに保存されないこと。
- recodesが空の場合は正常終了すること。

### 19.7 検索

get_recs：

- 最新1000件を取得できること。
- DESC順で取得できること。
- timestampが同一のレコードでもidによって順序が安定すること。
- 1000件未満の場合も正常に動作すること。

get_recs_with_limit：

- 指定した件数を取得できること。
- limit=0で空のVecになること。

get_recs_with：

- nameによる検索ができること。
- log_levelによる検索ができること。
- nameとlog_levelのAND検索ができること。
- 両方Noneの場合は全件検索できること。
- limitが正しく機能すること。
- 最新順で取得できること。
- SQLパラメータが正しくバインドされること。

### 19.8 reduce

- now_row_numがmax_rec_num以下の場合は削除しないこと。
- now_row_numがmax_rec_numを超えた場合に古いレコードを削除すること。
- daysに基づいて削除対象が正しく判定されること。
- 削除後のnow_row_numがDBの実際の行数と一致すること。
- 新しいレコードが残ること。
- days=0の場合に現在時刻より古いレコードが削除対象になること。
- reduceを呼び出さない限り削除されないこと。

### 19.9 initialize

- 既存DBを削除できること。
- テーブルが再作成されること。
- インデックスが再作成されること。
- now_row_numが0になること。
- SQLite接続が再確立されること。
- initialize後もログを正常に記録できること。

### 19.10 Drop

- Drop時に未書き込みレコードがDBに保存されること。
- Drop内のエラーによってpanicが発生しないこと。

---

## 20. 実装作業の進め方

以下の順序で作業してください。

### Step 1: プロジェクト確認

既存のCargo.toml、ソースコード、ディレクトリ構成を確認してください。

既存プロジェクトの設計や命名規則がある場合は、それを尊重してください。

### Step 2: 依存関係確認

必要なクレートとライセンスを確認してください。

不要な依存関係を追加しないでください。

### Step 3: 型定義

以下を実装してください。

- SqlLogLevel
- SqlLogRecode
- SqlLoggerError

この段階でログレベルの数値と比較ロジックを確定させてください。

### Step 4: SQLite処理

以下を実装してください。

- DB接続
- テーブル作成
- インデックス作成
- レコード数取得
- INSERT
- SELECT
- DELETE

### Step 5: SqlLogger

以下を実装してください。

- new
- new_with
- out
- get_recs
- get_recs_with_limit
- get_recs_with
- reduce
- initialize
- critical
- error
- warning
- info
- debug
- Drop

### Step 6: テスト

第19章のテストを実装してください。

特にログレベルの大小比較について、すべての組み合わせで正しい判定になることを確認してください。

### Step 7: 品質確認

以下を実行してください。

```bash
cargo fmt --all -- --check

cargo clippy --all-targets --all-features -- -D warnings

cargo test
```

すべて成功することを確認してください。

### Step 8: 最終報告

以下を報告してください。

1. 実装した機能
2. 作成・変更したファイル
3. 追加した外部クレートとライセンス
4. テスト結果
5. Clippyの結果
6. 仕様から変更した箇所がある場合は、その理由

---

## 21. 最終的な実装完了条件

以下をすべて満たした場合に実装完了としてください。

- SqlLogLevelが正常に動作する。
- SqlLogLevelの数値がDEBUG=1、INFO=2、WARNING=3、ERROR=4、CRITICAL=5になっている。
- ログレベルの大小比較が正しい。
- 比較条件が `record_level >= configured_log_level` になっている。
- SqlLogRecodeが正常に生成できる。
- Unix timeを正しく保持できる。
- DisplayとDebugでISO 8601表示できる。
- SqlLoggerがSQLiteに接続できる。
- ログレベルによるフィルタリングが正常に動作する。
- フィルタリングがSqlLogRecodeの生成前に実行される。
- realtimeの切り替えが正常に動作する。
- 複数レコードをトランザクションで一括INSERTできる。
- SELECTで最新のログを取得できる。
- nameとlog_levelで検索できる。
- reduceで古いデータを削除できる。
- initializeでDBを再作成できる。
- Dropで未書き込みログを保存できる。
- エラー発生時にログバッファが不意に消失しない。
- cargo fmtが成功する。
- cargo clippyが成功する。
- cargo testが成功する。

**重要：**

実装の簡略化を理由に、トランザクション処理、エラー処理、テストを省略してはいけません。

仕様に記載されていない動作を独自判断で追加せず、仕様を満たす最小限かつ堅牢な実装にしてください。