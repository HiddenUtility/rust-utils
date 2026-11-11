//! IO 関連のユーティリティを提供するクレートです。
//!
//! ここに、ファイル操作やストリーム操作などの便利な関数を実装していきます。

/// 動作確認用の関数
pub fn hello_io() {
    println!("Hello from io crate!");
    let _unused = 42; // Fixed: added underscore
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_io() {
        // このテストは単に呼び出せることを確認するもの
        hello_io();
    }
}
