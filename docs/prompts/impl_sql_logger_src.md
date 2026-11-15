

codex に 下記の仕様を実装させたいのですが，パラメータが少ないモデルで間違いなく作成できるような詳細仕様のプロンプトを出力してください．
開発言語はRustです．
外部クレート使う場合は MITなどのライセンスのもの．
外部クレートに候補がある場合，有名で Pure Rust製を優先します．
不明点あればきいてください．

# SqlLogger

簡易的な sqliteを使った logger を作ります．

## SqlLogLevel
- SqlLogLevel を enum で定義します．
- CRITICAL, ERROR, WARNING, INFO, DEBUG 持ち，formatで簡単に str に変換できるように実装します．

## SqlLogRecode
- SqlLogRecode を 構造体で定義します．
- 構造体のフィールドには timestamp:f64(unix time), level:SqlLogLevel, name:String, msg: String, pid: usize, 
- 構造体に critical, error, warning ,info, debug メソッド(以後ログメソッド)を実装します．
- ログメソッドの引数は name:str, msg: str のみです．
- levelにはログメソッド名に対応する LogLevel インスタンス時にセットします．
- ログメソッドは インスタンスした時間を ローカル時間を unix time を timestamp とします．
- DisplayやDebugではISO 8601規格で（example: 2026-11-11T12:13:45.123456）で表示してください．
- のちのちjson に変換可能なトレイト入れといてください．

## SqlLogger
- Logger 本体の フィールドは 
    db_path:PathBuf, 
    recodes:SqlLogRecode[], 
    realtime:bool, 
    log_level: SqlLogLevel, 
    now_row_num:u64, 
    max_rec_num:u64 です．
- db_path は sqliteのdbファイルの出力パスです．(example: logs/logs.db)
- db_path の親パスはなければ作ります．
- db_path が 存在してれば now_row_num にセットし，なければ now_row_num は0で初期化します．
- db_path が 存在してれば テーブルを作成します．
- recodes は SqlLogRecode 配列です．
- テーブルのカラムは timestamp:f64, log_level:String, name:String, pid:usizes, msg:String
- テーブルの インデックスは timestamp 単体および， log_level または name カラム で where 打ったときに効率よくなるようにしてください．
-  get_recs メソッドを実装します．
    - get_recsメソッドは SELECT を発行して，dbよりレコードを得て，SqlLogRecode に変換し，SqlLogRecode配列を返します．
    - `ORDER BY timestamp ORDER BY timestamp limit 1000` をつけて最新から 1000得るようにします． 
- get_recs_with_limit メソッドを実装します．
    - 最大行数を引数で指定します．get_recsメソッドと同じ SELECT 発行します．
    - `ORDER BY timestamp limit <引数>` をつけて最新から得るようにします． 
- get_recs_with メソッドを実装します．
    - 最大行数を引数で指定します．
    - オプションとして name, log_levelを指定できます．
    - get_recsメソッドと同じ SELECT 発行します．`ORDER BY timestamp DESC limit <引数>` をつけて最新から得るようにします． 
    - オプション指定ある場合は where を追加します．
- max_rec_num は 最大行数です．デフォルトは 10,000,000 rows です．
- reduce メソッドを実装します．
    - 引数は days: usize です． 
    - now_row_num が max_rec_num を超えていた場合，days前のデータを DELETE クエリで削除します．
- out メソッドを実装します．
    - SqlLogRecode 配列 を dbにインサートします. 
    - 速度を重視するため1回のトランザクションで複数行同時に書き込みます．
    - 成功したらLogRecode配列を空にします．
- LogRecode を書き出すとき，now_row_numを足し上げます．
- 構造体に ログメソッド を実装します．
- ログメソッド の引数は msg: str のみです．
- ログメソッドは self.log_level 満たないメソッドだった場合は無視します．(例 log_level=INFO なら debug 呼ばれても書き込まない)
- ログメソッド は メソッドに応じた SqlLogRecode を生成し，SqlLogRecode配列 に追加します．
- ログメソッド は realtime = true なら out を呼びます．
- デストラクタでは out メソッドを必ず呼びます．エラーは握りつぶします．
- initialize メソッドを実装します．db_path を消して，テーブルをrecreateします．
