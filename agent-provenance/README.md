# agent-provenance — AI入札エージェント入力の出所証明(ZK PoC)

[秘密計算による共同購入システム](../README.md)の**柱3**(ZKによるAI入力の出所証明)にあたるPoCです。SP1(zkVM)を使い、「AI入札エージェントに送ったプロンプトが、事前にコミットした自社データと公開データだけの関数だった」ことを、プロフィールの中身を明かさずに証明・検証します。

## これは何を証明するか / しないか

**証明できる(このPoCが実際にやっていること)**

- 各社が事前に自社プロフィールのコミットメント `commitment = sha256(bincode(profile) ++ salt)` を公開しておく
- SP1のguest(zkVM内)が、秘密のwitness(profile・salt・公開履歴)から `commit_profile` / `format_history` / `render_prompt` を再計算し、`commitment` / `history_digest` / `prompt_hash` をjournal(公開出力)にコミットする
- 検証者は、profileの中身を一切見ずに「送信予定のプロンプトは、公開済みのcommitmentとhistoryだけから決定的に導出された」ことを検証できる

**証明できない(スコープ外)**

- そのprompt_hashが実際にモデルAPIへ送信された文字列と一致すること(接続未実装)
- モデルが実際にその入力**だけ**から推論したこと(隠れコンテキストや他データで条件付けしていないこと)——verifiable ML inferenceは数十億パラメータ規模では非現実的
- 学習データに他社の非公開データが混入していないこと(verifiable training、研究段階)
- 提案内容がその会社の利益に反していないか、という妥当性そのもの

これは**カルテル対策そのもの**ではありません。対策の本体はデータ分離・人間確認という設計(spec 11.4/11.5)で、このPoCはその設計の主張——「他社の情報は入力に混ざっていない」——を第三者に検証可能にする一部分です。詳しい切り分けは [`docs/llm_agent_observation.md` §6・§7](../docs/llm_agent_observation.md) を参照。

## 構成

```
lib/      プロンプト生成の共有ロジック(host/guest両方から使う)
program/  SP1 guest(zkVM内で実行される再計算プログラム)
script/   host(profile/saltを用意し、execute/proveを実行するCLI)
```

- `lib/src/lib.rs` の `build_buyer_prompt` / `build_supplier_prompt` / `format_history` は、`mpc-rust/src/ai_agent.rs` の実パスから**コピー**したものです。1文字でもズレると「実際に使っているコードで再計算した」という証明の主張が崩れるため、変更する際は両方を同時に直してください(本番化する場合は単一の共有クレートに統合するのが望ましい、未実施)。
- クレート名は `agent-provenance-lib` / `agent-provenance-program` / `agent-provenance-script`(実行バイナリ名は `agent-provenance`)。当初はSP1公式テンプレートの `fibonacci-*` のままだったが、このプロジェクト向けにリネーム済み。

## 実行方法

### 依存関係

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/docs/sp1/getting-started/install)

### 実行のみ(証明なし、再計算結果の確認)

```sh
cd script
cargo run --release -- --execute
```

`commitment` / `history_digest` / `prompt_hash` と実行サイクル数を表示し、host側で独立に再計算した `commitment` と一致することを確認します。

### 証明生成・検証

```sh
cd script
cargo run --release -- --prove
```

CPU上でSP1 coreプルーフを生成し、その場で検証まで行って `proof verified` を出力します。ローカルCPU証明で数十秒〜数分オーダーです。

### Succinct Prover Networkを使う場合(任意)

```sh
cp .env.example .env
# .env で SP1_PROVER=network と NETWORK_PRIVATE_KEY を設定
```

`script/src/bin/vkey.rs`(検証鍵の取得)やGroth16/PLONK(EVM向け)生成コマンドはSP1公式テンプレート由来のまま残っていますが、このPoCでは未使用・未検証です。

## サンプルデータについて

`script/src/bin/main.rs` の profile・salt・roundsは動作確認用のハードコードされたサンプルです(saltも `[42u8; 32]` 固定でTODOコメントあり)。実運用相当にするには、各社が乱数saltでコミットメントを生成し、自社のprofileとsaltをwitnessとして渡す形に置き換える必要があります。

## 関連ドキュメント

- [プロジェクト全体のREADME](../README.md) — 3本柱の全体像
- [`docs/llm_agent_observation.md`](../docs/llm_agent_observation.md) — 柱2(LLM入札エージェント)の観察結果とこのPoCの位置づけ(§7)
- [`spec_v0.4.md`](../spec_v0.4.md) — §11.6(限界)・§15(制限事項)・§16(拡張案)
