# Codex Implementation Specification: AsyncHttpClient (Rust)

## 1. プロジェクト概要

Rust と `reqwest` を使用して、簡易的な非同期 HTTP Client ライブラリを実装してください。

本ライブラリは、JSON を使用する REST API との通信を主な用途とします。

GET / POST / PUT / DELETE の HTTP メソッドを個別の関数として提供し、API 通信に必要な共通設定を Client インスタンスに保持します。

### 1.1 基本方針

- Rust で実装する。
- HTTP 通信には `reqwest` を使用する。
- 非同期処理には `async / await` を使用する。
- JSON のシリアライズ・デシリアライズには `serde` / `serde_json` を使用する。
- レスポンス JSON はジェネリック型を使用して任意の構造体に変換できるようにする。
- エラーは `HttpClientError` に統一する。
- HTTP リトライ機能は実装しない。
- 必要以上の機能を追加しない。

## 2. 重要な設計方針

### 2.1 URL の扱い

すべての HTTP メソッドで、完全な URL を引数として受け取る。

例：

- `https://api.example.com/users`
- `https://api.example.com/users/1`
- `https://api.example.com/users?page=1&limit=10`

Client インスタンスには Base URL を保持しない。

URL の組み立て機能は本ライブラリの HTTP 通信機能とは分離し、別機能として実装する。

HTTP Client は渡された完全な URL を使用して通信する。

### 2.2 Client インスタンス

HTTP 通信に必要な共通設定は `HttpClient` インスタンスに保持する。

保持する設定：

- HTTP 通信を行うための `reqwest::Client`
- Authorization ヘッダー
- 任意の共通 HTTP ヘッダー
- タイムアウト設定

一度生成した Client インスタンスを複数の API 通信で再利用できるようにする。

### 2.3 HTTP メソッド

以下の4つを個別の関数として実装する。

- `get`
- `post`
- `put`
- `delete`

### 2.4 JSON

- POST / PUT のリクエストボディは JSON とする。
- GET / DELETE は通常、リクエストボディを使用しない。
- レスポンスボディは JSON として解析する。
- レスポンス型はジェネリック型 `T` を使用する。

## 3. 使用するライブラリ

以下のライブラリを使用する。

| Crate | 用途 |
|---|---|
| reqwest | HTTP 通信 |
| tokio | 非同期ランタイム |
| serde | JSON シリアライズ・デシリアライズ |
| serde_json | JSON 処理 |
| thiserror | エラー型の実装 |

### 3.1 reqwest の設定

- 非同期機能を有効にする。
- JSON 機能を有効にする。
- TLS は reqwest の標準的な安全な設定を利用する。
- HTTP Client は生成時に一度だけ構築し、使い回す。

依存クレートは必要最小限にすること。

## 4. HttpClient の仕様

### 4.1 構造体

以下の構成を基本とする。

```rust
pub struct HttpClient {
    client: reqwest::Client,
}
```

HTTP 通信に必要な設定は `reqwest::Client` の生成時に設定する。

認証ヘッダーや共通ヘッダーは `reqwest::ClientBuilder` に設定し、リクエストごとに共通設定を適用する。

### 4.2 コンストラクタ

以下のコンストラクタを実装する。

#### new

```rust
pub fn new() -> Result<Self, HttpClientError>
```

デフォルト設定で Client を生成する。

デフォルト設定：

- タイムアウト：30秒
- Authorization：未設定
- その他の共通ヘッダー：未設定

#### new_with_timeout

```rust
pub fn new_with_timeout(
    timeout: std::time::Duration,
) -> Result<Self, HttpClientError>
```

指定されたタイムアウトで Client を生成する。

### 4.3 Authorization ヘッダー

Authorization ヘッダーに認証トークンを設定できるようにする。

認証トークンは必須ではない。

以下の形式に対応する。

```http
Authorization: Bearer YOUR_TOKEN
```

