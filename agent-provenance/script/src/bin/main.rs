//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use agent_provenance_lib::{commit_profile, BuyerProfile, OrderRecord, Profile, RoundResult};
use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, ProvingKey, SP1Stdin,
};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
const PROVENANCE_ELF: Elf = include_elf!("agent-provenance-program");


/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,
    #[arg(long)]
    prove: bool,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    let args = Args::parse();
    if args.execute == args.prove {
        eprintln!("Error: --execute か --prove のどちらか一方");
        std::process::exit(1);
    }

    // --- サンプルデータ(本来は各社が事前にコミット済みの自社データ) ---
    let profile = Profile::Buyer(BuyerProfile {
        strategy: "在庫を確保しつつコストを抑える".into(),
        current_stock: 700, use_per_day: 130, reorder_point: 600, max_stock: 2500,
        order_history: vec![OrderRecord { date: "2026-07-15".into(), order_amount: 900 }],
        price_ceiling: 115, notes: "平常運転".into(),
    });
    let salt: [u8; 32] = [42u8; 32]; // TODO: 本番なら OsRng 等の乱数
    let rounds: Vec<RoundResult> = vec![]; // まずは履歴なし(初回入札)で試す
    let my_index: usize = 0;

    // --- guest への入力 ---
    let mut stdin = SP1Stdin::new();
    stdin.write(&profile);
    stdin.write(&salt);
    stdin.write(&rounds);
    stdin.write(&my_index);

    let client = ProverClient::from_env();

    if args.execute {
        let (output, report) = client.execute(PROVENANCE_ELF, stdin).run().unwrap();
        let journal = output.as_slice();

        let commitment: [u8; 32] = journal[0..32].try_into().unwrap();
        let history_digest: [u8; 32] = journal[32..64].try_into().unwrap();
        let prompt_hash: [u8; 32] = journal[64..96].try_into().unwrap();

        // ホスト側で独立に再計算(本来は「検証者」がやる。ここでは動作確認としてホストが両方やる)
        let expected_commitment = commit_profile(&profile, &salt);
        assert_eq!(commitment, expected_commitment, "commitment mismatch");

        println!("commitment      = {}", hex(&commitment));
        println!("history_digest  = {}", hex(&history_digest));
        println!("prompt_hash     = {}", hex(&prompt_hash));
        println!("cycles = {}", report.total_instruction_count());
    } else {
        let pk = client.setup(PROVENANCE_ELF).expect("setup failed");
        let proof = client.prove(&pk, stdin).run().expect("prove failed");
        client.verify(&proof, pk.verifying_key(), None).expect("verify failed");
        println!("proof verified");
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}
