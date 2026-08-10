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