ただし、Bearer 形式に限定しない。

呼び出し側が Authorization ヘッダーの値を自由に指定できるようにする。

例：

```rust
let mut headers = reqwest::header::HeaderMap::new();

headers.insert(
    reqwest::header::AUTHORIZATION,
    "Bearer YOUR_TOKEN".parse().unwrap(),
);
```

認証ヘッダーは Client 生成時に設定できるようにする。

#### new_with_options

共通ヘッダーなどをまとめて設定できるコンストラクタを実装する。

```rust
pub fn new_with_options(
    timeout: std::time::Duration,
    headers: reqwest::header::HeaderMap,
) -> Result<Self, HttpClientError>
```

`headers` には Authorization を含め、任意の HTTP ヘッダーを設定できる。

`new()` および `new_with_timeout()` は内部的に共通の生成処理を利用する。

ヘッダーを設定する際は、HeaderMap による型検証を利用すること。

不正なヘッダー値を無理に文字列から変換しないこと。

## 5. HTTP 通信 API

すべての関数を非同期関数として実装する。

すべての関数で URL は完全な URL を受け取る。

レスポンスはジェネリック型 `T` に変換する。

共通の戻り値は以下とする。

```rust
Result<T, HttpClientError>
```

### 5.1 GET

```rust
pub async fn get<T>(
    &self,
    url: &str,
) -> Result<T, HttpClientError>
where
    T: serde::de::DeserializeOwned;
```

仕様：

1. 指定された URL に GET リクエストを送信する。
2. 共通ヘッダーを付与する。
3. タイムアウト設定を適用する。
4. HTTP ステータスコードを確認する。
5. 2xx 以外の場合は `HttpClientError` を返す。
6. 2xx の場合はレスポンスボディを JSON として解析する。
7. 指定されたジェネリック型 `T` に変換する。

GET のクエリパラメータは URL に含めて渡す。

GET 専用のクエリパラメータ構造体は実装しない。

### 5.2 POST

```rust
pub async fn post<T, B>(
    &self,
    url: &str,
    body: &B,
) -> Result<T, HttpClientError>
where
    T: serde::de::DeserializeOwned,
    B: serde::Serialize + ?Sized;
```

仕様：

1. 指定された URL に POST リクエストを送信する。
2. リクエストボディを JSON にシリアライズする。
3. Content-Type に application/json を設定する。
4. 共通ヘッダーを付与する。
5. タイムアウト設定を適用する。
6. HTTP ステータスコードを確認する。
7. 2xx 以外の場合は `HttpClientError` を返す。
8. 2xx の場合はレスポンスボディを JSON として解析する。
9. 指定されたジェネリック型 `T` に変換する。

### 5.3 PUT

```rust
pub async fn put<T, B>(
    &self,
    url: &str,
    body: &B,
) -> Result<T, HttpClientError>
where
    T: serde::de::DeserializeOwned,
    B: serde::Serialize + ?Sized;
```

仕様：

POST と同じ JSON 処理を行い、HTTP メソッドを PUT とする。

### 5.4 DELETE

```rust
pub async fn delete<T>(
    &self,
    url: &str,
) -> Result<T, HttpClientError>
where
    T: serde::de::DeserializeOwned;
```

仕様：

1. 指定された URL に DELETE リクエストを送信する。
2. 共通ヘッダーを付与する。
3. タイムアウト設定を適用する。
4. HTTP ステータスコードを確認する。
5. 2xx 以外の場合は `HttpClientError` を返す。
6. 2xx の場合はレスポンスボディを JSON として解析する。
7. 指定されたジェネリック型 `T` に変換する。

DELETE のリクエストボディは実装しない。

## 6. HttpClientError の仕様

エラーの種類を呼び出し側で判別できるようにする。

`thiserror` を利用して実装する。

### 6.1 エラー定義

以下を基本構成とする。

