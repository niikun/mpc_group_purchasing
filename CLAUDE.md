# CLAUDE.md

このファイルはClaude Codeがセッション開始時に自動で読み込む設定ファイルです。詳細な技術仕様は `docs/DESIGN.md` を参照してください(`@docs/DESIGN.md`)。

## このプロジェクトの目的

**目的は学習であり、完成品を最速で作ることではありません。** ユーザーはAdvanced Cryptography Program(ZK/MPC/FHEの7週間講座)の受講生で、Rust歴3ヶ月、TypeScriptはほぼ未経験です。このプロジェクト(秘密計算による共同購入システム)を教材として、MPC→FHEの順に手を動かしながら理解を深めることが最優先です。

## Claude Codeへの行動指針(最重要)

1. **コードを書く前に、まず設計意図を説明する。** 「何を」だけでなく「なぜこの方式か」「他にどんな選択肢があったか」を一言添える。
2. **大きな実装を一度に生成しない。** 1機能・1関数単位で止め、動作確認してから次に進む。ユーザーが追いつけているか、区切りごとに確認する。
3. **フルの解答をすぐに渡さない。** ユーザーがまず自分で書けるよう、雛形・型定義・TODOコメントだけを示し、ヒントを小出しにする。ユーザーが「答えを見せて」と明示的に頼んだときだけ完全実装を出す。
4. **専門用語(秘密分散、しきい値復号、準同型など)が出たら、使う前に一言で定義する。** 前提知識があるものとして進めない。
5. **既存コードに手を入れる前に、変更内容と理由を短く説明してから編集する。**
6. セッションの節目(1機能が動いた、1週分の実装が終わったなど)で、このファイル末尾の「進捗ログ」に1〜2行追記することを提案する。

## プロジェクト概要

競合する複数の事業者(例:外食チェーン)が、塩・砂糖・食用油などのコモディティ資材を共同購入することで仕入単価を下げたい。ただし各社の発注量・希望価格は競争機密であり、互いにも運営者にも開示したくない。この「隠したまま協調する」を、秘密計算で実現する。

**コア機構**:ダブルオークション。各社が価格帯ごとの希望数量(買い手は上限価格、売り手は下限価格)を非公開で入力し、需要合計と供給合計が交わる清算価格だけを復元する。個社の入力値は最後まで非公開。詳細は `docs/DESIGN.md` 参照。

## 2段階構成

| 段階 | 技術 | 言語 | 状態 |
|---|---|---|---|
| 第1段階 | MPC(加法的秘密分散 → 将来的にShamir k-of-n) | Rust | 未着手 |
| 第2段階 | FHE(Zama fhEVM) | Solidity + TypeScript(`@zama-fhe/relayer-sdk`) | 未着手 |

同じダブルオークションのロジックを2つの技術で実装し、トレードオフ(信頼の起点、オンチェーン性、実装コスト)を比較することが最終的なアウトプットの核。

## 技術選定の理由(ユーザーとの合意事項)

- 第1段階はRust。ユーザーが3ヶ月の経験を持ち、MPC分野でも`swanky`等の本格的なライブラリがRustで書かれているため。
- 第2段階はSolidity(コントラクト)+ TypeScript(クライアント)。fhEVMの設計上この組み合わせが必須で、選択の余地はない。
- ノード運営・しきい値復号のインフラは自前実装しない。第2段階はZamaのKMS(13ノードのしきい値復号)を利用する前提。

## ディレクトリ構成(想定)

```
/mpc-rust/          # 第1段階:Rust実装
  src/
    secret_sharing.rs
    node.rs          # tokio/tonicでの複数プロセス通信
    auction.rs        # 清算価格ロジック
/fhe-solidity/       # 第2段階:fhEVM実装
  contracts/
    Auction.sol
  test/
  client/            # TypeScript, relayer-sdk
/docs/
  DESIGN.md           # 詳細仕様(価格グリッド・清算ロジック・按分計算)
  progress.md          # 週次の学習・実装メモ
```

