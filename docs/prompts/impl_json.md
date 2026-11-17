# Codex Implementation Specification: Rust JSON Utility Library

## 1. Project Overview

Rustを使用して、JSONとRust構造体の相互変換、およびJSONファイルの読み書きを行うユーティリティライブラリを実装してください。

JSONのシリアライズ・デシリアライズには `serde` および `serde_json` を使用してください。

本ライブラリの主な目的は以下のとおりです。

1. 任意のRust構造体をJSON文字列に変換する。
2. JSON文字列を任意のRust構造体に変換する。
3. JSONファイルの読み込みと書き込みを行う。
4. インデント付きの整形されたJSONを出力する。
5. JSON入出力用の構造体（DTO）と、アプリケーション内部で使用するドメイン構造体を分離する。
6. DTOからドメイン構造体への変換時に、値の検証とデフォルト値の補完を行う。

特に重要なのは、**JSONの不完全なデータやnullを許容しつつ、ドメイン構造体ではOptionを極力使用せず、常に利用可能な値を保持すること**です。

## 2. Development Requirements

### 2.1 Programming Language

- Rustを使用する。
- Rustの標準的な設計・命名規則に従う。
- 安全性と可読性を重視する。
- 不必要な依存関係を追加しない。

### 2.2 Dependencies

以下のクレートを使用する。

| Crate | Purpose |
|---|---|
| serde | Serialize / Deserialize |
| serde_json | JSONの変換とファイル入出力 |

JSONファイルの読み書きにはRust標準ライブラリの `std::fs` を使用する。

追加のクレートは原則として使用しないこと。

### 2.3 Dependency License

外部クレートを追加する場合は、MIT、Apache-2.0などの一般的に利用可能なライセンスであることを確認する。

既存の依存関係で実装できる場合は、新しい依存関係を追加しないこと。

## 3. Project Structure

以下の構成を基本として実装すること。

```text
src/
├── lib.rs
├── json.rs
├── error.rs
├── models/
│   ├── mod.rs
│   ├── dto.rs
│   └── domain.rs
└── validation.rs

tests/
├── json_tests.rs
├── file_tests.rs
└── conversion_tests.rs

examples/
└── user_example.rs

Cargo.toml
```

各ファイルの役割は以下のとおり。

| File | Responsibility |
|---|---|
| lib.rs | 公開APIの定義 |
| json.rs | JSON Serialize / Deserialize、ファイルIO |
| error.rs | エラー型の定義 |
| models/dto.rs | JSON入出力用構造体 |
| models/domain.rs | ドメイン構造体 |
| validation.rs | DTOからDomainへの変換時の検証 |
| tests/ | 各機能のテスト |
| examples/ | 使用例 |

必要に応じてファイルを追加してもよいが、責務を明確に分離すること。

## 4. JSON Utility Functions

任意の型に対応するジェネリック関数を実装すること。

### 4.1 Serialize

任意のSerialize可能な構造体をJSON文字列に変換する。

Function:

```rust
pub fn to_json<T>(value: &T) -> Result<String, JsonError>
where
    T: serde::Serialize;
```

Requirements:

- 任意のSerialize可能な型を受け取る。
- JSON文字列を返す。
- 変換に失敗した場合はエラーを返す。
- パニックを発生させない。

### 4.2 Deserialize

JSON文字列を任意の型に変換する。

Function:

```rust
pub fn from_json<T>(json: &str) -> Result<T, JsonError>
where
    T: serde::de::DeserializeOwned;
```

Requirements:

- ジェネリック型を使用する。
- 呼び出し側が変換先の型を指定できる。
- JSON形式が不正な場合はエラーを返す。
- 型が一致しない場合はエラーを返す。

### 4.3 Pretty Serialize

インデント付きのJSON文字列を生成する。

Function:

```rust
pub fn to_json_pretty<T>(value: &T) -> Result<String, JsonError>
where
    T: serde::Serialize;
```

Requirements:

- `serde_json::to_string_pretty` を使用する。
- JSONの階層構造が分かるようにインデントする。
- Serializeと同様にエラーを返す。

### 4.4 Generic Type Requirements

JSONユーティリティ関数は特定の構造体に依存してはならない。

例えば、以下のような異なる型で利用できること。

```rust
let user_json = to_json(&user)?;
let config_json = to_json(&config)?;
let settings_json = to_json(&settings)?;
```

## 5. JSON File I/O

JSONファイルの読み込みと書き込みを実装する。

### 5.1 Read JSON File

Function:

```rust
pub fn read_json_file<T, P>(path: P) -> Result<T, JsonError>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<std::path::Path>;
```

Requirements:

- JSONファイルを読み込む。
- 読み込んだJSONを指定された型に変換する。
- ファイルが存在しない場合はエラーを返す。
- ファイルアクセスに失敗した場合はエラーを返す。
- JSON形式が不正な場合はエラーを返す。
- 型