```rust
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {

    #[error("HTTP status error: {status}")]
    HttpStatus {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("Connection error: {0}")]
    Connection(#[source] reqwest::Error),

    #[error("Timeout error: {0}")]
    Timeout(#[source] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("JSON deserialization error: {0}")]
    JsonDeserialization(serde_json::Error),

    #[error("HTTP request error: {0}")]
    Request(#[source] reqwest::Error),
}
```

必要に応じて、Client 生成時のエラーも適切に分類できるようにする。

### 6.2 エラー分類

以下の区別を必ず実装する。

| エラー | 説明 |
|---|---|
| HttpStatus | HTTP ステータスコードが2xx以外 |
| Connection | 接続関連のエラー |
| Timeout | タイムアウト |
| JsonSerialization | リクエスト JSON の変換失敗 |
| JsonDeserialization | レスポンス JSON の変換失敗 |
| Request | その他の HTTP リクエストエラー |

### 6.3 エラー判定関数

`HttpClientError` に以下のメソッドを実装する。

```rust
impl HttpClientError {

    pub fn is_timeout(&self) -> bool;

    pub fn is_connection_error(&self) -> bool;

    pub fn is_json_error(&self) -> bool;

    pub fn is_http_error(&self) -> bool;

    pub fn is_auth_error(&self) -> bool;

    pub fn requires_reauthentication(&self) -> bool;

    pub fn status_code(&self) -> Option<reqwest::StatusCode>;
}
```

各関数の仕様：

#### is_timeout

Timeout の場合に true を返す。

#### is_connection_error

Connection の場合に true を返す。

#### is_json_error

JsonSerialization または JsonDeserialization の場合に true を返す。

#### is_http_error

HttpStatus の場合に true を返す。

#### is_auth_error

認証・認可に関連する HTTP ステータスの場合に true を返す。

対象：

- 401 Unauthorized
- 403 Forbidden

#### requires_reauthentication

再認証が必要な HTTP ステータスの場合に true を返す。

対象：

- 401 Unauthorized

403 Forbidden は権限不足として扱い、再認証が必要とは判定しない。

#### status_code

HttpStatus の場合はステータスコードを返す。

それ以外のエラーでは None を返す。

### 6.4 エラー分類の実装上の注意

reqwest のエラーを分類する際は、必ず `reqwest::Error` の情報を確認すること。

- `is_timeout()` を利用してタイムアウトを判定する。
- `is_connect()` を利用して接続エラーを判定する。
- その他は Request として分類する。

HTTP ステータスエラーと通信エラーを混同しないこと。

レスポンスのステータスコードを確認する際は `error_for_status()` の戻り値だけに依存せず、必要なレスポンス情報を取得してから分類すること。

HTTP エラーの場合、レスポンスボディを文字列として取得し、`HttpStatus` に保存する。

## 7. HTTP レスポンスの処理

### 7.1 ステータスコード

HTTP ステータスコードが 200〜299 の場合のみ成功とする。

それ以外は HttpStatus エラーとする。

### 7.2 JSON デシリアライズ

成功レスポンスの JSON をジェネリック型 `T` に変換する。

```rust
let response: UserResponse =
    client.get("https://api.example.com/users/1").await?;
```

呼び出し側で型を指定できるようにする。

### 7.3 空のレスポンスボディ

HTTP 204 No Content など、成功ステータスでレスポンスボディが空の場合がある。

JSON デシリアライズを実行するとエラーになるため、以下の方針とする。

- 基本 API は JSON レスポンスを返す API を対象とする。
- 空のレスポンスボディは JSON デシリアライズエラーとして扱う。
- 呼び出し側が空レスポンスを扱う必要がある場合に備え、将来的に `T = ()` などの対応を検討できる設計にする。
- 現段階では不要な追加 API を実装しない。

## 8. リトライ処理

リトライ処理は一切実装しない。

通信エラー、タイムアウト、HTTP ステータスエラーが発生した場合、再試行せずにエラーを返す。

