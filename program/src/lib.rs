use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // use spl_token::{error::TokenError, processor::Processor};
    // if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
    //     // catch the error so we can print it
    //     error.print::<TokenError>();
    //     return Err(error);
    // }
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut greeting = AccountState::try_from_slice(&account.data.borrow())?;
    greeting.counter += 1;
    greeting.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct AccountState {
    pub counter: u32,
}

#[cfg(feature = "app")]
impl AccountState {
    pub fn get_pda(
        program_id: &solana_sdk::pubkey::Pubkey,
        user: &solana_sdk::pubkey::Pubkey,
    ) -> Result<solana_sdk::pubkey::Pubkey, solana_sdk::pubkey::PubkeyError> {
        Pubkey::create_with_seed(user, "account_state", program_id)
    }

    pub fn get_or_create_account(
        cli: &solana_client::rpc_client::RpcClient,
        program_id: &solana_sdk::pubkey::Pubkey,
        user: &solana_sdk::signature::Keypair,
        payer: &solana_sdk::signature::Keypair,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use solana_sdk::signer::Signer;

        let pda = Self::get_pda(program_id, &user.pubkey())?;
        Ok(match cli.get_account(&pda) {
            Ok(account) => Self::try_from_slice(&account.data),
            Err(_) => {
                Self::create_account(cli, program_id, user, payer, &pda, "account_state")?;
                let account = cli.get_account(&pda)?;
                Self::try_from_slice(&account.data)
            }
        }?)
    }

    fn create_account(
        cli: &solana_client::rpc_client::RpcClient,
        program_id: &solana_sdk::pubkey::Pubkey,
        user: &solana_sdk::signature::Keypair,
        payer: &solana_sdk::signature::Keypair,
        pda: &solana_sdk::pubkey::Pubkey,
        data_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use solana_sdk::signer::Signer;

        let rent_fee = cli.get_minimum_balance_for_rent_exemption(4)?;
        let create_account_instruction = solana_sdk::system_instruction::create_account_with_seed(
            &payer.pubkey(),
            pda,
            &user.pubkey(),
            data_type,
            rent_fee,
            4,
            program_id,
        );

        let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[create_account_instruction],
            Some(&payer.pubkey()),
            &[payer, user],
            cli.get_latest_blockhash()?,
        );
        cli.send_and_confirm_transaction(&transaction)?;

        Ok(())
    }
}
