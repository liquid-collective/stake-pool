//! Program entrypoint

#![cfg(all(target_os = "solana", not(feature = "no-entrypoint")))]

use {
    crate::{error::StakePoolError, processor::Processor},
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program_error::PrintProgramError,
        pubkey::Pubkey,
    },
    solana_security_txt::security_txt,
};

solana_program::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<StakePoolError>();
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
    policy: "https://github.com/liquid-collective/security/blob/main/VULNERABILITY_DISCLOSURE.md ",

    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/liquid-collective/stake-pool/tree/liquid_collective_release",
    source_revision: "", // fill in after v1.0.0 bump lands
    source_release: "stake-pool-v2.0.3",
    auditors: "https://certificate.quantstamp.com/full/liquid-collective-solana/cbb18ece-3a03-4897-a040-f197fc641e4d/index.html"
}