reqwest 自体の接続管理機能と、アプリケーションレベルのリトライ処理を混同しないこと。

## 9. タイムアウト

### 9.1 デフォルト

デフォルトタイムアウトは30秒とする。

### 9.2 カスタマイズ

`new_with_timeout()` または `new_with_options()` で変更できるようにする。

タイムアウトが発生した場合は `HttpClientError::Timeout` として返す。

## 10. 共通ヘッダー

Client 生成時に任意の HTTP ヘッダーを設定できるようにする。

例：

```rust
let mut headers = reqwest::header::HeaderMap::new();

headers.insert(
    reqwest::header::AUTHORIZATION,
    "Bearer YOUR_TOKEN".parse().unwrap(),
);

headers.insert(
    reqwest::header::ACCEPT,
    "application/json".parse().unwrap(),
);
```

共通ヘッダーはすべての HTTP リクエストに適用する。

POST / PUT の JSON リクエストでは Content-Type を application/json とする。

Content-Type が共通ヘッダーにも指定されている場合は、JSON リクエストに必要な Content-Type が確実に適用されるようにする。

Authorization ヘッダーを必須にしない。

## 11. URL 生成機能について

URL 生成機能は本 HTTP Client の実装範囲外とする。

以下の機能は実装しない。

- Base URL の保持
- パスの結合
- クエリパラメータの組み立て
- URL の自動補完

HTTP Client は呼び出し側から渡された URL をそのまま使用する。

## 12. 使用例

以下のような利用方法を想定する。

### 12.1 Client の生成

```rust
use std::time::Duration;
use reqwest::header::{HeaderMap, AUTHORIZATION, ACCEPT};

let mut headers = HeaderMap::new();

headers.insert(
    AUTHORIZATION,
    "Bearer YOUR_TOKEN".parse().unwrap(),
);

headers.insert(
    ACCEPT,
    "application/json".parse().unwrap(),
);

let client = HttpClient::new_with_options(
    Duration::from_secs(30),
    headers,
)?;
```

### 12.2 GET

```rust
#[derive(Debug, serde::Deserialize)]
struct User {
    id: u64,
    name: String,
}

let user: User = client
    .get("https://api.example.com/users/1")
    .await?;
```

### 12.3 POST

```rust
#[derive(serde::Serialize)]
struct CreateUser {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct CreateUserResponse {
    id: u64,
    name: String,
}

let body = CreateUser {
    name: "test".to_string(),
};

let response: CreateUserResponse = client
    .post("https://api.example.com/users", &body)
    .await?;
```

### 12.4 PUT

```rust
let response: User = client
    .put("https://api.example.com/users/1", &body)
    .await?;
```

### 12.5 DELETE

```rust
let response: serde_json::Value = client
    .delete("https://api.example.com/users/1")
    .await?;
```

### 12.6 エラー判定

```rust
match client.get::<User>(
    "https://api.example.com/users/1"
).await {
    Ok(user) => {
        println!("{:?}", user);
    }

    Err(err) => {
        if err.requires_reauthentication() {
            println!("再認証が必要です");
        } else if err.is_timeout() {
            println!("通信がタイムアウトしました");
        } else if err.is_connection_error() {
            println!("接続エラーが発生しました");
        } else if err.is_json_error() {
            println!("JSON の処理に失敗しました");
        } else if err.is_http_error() {
            println!("HTTP エラー: {:?}", err.status_code());
        } else {
            println!("その他のエラー: {}", err);
        }
    }
}
```

## 13. テスト仕様

外部 API に依存しないテストを実装する。

ローカルのモック HTTP サーバーなどを使用して検証する。

### 13.1 Client 生成

- デフォルト設定で生成できること。
- タイムアウトを指定して生成できること。
- 共通ヘッダーを指定して生成できること。
- Authorization ヘッダーを指定できること。
- Authorization 未設定でも生成できること。

