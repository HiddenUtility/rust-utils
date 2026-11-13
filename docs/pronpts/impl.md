# logger

簡易的な logger を作ります．

まず LogRecode を 構造体で定義します．
構造体のフィールドには timestamp, level, msg
構造体に critical, error, info, debug メソッドを実装します．
critical, error, info, debug メソッド の引数は msg: str のみです．
levelは log levelに相当します critical, error, info, debug メソッドに応じてインスタンス時にセットします．
これらのメソッドは インスタンスした時間（マイクロ秒まで）を timestamp とします．

Logger 本体の フィールドは dir_path, recodes, max_file_num, max_file_sizeです．
dir_path は logの出力先です．
recodes は LogRecode配列です．

構造体に critical, error, info, debug メソッドを実装します．
critical, error, info, debug メソッド の引数は msg: str のみです．
critical, error, info, debug メソッド は メソッドに応じた LogRecode を生成し，LogRecode配列 に追加します．
out メソッドを持ち，LogRecode配列 を logファイルとして書き出し，成功したらLogRecode配列を空にします．
デスコンストラクタでは out メソッドを必ず呼びます．
