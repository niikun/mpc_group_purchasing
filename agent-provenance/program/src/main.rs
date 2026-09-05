//! profile・salt・公開履歴を witness として受け取り、commitment・history_digest・prompt_hash を
//! journal(公開出力)にコミットする guest プログラム。

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use agent_provenance_lib::{commit_profile, format_history, render_prompt, sha256, Profile, RoundResult};

pub fn main() {
    // --- private inputs(witness、journal には出ない) ---
    let profile: Profile = sp1_zkvm::io::read();
    let salt:[u8;32] = sp1_zkvm::io::read();
    let rounds: Vec<RoundResult> = sp1_zkvm::io::read();
    let my_index = sp1_zkvm::io::read();

    // --- 再計算 ---
    let commitment = commit_profile(&profile, &salt);
    let history = format_history(&rounds, my_index);
    let history_digest = sha256(history.as_bytes());
    let prompt = render_prompt(&profile, &history);
    let prompt_hash = sha256(prompt.as_bytes());

    // --public values(journal,検証者が見る)--
    sp1_zkvm::io::commit_slice(&commitment);
    sp1_zkvm::io::commit_slice(&history_digest);
    sp1_zkvm::io::commit_slice(&prompt_hash);
}
