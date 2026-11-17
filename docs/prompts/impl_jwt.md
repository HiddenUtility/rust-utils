# Codex Implementation Specification: JWT Utility Library (Rust)

## 1. Project Overview

RustでJWT（JSON Web Token）を生成・検証・読み込みするユーティリティライブラリを実装してください。

本ライブラリは、JWTの署名付きトークンの生成、秘密鍵の生成、Payloadのシリアライズ・デシリアライズ、および有効期限の検証機能を提供します。

### 1.1 基本仕様

| 項目 | 仕様 |
|---|---|
| 言語 | Rust |
| JWT署名アルゴリズム | HS256 |
| UUID | UUID v4 |
| Payload形式 | JSON |
| シリアライズ | Serde |
| デシリアライズ | Serde |
| 時刻表現 | Unix Timestamp（秒） |
| 秘密鍵 | 暗号学的に安全な乱数から生成 |
| エラー処理 | Result型 |
| 非同期処理 | 使用しない |

### 1.2 実装方針

以下の方針を厳守してください。

1. 仕様に記載されていない機能を独自に追加しないこと。
2. 不必要な外部クレートを追加しないこと。
3. エラーの種類を明確に区別すること。
4. JWTの署名検証にはHS256のみを使用すること。
5. 秘密鍵の生成には暗号学的に安全な乱数生成器を使用すること。
6. UUIDはUUID v4を使用すること。
7. PayloadはSerdeによってシリアライズ・デシリアライズできること。
8. 既存のJWT標準仕様と互換性を持たせること。
9. コンパイル可能で、テストが実行できる状態まで実装すること。

---

## 2. Dependencies

以下のクレートを使用してください。

| Crate | 用途 |
|---|---|
| jsonwebtoken | JWTの生成・署名・検証 |
| serde | Serialize / Deserialize |
| serde_json | JSONのシリアライズ・デシリアライズ |
| uuid | UUID v4の生成 |
| rand | 暗号学的に安全な乱数生成 |

### 2.1 Cargo.toml

必要な機能を有効にして依存関係を定義してください。

- `serde`：derive機能を有効にする。
- `uuid`：v4機能を有効にする。
- `jsonwebtoken`：HS256による署名・検証を使用する。
- `rand`：OSの暗号学的乱数生成器を使用する。

クレートのバージョンは、実装時点で利用可能な安定版を使用してください。

外部クレートを追加する場合は、追加する必要性を説明してください。

---

## 3. JWT Secret Key

JWTの署名・検証に使用する秘密鍵を生成する関数を実装してください。

### 3.1 Secret Key Generation

関数名：

`generate_secret_key`

シグネチャ：

```rust
pub fn generate_secret_key() -> Result<String, JwtError>
```

仕様：

1. 暗号学的に安全な乱数生成器を使用する。
2. 32バイト（256ビット）以上のランダムデータを生成する。
3. 生成したバイナリデータを16進数文字列に変換する。
4. 生成した文字列を秘密鍵として返す。
5. 乱数生成に失敗した場合はエラーを返す。
6. 秘密鍵は毎回異なる値になること。

生成する秘密鍵の形式：

- 64文字以上の16進数文字列。
- 大文字・小文字は問わない。
- 暗号学的に安全な乱数から生成すること。

注意：

- UUIDを単純に秘密鍵として使用しないこと。
- 固定文字列や時刻などから秘密鍵を生成しないこと。
- 生成した秘密鍵をログに出力しないこと。

---

## 4. JWT Payload

JWTのPayloadを表現する構造体を実装してください。

構造体名：

`JwtPayload`

