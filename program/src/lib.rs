use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use states::account_state::AccountState;

pub mod states {
    pub mod account_state;
}

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // use spl_token::{error::TokenError, processor::Processor};
    // if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
    //     // catch the error so we can print it
    //     error.print::<TokenError>();
    //     return Err(error);
    // }
    if instruction_data.is_empty() {
        return Ok(());
    }

    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut greeting = AccountState::try_from_slice(&account.data.borrow())?;
    match CustomInstruction::try_from_slice(instruction_data)? {
        CustomInstruction::Add(value) => greeting.counter += value,
        CustomInstruction::Sub(value) => greeting.counter -= value,
    };

    greeting.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum CustomInstruction {
    Add(u32),
    Sub(u32),
}
