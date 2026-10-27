//! Permissioning guard for the Liquid Collective fork.
//!
//! Every instruction that moves user funds into or out of the pool must be
//! gated, so that only the Liquid Collective proxy program can perform it on
//! behalf of its allowlisted users. Upstream gates `DepositStake`,
//! `DepositSol` and `WithdrawSol` (and their `WithSlippage` variants), and the
//! Liquid Collective patch gates `WithdrawStake` and
//! `WithdrawStakeWithSlippage` with `sol_withdraw_authority`.
//!
//! The tests in this file exist so that this rule cannot silently regress when
//! the fork is rebased on a newer upstream release.

// Modified by Alluvial Finance, Inc. for Liquid Collective on 27-10-2026
// Changes: Permissioning the stake-pool

#![allow(clippy::arithmetic_side_effects)]
// `Redelegate` is deprecated, but it still has to be classified
#![allow(deprecated)]

mod helpers;

use {
    helpers::*,
    solana_program::{
        instruction::{Instruction, InstructionError},
        pubkey::Pubkey,
    },
    solana_program_test::*,
    solana_sdk::{
        signature::{Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
    solana_stake_interface as stake,
    spl_stake_pool::{
        error::StakePoolError,
        id,
        instruction::{self, FundingType, StakePoolInstruction},
        MINIMUM_RESERVE_LAMPORTS,
    },
    test_case::test_case,
};

/// The account that has to sign for a fund-moving instruction
#[derive(Debug, PartialEq)]
enum Gate {
    StakeDeposit,
    SolDeposit,
    SolWithdraw,
}

#[derive(Debug, PartialEq)]
enum Class {
    /// Moves user funds into or out of the pool, so it must be gated
    FundMoving(Gate),
    /// Only the manager or the staker can call it
    Privileged,
    /// Permissionless bookkeeping that does not move user funds
    Crank,
}

/// Classifies every instruction of the program.
///
/// **There is deliberately no wildcard arm.** When an upstream release adds an
/// instruction, this function stops compiling until someone decides what the
/// new instruction does. If it deposits or withdraws funds, it must take a
/// signer that the pool can restrict (see `WithdrawStake`), and a test below
/// has to show that it is gated.
fn classify(instruction: &StakePoolInstruction) -> Class {
    use {Class::*, Gate::*, StakePoolInstruction as I};
    match instruction {
        I::DepositStake | I::DepositStakeWithSlippage { .. } => FundMoving(StakeDeposit),
        I::DepositSol(_) | I::DepositSolWithSlippage { .. } => FundMoving(SolDeposit),
        I::WithdrawStake(_)
        | I::WithdrawStakeWithSlippage { .. }
        | I::WithdrawSol(_)
        | I::WithdrawSolWithSlippage { .. } => FundMoving(SolWithdraw),

        I::Initialize { .. }
        | I::AddValidatorToPool(_)
        | I::RemoveValidatorFromPool
        | I::DecreaseValidatorStake { .. }
        | I::IncreaseValidatorStake { .. }
        | I::SetPreferredValidator { .. }
        | I::SetManager
        | I::SetFee { .. }
        | I::SetStaker
        | I::SetFundingAuthority(_)
        | I::CreateTokenMetadata { .. }
        | I::UpdateTokenMetadata { .. }
        | I::IncreaseAdditionalValidatorStake { .. }
        | I::DecreaseAdditionalValidatorStake { .. }
        | I::DecreaseValidatorStakeWithReserve { .. }
        | I::Redelegate { .. } => Privileged,

        I::UpdateValidatorListBalance { .. }
        | I::UpdateStakePoolBalance
        | I::CleanupRemovedValidatorEntries => Crank,
    }
}

#[test]
fn fund_moving_instructions_are_classified() {
    use {Gate::*, StakePoolInstruction as I};
    let fund_moving = [
        (I::DepositStake, StakeDeposit),
        (
            I::DepositStakeWithSlippage {
                minimum_pool_tokens_out: 0,
            },
            StakeDeposit,
        ),
        (I::DepositSol(0), SolDeposit),
        (
            I::DepositSolWithSlippage {
                lamports_in: 0,
                minimum_pool_tokens_out: 0,
            },
            SolDeposit,
        ),
        (I::WithdrawSol(0), SolWithdraw),
        (
            I::WithdrawSolWithSlippage {
                pool_tokens_in: 0,
                minimum_lamports_out: 0,
            },
            SolWithdraw,
        ),
        (I::WithdrawStake(0), SolWithdraw),
        (
            I::WithdrawStakeWithSlippage {
                pool_tokens_in: 0,
                minimum_lamports_out: 0,
            },
            SolWithdraw,
        ),
    ];
    for (instruction, gate) in fund_moving {
        assert_eq!(classify(&instruction), Class::FundMoving(gate));
    }
}

async fn send(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    extra_signers: &[&Keypair],
) -> Option<TransactionError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .unwrap();
    let mut signers = vec![&context.payer];
    signers.extend_from_slice(extra_signers);
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &signers,
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .err()
        .map(|e| e.unwrap())
}

fn instruction_error(error: InstructionError) -> Option<TransactionError> {
    Some(TransactionError::InstructionError(0, error))
}

fn custom_error(error: StakePoolError) -> Option<TransactionError> {
    instruction_error(InstructionError::Custom(error as u32))
}

// WithdrawStake and WithdrawStakeWithSlippage are gated by the Liquid
// Collective patch, so every way of presenting the authority is tested.

#[derive(Clone, Copy)]
enum Presented {
    /// The pool's authority, signing
    Correct,
    /// Another key, signing
    WrongKey,
    /// The pool's authority, but without its signature
    NotSigned,
    /// No authority account at all, as with upstream's builders
    Omitted,
}

async fn withdraw_stake_presenting(
    slippage: bool,
    presented: Presented,
) -> Option<TransactionError> {
    let (
        mut context,
        stake_pool_accounts,
        validator_stake_account,
        deposit_info,
        user_transfer_authority,
        user_stake_recipient,
        tokens_to_withdraw,
    ) = setup_for_withdraw(spl_token_interface::id(), 0).await;

    let wrong_authority = Keypair::new();
    let authority = match presented {
        Presented::WrongKey => wrong_authority.pubkey(),
        _ => stake_pool_accounts.sol_withdraw_authority.pubkey(),
    };
    let new_authority = Pubkey::new_unique();

    let mut instruction = if slippage {
        instruction::withdraw_stake_with_slippage(
            &id(),
            &stake_pool_accounts.stake_pool.pubkey(),
            &authority,
            &stake_pool_accounts.validator_list.pubkey(),
            &stake_pool_accounts.withdraw_authority,
            &validator_stake_account.stake_account,
            &user_stake_recipient.pubkey(),
            &new_authority,
            &user_transfer_authority.pubkey(),
            &deposit_info.pool_account.pubkey(),
            &stake_pool_accounts.pool_fee_account.pubkey(),
            &stake_pool_accounts.pool_mint.pubkey(),
            &stake_pool_accounts.token_program_id,
            tokens_to_withdraw,
            0,
        )
    } else {
        instruction::withdraw_stake(
            &id(),
            &stake_pool_accounts.stake_pool.pubkey(),
            &authority,
            &stake_pool_accounts.validator_list.pubkey(),
            &stake_pool_accounts.withdraw_authority,
            &validator_stake_account.stake_account,
            &user_stake_recipient.pubkey(),
            &new_authority,
            &user_transfer_authority.pubkey(),
            &deposit_info.pool_account.pubkey(),
            &stake_pool_accounts.pool_fee_account.pubkey(),
            &stake_pool_accounts.pool_mint.pubkey(),
            &stake_pool_accounts.token_program_id,
            tokens_to_withdraw,
        )
    };

    let mut signers = vec![&user_transfer_authority];
    match presented {
        Presented::Correct => signers.push(&stake_pool_accounts.sol_withdraw_authority),
        Presented::WrongKey => signers.push(&wrong_authority),
        Presented::NotSigned => instruction.accounts.last_mut().unwrap().is_signer = false,
        Presented::Omitted => {
            instruction.accounts.pop();
        }
    }
    send(&mut context, instruction, &signers).await
}

#[test_case(false; "withdraw_stake")]
#[test_case(true; "withdraw_stake_with_slippage")]
#[tokio::test]
async fn withdraw_stake_succeeds_with_the_sol_withdraw_authority(slippage: bool) {
    assert_eq!(
        withdraw_stake_presenting(slippage, Presented::Correct).await,
        None
    );
}

#[test_case(false; "withdraw_stake")]
#[test_case(true; "withdraw_stake_with_slippage")]
#[tokio::test]
async fn withdraw_stake_fails_with_a_wrong_sol_withdraw_authority(slippage: bool) {
    assert_eq!(
        withdraw_stake_presenting(slippage, Presented::WrongKey).await,
        custom_error(StakePoolError::InvalidSolWithdrawAuthority)
    );
}

#[test_case(false; "withdraw_stake")]
#[test_case(true; "withdraw_stake_with_slippage")]
#[tokio::test]
async fn withdraw_stake_fails_without_the_sol_withdraw_authority_signature(slippage: bool) {
    assert_eq!(
        withdraw_stake_presenting(slippage, Presented::NotSigned).await,
        custom_error(StakePoolError::SignatureMissing)
    );
}

#[test_case(false; "withdraw_stake")]
#[test_case(true; "withdraw_stake_with_slippage")]
#[tokio::test]
async fn withdraw_stake_fails_without_the_sol_withdraw_authority_account(slippage: bool) {
    assert_eq!(
        withdraw_stake_presenting(slippage, Presented::Omitted).await,
        instruction_error(InstructionError::NotEnoughAccountKeys)
    );
}

// Upstream's tests cover the authority of `DepositSol`, `WithdrawSol` and
// `DepositStake`. The tests below cover their `WithSlippage` variants, which
// share the processing code but are separate instructions.

/// A pool with a user who holds pool tokens, with the SOL deposit and SOL
/// withdraw authorities set
async fn setup_sol_pool() -> (
    ProgramTestContext,
    StakePoolAccounts,
    Keypair,
    Pubkey,
    u64,
    Keypair,
) {
    let mut context = program_test().start_with_context().await;
    let stake_pool_accounts = StakePoolAccounts::default();
    stake_pool_accounts
        .initialize_stake_pool(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            MINIMUM_RESERVE_LAMPORTS,
        )
        .await
        .unwrap();

    let user = Keypair::new();
    let pool_token_account = Keypair::new();
    create_token_account(
        &mut context.banks_client,
        &context.payer,
        &context.last_blockhash,
        &stake_pool_accounts.token_program_id,
        &pool_token_account,
        &stake_pool_accounts.pool_mint.pubkey(),
        &user,
        &[],
    )
    .await
    .unwrap();

    // deposit while the pool is still open, to give the user pool tokens
    let error = stake_pool_accounts
        .deposit_sol(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            &pool_token_account.pubkey(),
            TEST_STAKE_AMOUNT,
            None,
        )
        .await;
    assert!(error.is_none(), "{:?}", error);
    let pool_tokens =
        get_token_balance(&mut context.banks_client, &pool_token_account.pubkey()).await;

    let authority = Keypair::new();
    for funding_type in [FundingType::SolDeposit, FundingType::SolWithdraw] {
        let instruction = instruction::set_funding_authority(
            &id(),
            &stake_pool_accounts.stake_pool.pubkey(),
            &stake_pool_accounts.manager.pubkey(),
            Some(&authority.pubkey()),
            funding_type,
        );
        let error = send(&mut context, instruction, &[&stake_pool_accounts.manager]).await;
        assert_eq!(error, None);
    }

    (
        context,
        stake_pool_accounts,
        user,
        pool_token_account.pubkey(),
        pool_tokens,
        authority,
    )
}

#[tokio::test]
async fn deposit_sol_with_slippage_is_gated() {
    let (mut context, stake_pool_accounts, _, pool_token_account, _, authority) =
        setup_sol_pool().await;

    // upstream's builder passes no authority account
    let error = stake_pool_accounts
        .deposit_sol_with_slippage(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            &pool_token_account,
            TEST_STAKE_AMOUNT,
            0,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        Some(error),
        instruction_error(InstructionError::NotEnoughAccountKeys)
    );

    let payer = context.payer.pubkey();
    let deposit = |authority: &Pubkey| {
        instruction::deposit_sol_with_authority_and_slippage(
            &id(),
            &stake_pool_accounts.stake_pool.pubkey(),
            authority,
            &stake_pool_accounts.withdraw_authority,
            &stake_pool_accounts.reserve_stake.pubkey(),
            &payer,
            &pool_token_account,
            &stake_pool_accounts.pool_fee_account.pubkey(),
            &stake_pool_accounts.pool_fee_account.pubkey(),
            &stake_pool_accounts.pool_mint.pubkey(),
            &stake_pool_accounts.token_program_id,
            TEST_STAKE_AMOUNT,
            0,
        )
    };
    let wrong_authority = Keypair::new();
    let wrong = deposit(&wrong_authority.pubkey());
    assert_eq!(
        send(&mut context, wrong, &[&wrong_authority]).await,
        custom_error(StakePoolError::InvalidSolDepositAuthority)
    );

    // control: the same instruction succeeds with the authority
    let right = deposit(&authority.pubkey());
    assert_eq!(send(&mut context, right, &[&authority]).await, None);
}

#[tokio::test]
async fn withdraw_sol_with_slippage_is_gated() {
    let (mut context, stake_pool_accounts, user, pool_token_account, pool_tokens, authority) =
        setup_sol_pool().await;

    // upstream's builder passes no authority account
    let error = stake_pool_accounts
        .withdraw_sol_with_slippage(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            &user,
            &pool_token_account,
            pool_tokens,
            0,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        Some(error),
        instruction_error(InstructionError::NotEnoughAccountKeys)
    );

    let withdraw = |authority: &Pubkey| {
        instruction::withdraw_sol_with_authority_and_slippage(
            &id(),
            &stake_pool_accounts.stake_pool.pubkey(),
            authority,
            &stake_pool_accounts.withdraw_authority,
            &user.pubkey(),
            &pool_token_account,
            &stake_pool_accounts.reserve_stake.pubkey(),
            &user.pubkey(),
            &stake_pool_accounts.pool_fee_account.pubkey(),
            &stake_pool_accounts.pool_mint.pubkey(),
            &stake_pool_accounts.token_program_id,
            pool_tokens,
            0,
        )
    };
    let wrong_authority = Keypair::new();
    let wrong = withdraw(&wrong_authority.pubkey());
    assert_eq!(
        send(&mut context, wrong, &[&user, &wrong_authority]).await,
        custom_error(StakePoolError::InvalidSolWithdrawAuthority)
    );

    // control: the same instruction succeeds with the authority
    let right = withdraw(&authority.pubkey());
    assert_eq!(send(&mut context, right, &[&user, &authority]).await, None);
}

#[tokio::test]
async fn deposit_stake_with_slippage_is_gated() {
    let mut context = program_test().start_with_context().await;
    // the pool restricts stake deposits to this key
    let stake_deposit_authority = Keypair::new();
    let stake_pool_accounts =
        StakePoolAccounts::new_with_deposit_authority(stake_deposit_authority);
    stake_pool_accounts
        .initialize_stake_pool(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            MINIMUM_RESERVE_LAMPORTS,
        )
        .await
        .unwrap();
    let validator_stake_account = simple_add_validator_to_pool(
        &mut context.banks_client,
        &context.payer,
        &context.last_blockhash,
        &stake_pool_accounts,
        stake_pool_accounts.stake_deposit_authority_keypair.as_ref(),
    )
    .await;

    let user = Keypair::new();
    let user_stake = Keypair::new();
    create_independent_stake_account(
        &mut context.banks_client,
        &context.payer,
        &context.last_blockhash,
        &user_stake,
        &stake::state::Authorized {
            staker: user.pubkey(),
            withdrawer: user.pubkey(),
        },
        &stake::state::Lockup::default(),
        TEST_STAKE_AMOUNT,
    )
    .await;
    delegate_stake_account(
        &mut context.banks_client,
        &context.payer,
        &context.last_blockhash,
        &user_stake.pubkey(),
        &user,
        &validator_stake_account.vote.pubkey(),
    )
    .await;
    let user_pool_account = Keypair::new();
    create_token_account(
        &mut context.banks_client,
        &context.payer,
        &context.last_blockhash,
        &stake_pool_accounts.token_program_id,
        &user_pool_account,
        &stake_pool_accounts.pool_mint.pubkey(),
        &user,
        &[],
    )
    .await
    .unwrap();

    // upstream's builder uses the pool's default deposit authority, which is
    // not the key the pool is restricted to
    let error = stake_pool_accounts
        .deposit_stake_with_slippage(
            &mut context.banks_client,
            &context.payer,
            &context.last_blockhash,
            &user_stake.pubkey(),
            &user_pool_account.pubkey(),
            &validator_stake_account.stake_account,
            &user,
            0,
        )
        .await
        .unwrap()
        .unwrap();
    match error {
        TransactionError::InstructionError(_, InstructionError::Custom(error_index)) => {
            assert_eq!(
                error_index,
                StakePoolError::InvalidStakeDepositAuthority as u32
            );
        }
        _ => panic!("Wrong error: {:?}", error),
    }
}