### 4.1 Struct Definition

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtPayload {
    /// 発行者
    pub iss: Option<String>,

    /// 主体（ユーザーIDなど）
    pub sub: Option<String>,

    /// 対象者
    pub aud: Option<String>,

    /// 有効期限（Unix Timestamp 秒）
    pub exp: u64,

    /// 有効開始日時（Unix Timestamp 秒）
    pub nbf: u64,

    /// 発行日時（Unix Timestamp 秒）
    pub iat: u64,

    /// JWT ID（UUID v4）
    pub jti: String,

    /// アプリケーション独自のデータ
    pub data: Option<Value>,
}
```

### 4.2 Payload Attributes

JWT標準クレームとして以下を定義してください。

| フィールド | 意味 | 必須 |
|---|---|---|
| iss | 発行者（Issuer） | 任意 |
| sub | 主体（Subject） | 任意 |
| aud | 対象者（Audience） | 任意 |
| exp | 有効期限（Expiration Time） | 必須 |
| nbf | 有効開始日時（Not Before） | 必須 |
| iat | 発行日時（Issued At） | 必須 |
| jti | JWT ID（JWT ID） | 必須 |
| data | アプリケーション独自データ | 任意 |

時刻はすべてUnix Timestampの秒単位で扱ってください。

### 4.3 jti

`jti` はJWTを一意に識別するためのIDです。

以下の仕様にしてください。

- UUID v4を使用する。
- Payload生成時に自動生成する。
- 呼び出し側から指定する必要はない。
- 生成したUUIDを文字列として保持する。

### 4.4 data

`data` はアプリケーション固有の任意のJSONデータを格納するためのフィールドです。

`serde_json::Value` を使用してください。

以下のようなデータを格納できること。

```json
{
    "user_id": 123,
    "username": "test_user",
    "role": "admin",
    "permissions": ["read", "write"]
}
```

`data` が `None` の場合は、JSONの `null` として扱うのではなく、JWTのJSON Payloadからフィールド自体を省略してください。

そのため、必要に応じてSerdeの `skip_serializing_if` を使用してください。

---

## 5. Payload Constructor

Payloadを生成するためのコンストラクタを実装してください。

関数名：

`new`

シグネチャ：

```rust
pub fn new(
    exp: u64,
    nbf: u64,
    data: Option<Value>,
) -> Self
```

仕様：

1. `exp` を有効期限として設定する。
2. `nbf` を有効開始日時として設定する。
3. `iat` は現在時刻から自動生成する。
4. `jti` はUUID v4から自動生成する。
5. `iss` は `None` に設定する。
6. `sub` は `None` に設定する。
7. `aud` は `None` に設定する。
8. `data` は引数の値を設定する。

現在時刻はUnix Timestampの秒単位で取得してください。

システム時刻の取得に失敗した場合の扱いも明確にし、パニックを発生させない実装にしてください。必要であればコンストラクタを `Result<Self, JwtError>` にしてください。

---

## 6. JWT Serialization

PayloadをJWT形式の文字列にシリアライズする関数を実装してください。

関数名：

`encode_jwt`

シグネチャ：

```rust
pub fn encode_jwt(
    payload: &JwtPayload,
    secret_key: &str,
) -> Result<String, JwtError>
```

仕様：

1. `JwtPayload` をJSONにシリアライズする。
2. JWT標準の形式に従ってトークンを生成する。
3. 署名アルゴリズムにはHS256を使用する。
4. `secret_key` を署名鍵として使用する。
5. 生成したJWT文字列を返す。
6. シリアライズや署名処理に失敗した場合はエラーを返す。
7. 空文字列の秘密鍵は拒否する。

JWT Headerのアルゴリズムは必ずHS256にしてください。

他のアルゴリズムを使用した署名は実装しないでください。

注意：

JWTの生成処理では有効期限の自動更新やPayloadの書き換えを行わないでください。

---

## 7. JWT Deserialization and Verification

JWT文字列からPayloadを読み込む関数を実装してください。

関数名：

`decode_jwt`

シグネチャ：

```rust
pub fn decode_jwt(
    token: &str,
    secret_key: &str,
) -> Result<JwtPayload, JwtError>
```

### 7.1 処理内容

以下の順序で処理してください。

1. 秘密鍵が空文字列でないことを確認する。
2. JWT Headerの形式を確認する。
3. JWTの形式が正しいことを確認する。
4. 署名アルゴリズムがHS256であることを確認する。
5. 秘密鍵を使用して署名を検証する。
6. 署名検証に成功した場合、PayloadをJSONからデシリアライズする。
7. Payloadの有効期限を検証する。
8. Payloadの有効開始日時を検証する。
9. すべての検証に成功した場合、`JwtPayload` を返す。

### 7.2 Signature Verification

署名の検証には `jsonwebtoken` クレートを使用してください。

以下の場合は署名検証エラーとして扱ってください。

- 秘密鍵が間違っている。
- JWTの署名が改ざんされている。
- 署名が正しくない。

秘密鍵の不一致と署名の改ざんは、どちらも `InvalidSignature` として扱って構いません。

### 7.3 Error Distinction

以下のエラーを区別できるようにしてください。

- JWTの形式が不正。
- JWT Headerのパース失敗。
- JWT Payloadのパース失敗。
- 署名検証失敗。
- 有効期限切れ。
- 有効開始日時前。
- 秘密鍵の不正。

特に、秘密鍵の間違いによる署名検証失敗とJSONのパース失敗を同じエラーにしないでください。

`jsonwebtoken` のエラー型を適切に判定して独自のエラー型に変換してください。

JWT HeaderのパースとPayloadのパースは区別して処理してください。

---

## 8. Payload Validation

Payloadの有効期限および有効開始日時を検証する関数を実装してください。

関数名：

`validate_time`

シグネチャ：

```rust
pub fn validate_time(
    payload: &JwtPayload,
) -> Result<(), JwtError>
```

### 8.1 Expiration Validation

`exp` は有効期限を表します。

以下の場合は有効期限切れと判定してください。

```rust
now >= payload.exp
```

### 8.2 Not Before Validation

`nbf` は有効開始日時を表します。

以下の場合は有効開始日時前と判定してください。

```rust
now < payload.nbf
```

### 8.3 Valid Condition

以下の条件をすべて満たした場合に有効とします。

```rust
payload.nbf <= now && now < payload.exp
```

### 8.4 Time Handling

- 現在時刻はUnix Timestampの秒単位で取得する。
- 時刻の比較は秒単位で行う。
- タイムゾーンによる影響を受けないこと。
- 有効期限の判定に猶予時間（leeway）を設けないこと。
- システム時刻の取得に失敗した場合はエラーを返す。

### 8.5 Validation Order

以下の順序で検証してください。

1. 有効期限（exp）
2. 有効開始日時（nbf）

両方に問題がある場合は、有効期限切れを優先して返してください。

### 8.6 decode_jwtとの連携

`decode_jwt` の内部でも必ず有効期限と有効開始日時を検証してください。

個別の `validate_time` 関数も公開し、すでに取得済みのPayloadに対して再検証できるようにしてください。

JWTの読み込み時に検証を省略するオプションは設けないでください。

`jsonwebtoken` のValidation設定では以下を明示してください。

- アルゴリズム：HS256のみ。
- `validate_exp`：true。
- `validate_nbf`：true。
- `leeway`：0。
- 必須クレーム：exp、nbf。

また、ライブラリの標準検証に加え、独自の `validate_time` を使用してエラーの種類と判定条件を統一してください。

---

## 9. Error Handling

独自のエラー型 `JwtError` を実装してください。

エラー型には `thiserror` などを使用して構いませんが、不要な依存関係を増やさないでください。

### 9.1 JwtError

以下のエラーを定義してください。

```rust
#[derive(Debug)]
pub enum JwtError {
    /// 秘密鍵が不正、または空
    InvalidKey,