## 参考資料

- MPC全般:Evans, Kolesnikov, Rosulek "A Pragmatic Introduction to Secure Multi-Party Computation"(無料公開)
- ZKP全般:Justin Thaler "Proofs, Arguments, and Zero-Knowledge"(無料PDF:https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf)
- MPCライブラリ(Rust):Galois社 `swanky`
- FHE/fhEVM公式:https://docs.zama.ai/fhevm 、サンプル集:`github.com/zama-ai/fhevm` の examples、`github.com/zama-ai/awesome-zama`
- 実例:2008年デンマーク砂糖大根オークション(世界初の商用MPC実装、単一買い手×多数農家のダブルオークション)

## 進捗ログ

<!-- セッションごとに1〜2行追記。例: 2026-08-09: 加法的秘密分散のRust実装(単一プロセス版)完了 -->

- 2026-08-10: `mpc-rust`プロジェクト作成、`Fp`型(法M上の演算)と加法的秘密分散`split_into_shares`/`reconstruct`を実装・テスト完了(spec §8.1〜8.3)
- 2026-08-11: `PriceQuantity`(9段階価格グリッドの導出ベクトル)と`quantities_share`/`quantities_join`(1人分の分散・復元)を実装。`Node`構造体で局所合算(`add_share`)と3ノード分の復元(`add_node`)を実装・テスト完了(spec §8.2〜8.3)。次回は§8.4清算価格ロジック
- 2026-08-11: `secret_sharing.rs`/`node.rs`/`auction.rs`にファイル分割。`clearing_price`(§8.4)実装にあわせ、清算価格の定義を「D(p) <= S(p)を満たす最も低い価格」(買い手優先)に決定し、DESIGN.md/spec_v0.4.mdを修正。`allocate`(§8.5按分)を実装・全11テスト完了でMPCコアロジックの部品が出揃った。tokioの基礎(`async`/`.await`/`join!`)にも入門。次回はB1〜S2をテスト可能な形で通しで動かす関数、その後tokioでの実プロセス分離
- 2026-08-13: `aggrigate_market`(§8.6 全体フロー)を実装。`clearing_price`の価格からインデックスを引く際に`.find()`ではなく`.position()`を使う必要がある点、按分`allocate`の`total`には清算価格時点の需要/供給量を渡す必要がある点を修正し、build/test通過。「買い手優先」の設計(需要側は全量約定、供給側のみ比例縮小)を確認。次回は`aggrigate_market`専用のテストを書く
- 2026-08-13: `aggrigate_market`のテストを3ケース(需給ぴったり、需要ゼロでNone、按分・境界絡み)作成する過程で2つのバグを発見・修正。(1) `node_a.add_share(...)`が`&self`を取り新しい`Node`を返す非破壊的なメソッドなのに戻り値を再代入しておらず、合算結果が反映されていなかった(`node_a = node_a.add_share(...)`に修正)。(2) 清算価格ちょうどで参加するはずの買い手・売り手が`price < threshold`/`price > threshold`という境界を含まない不等号で除外されていた(`derive`の参加条件`price <= threshold`(買い手)/`price >= threshold`(売り手)に合わせて`<=`/`>=`に修正)。全15テスト通過
- 2026-08-13: MPCコアロジック(§8.1〜8.6)が一通り完成したところで、次のステップをtokioでのノード間通信に決定。いきなり実プロセス分離(別バイナリ+TCP/tonic)に進むと非同期処理自体の理解と通信の複雑さが同時に来るため、まず(a)1プロセス内で`tokio::spawn`+`mpsc`チャネルを使い「独立したノードが通信する」構造をシミュレートする方針に。次回はここから着手
- 2026-08-15: tokioの`async`/`.await`/`tokio::spawn`/`mpsc`チャネルを一から学び、1プロセス内で3つの独立したノードタスクを模擬。各ノードがチャネル経由でシェアを受け取り`add_share`で集計、`Node::add_node`で最終合算するところまで実装(手計算で合計値の一致を確認)。次回は今使っている適当なテスト値を、実際に`split_into_shares`で分割した値に置き換えるところから
- 2026-08-16: `main.rs`のテスト値を、実際に`derive`→`quantities_share`で分割した本物のシェアに置き換え(`set_share_for_send`ヘルパーを追加、買い手3人+売り手2人分)。過程で2つ詰まった点を解消。(1) `Branch`enumに`#[derive(Clone, Copy)]`が付いておらず、ループ内で複数回使うとmoveエラーになる点(`Copy`は暗黙コピー、`Clone`は明示`.clone()`という違いを確認)。(2) 二重ループで内側が`branchs`配列を毎回先頭からzipしていたため、外側が誰(何人目)を処理中かに関わらず常に最初の3人分のbranchが使われ、売り手のシェアがBuyerとして送られてしまっていたバグ(`enumerate`で人ごとのindexを掴んで直す方向で解消)。`cargo run`の出力(buyer_quantities/seller_quantities)を手計算と突き合わせて一致を確認。tokioでのノード間通信シミュレーションが実データで一通り動作
- 2026-08-16: `auction.rs`の`clearing_price`を`pub`にして`main.rs`から呼べるようにし、tokioで集計した`Node`(buyer_quantities/seller_quantities)を`PriceQuantity`に変換して`clearing_price`に渡すところまで接続。出力(price:105, quantity:700)が手計算と一致し、8/13に決めた「(a) 1プロセス内シミュレーション」の目標を達成。次は`allocate`も繋いで各社の約定数量まで出すか、(b)実プロセス分離に進むか検討
- 2026-08-16: `main.rs`に`allocate`も接続し、各社(b0/b1/b2/s0/s1)の約定数量まで出すところまで完成。詰まった点は主に「参加判定してない全員の生の希望数量を単純合計していた」「ループ変数`q`/`th`を使わず特定の1人(`quantity_b0`)を使い回していた」「seller用のループなのに`total_demand`に足し込んでいた」の3つで、いずれも`aggrigate_market`の「price<=threshold(買い手)/price>=threshold(売り手)で参加判定してから按分」というパターンを再確認する形で修正。結果(b0除外、b1/b2/s0/s1が700を分母にフル約定)が手計算と一致。これでtokioシミュレーション経由でも`aggrigate_market`と同等の出力が出せるようになった。**次回はここから: (b) 実プロセス分離(別バイナリ/プロセス + TCP or tonicでの通信)に着手。async/await/mpscチャネルの基礎は掴めているので、次はネットワーク越しの通信・シリアライズが新しい壁になる**
- 2026-08-17: (b)実プロセス分離に向けて方針を決定。通信は生のTCP(`tokio::net`)+`serde`から始め、余裕があればtonic(gRPC)へ、というステップを踏むことに(いきなりtonicだとProtocol Buffers/build.rsまで一度に増えるため)。プロセス構成は`src/bin/`配下にファイルを置くと自動的に別バイナリになるCargoの規約を使い、`node`(3ノード分)と`coordinator`(旧main.rsのシェア送信役)に分ける方向。まず`src/bin/serde_test.rs`で`serde`+`serde_json`(`Cargo.toml`に追加)を単体で試し、簡単な構造体の`to_string`/`from_str`が動くことを確認。次回はここから: `Fp`/`Branch`に`Serialize`/`Deserialize`を付け、実際にTCP経由で送受信するところに進む
- 2026-08-20: `tcp_client.rs`から`secret_sharing`等が見えないエラーをきっかけに、`main.rs`/`tcp_client.rs`/`tcp_server.rs`がそれぞれ別クレートで、`mod`宣言は同じクレート内でしか使えないことを確認。共通コードを共有するため`src/lib.rs`を新設し、`secret_sharing`/`auction`/`node`の`mod`宣言をそちらに集約(`pub mod`に変更)、3バイナリ側は`use mpc_rust::...`で読み込む構成に変更。詰まった点は(1)モジュール宣言に`pub`を付け忘れ「見つからない」エラーになった点、(2)`main.rs`に古い`mod secret_sharing;`が残り型が二重定義されて`mismatched types`になった点。その後`tcp_client.rs`で`Fp`の値を`serde_json::to_string`でJSON化して送信、`tcp_server.rs`で`serde_json::from_str`で`Fp`に戻す送受信を実装。`format!("{:?}\n", json)`のDebugフォーマットがJSON文字列を二重エスケープするバグに気づき`{}`(Display)に直して解消、`Fp { value: 42 }`が正しく復元されることを確認。次回はNode(実際のシェア)を送るところ、または複数クライアント/複数ノード間の通信に進む
- 2026-08-20: `tcp_client.rs`で`Fp`単体だけでなく`Node`(シェア配列+`Branch`)もJSON化してTCP送信できることを確認。続けて`tcp_server.rs`を`loop`+`accept`+`tokio::spawn`で複数クライアントを並行に受け付けられる構成に変更(`main.rs`で使った`tokio::spawn`の知識を再利用)。詰まった点は`reader.read_line(&mut line);`に`.await`を付け忘れ、実際には何も読み込まれないまま空文字列を`serde_json::from_str`に渡してしまい「EOF while parsing a value」でパニックしていた点(非同期関数は`.await`しないと実行されないことを再確認)。複数クライアントを並行処理できるサーバーが動作するところまで確認
- 2026-08-21: `tcp_server.rs`の`read_line`の`match`が`Ok(0)`(EOF=クライアント切断)しか腕を持たず全パターン網羅できていなかったコンパイルエラーを修正。`Ok(_)`/`Err(e)`を追加し、JSONパース失敗時は接続を切らず`continue`、`Mutex::lock()`失敗時は`PoisonError::into_inner()`で復旧する処理を追加。「poisoning」(ロック保持中にスレッドがpanicすると以後の`.lock()`が`Err`を返す仕組み)を検証するため、`Fp`が`#[derive(Deserialize)]`で`value`フィールドに直書きされ`Fp::new`の`% M`丸めを迂回できる(コンストラクタの不変条件をシリアライズが無視する)ことを利用し、`u64::MAX`近い値を埋め込んだ生JSONを送信して`Fp::add`内でoverflow panicを実際に発生させ、poisoned→`into_inner()`での復旧ログを確認。さらに`add_share`が`&self`を取り新しい`Node`を返す非破壊的な設計(8/13にバグ修正で採用した形)のおかげで、panicが`*guard = guard.add_share(...)`の代入前(右辺計算中)に起きたため、poisoned後も`guard`の中身は前の正しい状態のまま保たれていたことを確認(poisoning=必ずしも実際のデータ破損を意味しない、という気づき)
- 2026-08-21: `tcp_server.rs`をポート固定(8000)から、複数プロセス起動を見据えて起動時に引数でポート番号を指定できる形に変更しようとした際、編集中にブレース対応が崩れ処理ブロックが二重定義されコンパイル不能に。直前コミット(`git checkout`)まで一旦戻し、`env::args()`でポート番号を受け取り`127.0.0.1:{port}`形式に変換して`TcpListener::bind`に渡す形で最小限にやり直し。詰まった点は`addresses[0]`のように`Vec<String>`の要素をインデックスでそのまま値として使おうとし`cannot move out of index`エラーになった点(`String`は`Copy`ではないためVecから要素だけを持ち去れない)で、`&addresses[0]`と借用に直して解消。`cargo run --bin tcp_server -- <port>`での起動・接続を動作確認済み
