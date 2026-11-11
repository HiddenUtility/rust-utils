---
name: quality-guard
description: scripts/quality_guard.sh を実行し、コードの品質（フォーマット、リンター、テスト）を判定する。
---

# Quality Guard Skill

コードの品質がプロジェクトの規約に適合しているかを確認するためのスキルです。

## 実行手順

1. プロジェクトのルートディレクトリで `./scripts/quality_guard.sh` を実行する。
2. コマンドの終了ステータスを確認する。

## 入力

- 特になし（プロジェクトの現在のファイル状態を使用）。

## 出力

- **成功時**:
  - ステータス: `PASS`
  - メッセージ: `All quality checks passed.`
- **失敗時**:
  - ステータス: `FAIL`
  - メッセージ: `Quality check failed.`
  - エラー詳細: 実行時に出力されたエラー内容（`cargo fmt`, `cargo clippy`, `cargo test` のいずれかからの出力）。

## 制約

- `scripts/quality_guard.sh` が実行可能な状態で存在していること。
- 実行環境に `cargo`, `rustfmt`, `clippy` がインストールされていること。

## 完了条件

- `scripts/quality_guard.sh` の実行が完了し、その結果（PASS または FAIL とエラー内容）が報告されること。