    /// JWTの形式が不正
    InvalidFormat,

    /// JWT Headerのパース失敗
    InvalidHeader,

    /// JWT Payloadのパース失敗
    InvalidPayload,

    /// 署名検証失敗
    InvalidSignature,

    /// 有効期限切れ
    Expired,

    /// 有効開始日時前
    NotYetValid,

    /// 現在時刻の取得失敗
    TimeError,

    /// 乱数生成失敗
    RandomGenerationError,

    /// JWTのシリアライズ・署名生成失敗
    EncodingError,

    /// その他のJWT関連エラー
    Other(String),
}
```

実際の実装に合わせて必要な内部情報を保持して構いません。

### 9.2 Error Mapping

`jsonwebtoken` のエラーを `JwtError` に変換してください。

| jsonwebtoken側のエラー | JwtError |
|---|---|
| InvalidSignature | InvalidSignature |
| ExpiredSignature | Expired |
| ImmatureSignature | NotYetValid |
| JWT Headerのパース失敗 | InvalidHeader |
| PayloadのJSONパース失敗 | InvalidPayload |
| JWT形式の不正 | InvalidFormat |
| その他のエラー | Other |

秘密鍵が空の場合は `InvalidKey` を返してください。

重要：

- 署名検証エラーを `InvalidFormat` に変換しないこと。
- JSONパースエラーを `InvalidSignature` に変換しないこと。
- エラーを握りつぶさないこと。
- `unwrap()` や `expect()` によるパニックを通常のエラー処理に使用しないこと。

### 9.3 Display

`JwtError` は `Display` および `std::error::Error` を実装してください。

エラーメッセージは、エラーの原因が分かる内容にしてください。

秘密鍵そのものやJWTの秘密情報をエラーメッセージに含めないでください。

---

## 10. Public API

外部から使用する公開APIは以下を基本としてください。

```rust
pub fn generate_secret_key() -> Result<String, JwtError>;

