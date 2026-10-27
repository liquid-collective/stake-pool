//! Program entrypoint

#![cfg(all(target_os = "solana", not(feature = "no-entrypoint")))]

use {
    crate::{error::StakePoolError, processor::Processor},
    solana_account_info::AccountInfo,
    solana_msg::msg,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
    solana_security_txt::security_txt,
};

solana_program_entrypoint::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        msg!(error.to_str::<StakePoolError>());
        Err(error)
    } else {
        Ok(())
    }
}

security_txt! {
    // Required fields
    name: "SPL Stake Pool",
    project_url: "https://liquidcollective.io/liquid-staked-sol/",
    contacts: "mailto:security@liquidcollective.io",
    policy: "https://github.com/liquid-collective/security/blob/main/VULNERABILITY_DISCLOSURE.md",

    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/liquid-collective/stake-pool",
    source_revision: "3402542b88648aa423a7aa0416ff1055f46081d3", // the commit before the release commit
    source_release: concat!("program@v", env!("CARGO_PKG_VERSION"), "-lc.1"),
    auditors: "https://certificate.quantstamp.com/full/liquid-collective-solana/cbb18ece-3a03-4897-a040-f197fc641e4d/index.html"
}