### 13.2 GET

- GET リクエストが送信されること。
- JSON レスポンスが指定した構造体に変換されること。
- 共通ヘッダーが送信されること。
- 2xx 以外の場合に HttpStatus が返ること。

### 13.3 POST

- POST リクエストが送信されること。
- リクエストボディが正しく JSON に変換されること。
- Content-Type が application/json であること。
- JSON レスポンスが指定した構造体に変換されること。
- 2xx 以外の場合に HttpStatus が返ること。

### 13.4 PUT

- PUT リクエストが送信されること。
- リクエストボディが正しく JSON に変換されること。
- JSON レスポンスが指定した構造体に変換されること。
- 2xx 以外の場合に HttpStatus が返ること。

### 13.5 DELETE

- DELETE リクエストが送信されること。
- JSON レスポンスが指定した構造体に変換されること。
- 2xx 以外の場合に HttpStatus が返ること。

### 13.6 エラー分類

以下をそれぞれテストする。

- HTTP 400
- HTTP 401
- HTTP 403
- HTTP 404
- HTTP 500
- 接続エラー
- タイムアウト
- JSON シリアライズエラー
- JSON デシリアライズエラー

特に以下の判定が正しいこと。

| 状態 | is_auth_error | requires_reauthentication |
|---|---|---|
| 401 | true | true |
| 403 | true | false |
| 404 | false | false |
| 500 | false | false |

## 14. 実装時の制約

以下を必ず守ること。

1. GET / POST / PUT / DELETE はそれぞれ独立した公開関数として実装する。
2. すべての HTTP メソッドで完全な URL を受け取る。
3. Base URL は保持しない。
4. Authorization は任意設定とする。
5. 任意の共通 HTTP ヘッダーを設定できるようにする。
6. レスポンスはジェネリック型で受け取る。
7. すべてのエラーを `HttpClientError` に統一する。
8. エラー種別を判定するメソッドを実装する。
9. HTTP 401 と HTTP 403 を区別する。
10. デフォルトタイムアウトは30秒とする。
11. リトライ機能を実装しない。
12. URL 生成機能を実装しない。
13. 指定されていない機能を独断で追加しない。
14. 不必要な外部クレートを追加しない。
15. 公開 API の仕様を独断で変更しない。

## 15. 実装手順

以下の順番で実装すること。

1. Cargo.toml と依存クレートを設定する。
2. HttpClientError を実装する。
3. HttpClient の構造体とコンストラクタを実装する。
4. 共通ヘッダーとタイムアウトの設定を実装する。
5. GET を実装する。
6. POST を実装する。
7. PUT を実装する。
8. DELETE を実装する。
9. エラー判定メソッドを実装する。
10. 単体テスト・統合テストを実装する。
11. cargo fmt を実行する。
12. cargo clippy を実行する。
13. cargo test を実行する。
14. コンパイルエラー・警告・テスト失敗を修正する。

## 16. 完了条件

以下をすべて満たした場合に実装完了とする。

- GET / POST / PUT / DELETE が正常に動作する。
- 完全な URL を使用して通信できる。
- Authorization ヘッダーを設定できる。
- 任意の共通 HTTP ヘッダーを設定できる。
- JSON リクエストを送信できる。
- JSON レスポンスを任意の構造体に変換できる。
- HTTP ステータスエラーを判定できる。
- 401 と 403 を区別できる。
- 通信エラーを判定できる。
- JSON エラーを判定できる。
- タイムアウトを判定できる。
- デフォルトタイムアウトが30秒である。
- すべてのテストが成功する。
- cargo fmt が成功する。
- cargo clippy が成功する。

実装完了後は、以下を報告すること。

- 実装した機能
- 作成・変更したファイル
- 公開 API の使用方法
- テスト結果
- cargo clippy の結果
- 未実装事項や制約

以上の仕様に従って実装してください。