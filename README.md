# 秘密計算による共同購入システム

競合する複数の事業者(例:外食チェーン)が、塩・砂糖・食用油などのコモディティ資材を共同購入して仕入単価を下げたい。ただし各社の発注量・希望価格は競争機密であり、互いにも運営者にも開示したくない。この「隠したまま協調する」を、秘密計算で実現するプロジェクトです。

Advanced Cryptography Program(ZK/MPC/FHEを扱う全7週の実装講座)の最終成果物として、同一のダブルオークション・ロジックを2つの秘密計算技術(MPC・FHE)で実装し、そのトレードオフを比較することを目的としています。

## コア機構:ダブルオークション

各社が価格帯ごとの希望数量(買い手は上限価格、売り手は下限価格)を非公開で入力し、需要合計 D(p) と供給合計 S(p) が交わる清算価格だけを復元します。個社の入力値は最後まで非公開のままです。詳細は [`spec_v0.4.md`](spec_v0.4.md) を参照してください。

## 3本柱

このプロジェクトのアウトプットは、主従関係を持つ3本柱で構成されています。柱2・3は、柱1で実現した「入力を隠したまま協調する」秘密計算があって初めて成立する応用です。

### 柱1(土台):MPC 対 FHE 比較

同一のダブルオークション・ロジックを、加法的秘密分散(MPC)と完全準同型暗号(FHE、Zama `tfhe-rs`)の2通りで独立に実装し、信頼の起点・計算コスト・スケール特性を実測で比較します。

| | MPC(3ノード加法的秘密分散) | FHE(`tfhe-rs`) |
|---|---|---|
| 実装 | [`mpc-rust/`](mpc-rust/) | [`fhe-rust/`](fhe-rust/) |
| 状態 | 完了(実プロセス版まで) | コア完了(derive→集計→清算価格→按分) |
| 信頼の起点 | 3ノードが全員結託しない限り秘匿 | 復号鍵を持つ者は全部復号できる(鍵分散は簡略化) |
| 早期終了 | できる | できない(暗号値は分岐不可、全価格帯を評価) |
| 1取引の通し時間(買い手3・売り手2) | median **1.2 µs**(n=1000) | median **105 秒**(n=4) |

比はおよそ **8.8×10⁷倍(約8桁)**。定性・定量の詳細な比較は [`docs/comparison.md`](docs/comparison.md) を参照。

### 柱2:AI入札エージェントの多ラウンド観察

各社が自社データのみを見て入札判断を行う設計(spec 11章)をベースに、人間確認を外した研究・分析用モードで15ラウンド自動売買させ、収束・振動・カルテル類似構図の有無を観察しました。実装は [`mpc-rust/src/ai_agent.rs`](mpc-rust/src/ai_agent.rs)。米RealPage事件(アルゴリズム経由のハブ・アンド・スポーク型共謀、2025年11月DOJ和解)を踏まえた設計上の安全策(データ分離・人間確認)も併せて実装しています。観察結果は [`docs/llm_agent_observation.md`](docs/llm_agent_observation.md) を参照。

この構図は米国だけの話ではありません。公正取引委員会も[「アルゴリズム/AIと競争政策」報告書(2021年3月)](https://www.jftc.go.jp/houdou/pressrelease/2021/mar/210331_digital.html)で、「アルゴリズムを介して価格が連動していると認識し、それを受け入れて使い続けていた」場合は独占禁止法上の「意思の連絡」に当たりうると分析しており、日本の独禁法の下でも同種のリスクがあります(詳細は [`spec_v0.4.md` 17章](spec_v0.4.md)・11.5節)。

### 柱3:ZK による AI 入力の出所証明 PoC

柱2の設計が主張する「AIへの入力は自社データのみ」という性質を、SP1(zkVM)で暗号学的に検証可能にする PoC です。**カルテル対策そのもの**(それはデータ分離・人間確認という設計側が担う)**ではなく**、その設計の主張を第三者に開示せず検証できるようにする、証明可能性の一部という位置づけです。実装は [`agent-provenance/`](agent-provenance/)。プロフィールのコミットメント公開 → guest内での再計算(`commit_profile`/`render_prompt`/`sha256`)→ 証明生成・検証まで動作確認済み。

## ディレクトリ構成

| パス | 内容 |
|---|---|
| [`mpc-rust/`](mpc-rust/) | 第1段階:MPC実装(秘密分散・ノード間通信・清算価格・按分・AIエージェント) |
| [`fhe-rust/`](fhe-rust/) | 第2段階:FHE実装(`tfhe-rs`、同じ4フェーズを暗号文のまま) |
| [`agent-provenance/`](agent-provenance/) | 柱3:SP1(zkVM)によるAI入力出所証明のPoC |
| [`analysis/`](analysis/) | ベンチマーク・シミュレーション結果の可視化(Python、`uv`) |
| [`docs/`](docs/) | 比較考察・観察結果・発表原稿案 |
| [`spec_v0.4.md`](spec_v0.4.md) | 詳細仕様書(用語・アルゴリズム・スコープ・制限事項) |
| [`DESIGN.md`](DESIGN.md) | 初期設計メモ(高レベルな要点。最新の詳細は `spec_v0.4.md` を参照) |
| [`CLAUDE.md`](CLAUDE.md) | 開発方針・進捗ログ(Claude Codeとの協働記録) |

## 動かし方

```sh
# MPC: 実プロセス版デモ(3ノード + 5参加者 + coordinator)
cd mpc-rust && ./demo.sh

# MPC: ベンチマーク(1000回試行の中央値)
cd mpc-rust && cargo run --release --bin bench

# FHE: derive → 集計 → 清算価格 → 按分 の一連の例
cd fhe-rust && RUSTFLAGS="-C target-cpu=native" cargo run --release --example derive

# 柱2: LLM入札エージェントの多ラウンド観察(要 ANTHROPIC_API_KEY)
cd mpc-rust && cargo run --release --bin observe_demo

# 柱3: ZK入力証明PoC(実行のみ / 証明生成)
cd agent-provenance/script
cargo run --release -- --execute
cargo run --release -- --prove
```

## ドキュメント一覧

- [`spec_v0.4.md`](spec_v0.4.md) — 詳細仕様書
- [`docs/comparison.md`](docs/comparison.md) — MPC対FHEの定性・定量比較
- [`docs/llm_agent_observation.md`](docs/llm_agent_observation.md) — LLM入札エージェントの観察結果とZK証明の切り分け
- [`CLAUDE.md`](CLAUDE.md) — 開発方針・週次の進捗ログ

## 位置づけ

本プロジェクトは学習を目的とした教育的な比較研究であり、本番運用を想定したプロダクトではありません。鍵管理の簡略化(3-of-3固定、しきい値復号KMS未使用)、能動的安全性の欠如、AIエージェント機能のオプション性など、既知の制限事項は [`spec_v0.4.md` 15章](spec_v0.4.md) にまとめています。