pub fn encode_jwt(
    payload: &JwtPayload,
    secret_key: &str,
) -> Result<String, JwtError>;

pub fn decode_jwt(
    token: &str,
    secret_key: &str,
) -> Result<JwtPayload, JwtError>;

pub fn validate_time(
    payload: &JwtPayload,
) -> Result<(), JwtError>;
```

また、`JwtPayload` と `JwtError` は外部から利用できるように公開してください。

必要に応じて、Payload生成用のコンストラクタを実装してください。

公開APIを必要以上に増やさないでください。

---

## 11. Module Structure

シンプルなライブラリ構成にしてください。

推奨構成：

```text
src/
├── lib.rs
├── payload.rs
├── error.rs
└── jwt.rs

tests/
└── jwt_tests.rs
```

各ファイルの責務：

| ファイル | 責務 |
|---|---|
| lib.rs | 公開APIの定義・再エクスポート |
| payload.rs | JwtPayload構造体、コンストラクタ |
| error.rs | JwtError定義 |
| jwt.rs | 秘密鍵生成、JWT生成・読み込み、有効期限検証 |
| jwt_tests.rs | 結合テスト |

この構成は推奨ですが、同等の責務分離ができる場合は多少変更しても構いません。

---

## 12. Unit Tests

以下のテストを必ず実装してください。

### 12.1 Secret Key Tests

1. 秘密鍵が正常に生成できる。
2. 生成した秘密鍵が64文字以上である。
3. 生成した秘密鍵が16進数文字列である。
4. 連続して生成した秘密鍵が異なる。

### 12.2 Payload Tests

1. Payloadを正常に生成できる。
2. `iat` が現在時刻に設定されている。
3. `jti` がUUID v4形式である。
4. `iss`、`sub`、`aud` が初期状態でNoneである。
5. `data` に任意のJSONを設定できる。
6. PayloadをJSONにシリアライズできる。
7. JSONからPayloadにデシリアライズできる。
8. `data` がNoneの場合、JSONにdataフィールドが出力されない。

### 12.3 JWT Encoding Tests

1. 正しい秘密鍵でJWTを生成できる。
2. 生成したJWTが3つのセグメントで構成されている。
3. JWT HeaderのアルゴリズムがHS256である。
4. Payloadの内容が正しく保持されている。
5. 空の秘密鍵ではエラーになる。

### 12.4 JWT Decoding Tests

1. 正しい秘密鍵でJWTを読み込める。
2. 読み込んだPayloadが元のPayloadと一致する。
3. 間違った秘密鍵では `InvalidSignature` になる。
4. 改ざんされたJWTでは `InvalidSignature` になる。
5. 不正なJWT形式では `InvalidFormat` または適切な形式エラーになる。
6. 不正なHeaderでは `InvalidHeader` になる。
7. 不正なPayload JSONでは `InvalidPayload` になる。
8. 空の秘密鍵では `InvalidKey` になる。
9. HS256以外のアルゴリズムを使用したJWTは拒否される。

### 12.5 Time Validation Tests

1. 現在時刻が有効期間内なら成功する。
2. 有効期限を過ぎたPayloadでは `Expired` になる。
3. 有効期限と現在時刻が一致した場合は `Expired` になる。
4. 有効開始日時が未来の場合は `NotYetValid` になる。
5. 有効開始日時と現在時刻が一致した場合は成功する。
6. 有効期限と有効開始日時の両方に問題がある場合は `Expired` になる。

### 12.6 Automatic Validation Tests

1. 有効なJWTは `decode_jwt` で正常に読み込める。
2. 有効期限切れのJWTは `decode_jwt` で `Expired` になる。
3. 有効開始日時前のJWTは `decode_jwt` で `NotYetValid` になる。

時刻に依存するテストは、可能な限り固定時刻を使用する内部検証関数を用意して、テスト結果が不安定にならないようにしてください。

---

## 13. Security Requirements

JWTを扱うライブラリとして、以下のセキュリティ要件を守ってください。

1. 署名アルゴリズムはHS256に固定する。
2. 秘密鍵は暗号学的に安全な乱数から生成する。
3. 秘密鍵をソースコードにハードコードしない。
4. 秘密鍵をログに出力しない。
5. 署名検証を省略しない。
6. JWTの署名検証に成功する前にPayloadを信頼しない。
7. JWT Headerに指定されたアルゴリズムを無条件に信用しない。
8. HS256以外のアルゴリズムを許可しない。
9. 有効期限の検証を省略しない。
10. 有効開始日時の検証を省略しない。

---

## 14. Implementation Restrictions

以下の制約を守ってください。

- 非同期処理を導入しない。
- 不要なトレイトや抽象化を追加しない。
- 不要な設定項目を追加しない。
- RSAなどの公開鍵方式を追加しない。
- HS384、HS512など他の署名アルゴリズムを追加しない。
- JWTのリフレッシュ機能を追加しない。
- JWTのブラックリスト機能を追加しない。
- JWTの保存機能を追加しない。
- 独自の暗号化処理を実装しない。
- Payloadの独自データには `serde_json::Value` を使用する。
- 時刻はUnix Timestampの秒単位で統一する。

---

## 15. Completion Criteria

実装完了前に以下を確認してください。

1. 仕様に記載されたすべての関数が実装されている。
2. すべての公開APIが外部から利用できる。
3. JWTの生成・読み込みが正常に動作する。
4. HS256による署名検証が正常に動作する。
5. UUID v4によるjtiの自動生成が動作する。
6. 任意のJSONデータをPayloadに格納できる。
7. 有効期限と有効開始日時を正しく検証できる。
8. 秘密鍵の不一致とJSONパースエラーを区別できる。
9. エラーがResult型で適切に返される。
10. すべてのテストが成功する。
11. `cargo fmt --check` が成功する。
12. `cargo clippy --all-targets --all-features -- -D warnings` が成功する。
13. `cargo test` が成功する。

---

## 16. Final Instructions for Codex

実装を開始する前に、既存プロジェクトの構成とCargo.tomlを確認してください。

既存の実装や設定がある場合は、むやみに変更・削除しないでください。

仕様に矛盾や実装上の重大な問題がある場合は、実装を開始する前にユーザーへ確認してください。

特に問題がなければ、以下の順序で実装してください。

1. Cargo.tomlの依存関係を確認・追加する。
2. JwtErrorを実装する。
3. JwtPayloadを実装する。
4. 秘密鍵生成関数を実装する。
5. JWT生成関数を実装する。
6. JWT読み込み・署名検証関数を実装する。
7. 有効期限検証関数を実装する。
8. 単体テスト・結合テストを実装する。
9. cargo fmtを実行する。
10. cargo clippyを実行する。
11. cargo testを実行する。
12. 実装内容とテスト結果をユーザーに報告する。

実装が完了したら、以下を報告してください。

- 実装した機能
- 追加・変更したファイル
- 使用した外部クレート
- テストの実行結果
- 未対応事項や既知の問題

仕様に記載されていない機能は実装しないでください。