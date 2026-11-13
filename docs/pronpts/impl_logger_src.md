

codex に 下記の仕様を実装させたいのですが，パラメータが少ないモデルで間違いなく作成できるような詳細な仕様書のプロンプトに書き直してくれますか．不明点あればきいてください

# logger

簡易的な text logger を作ります．

LogLevel を enum で定義します．
CRITICAL, ERROR, WARNING, INFO, DEBUG 持ち，formatで簡単に str に変換できるように実装します．

LogRecode を 構造体で定義します．
構造体のフィールドには timestamp:String(unix time), level:enum, msg: String, pid: int
構造体に critical, error, warning ,info, debug メソッドを実装します．
critical, error, warning ,info, debug メソッド の引数は msg: str のみです．
levelは log levelに相当します critical, error, warning ,info, debug メソッドに応じてインスタンス時にセットします．
これらのメソッドは インスタンスした時間をISO 8601規格で（example: 2026-11-11T12:13:45.123456） timestamp とします．
to_logメソッドを実装します．フィールドから `timestamp [level] msg` を生成し Stringの所有権を渡します．

Logger 本体の フィールドは log_path:PathBaf, recodes:LogRecode[], max_file_num:u32, max_file_size:u32, realtime:bool,now_size:u32 です．
log_path は logファイルの出力パスです．(example: logs/name.log)
log_path の親パスはなければ作ります．
log_path が 存在してれば now_size にセットし，なければ now_sizeは0で初期化します．
recodes は LogRecode配列です．
max_file_num は logファイルの出力数です．超えたものは古い順に削除します．デフォルトは 7 です．
max_file_size は logフィルの最大サイズ数です．
out メソッドを持ち，LogRecode配列 を logファイルとして書き出し，成功したらLogRecode配列を空にします．
LogRecodeを書き出すとき，now_sizeを足し上げます．
すべてのLogRecodeを書きだしたあとに，now_size が max_file_size を超えていた場合，logファイルの拡張子を log.1とします．
すでに log.1 が存在した場合は log.1 を log.2へ，log.2 があればlog.3へを繰り返し，max_file_numに到達したlogファイルは削除します．
構造体に critical, error, warning ,info, debug メソッドを実装します．
critical, error, warning ,info, debug メソッド の引数は msg: str のみです．
critical, error, warning ,info, debug メソッド は メソッドに応じた LogRecode を生成し，LogRecode配列 に追加します．
critical, error, warning ,info, debug メソッド は realtime が true なら out を呼びます．
デストラクタでは out メソッドを必ず呼びます．

