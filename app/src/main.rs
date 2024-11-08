use std::error::Error;

use borsh::{BorshDeserialize, BorshSerialize};
use expiry_token::{
    states::{account_state::AccountState, State},
    CustomInstruction,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let cli = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    let program_id = Pubkey::try_from(
        option_env!("PROGRAM_ID").expect("You must set `PROGRAM_ID` environment"),
    )?;

    let payer =
        read_keypair_file(option_env!("PAYER_KEYPAIR_FILE").unwrap_or("~/.config/solana/id.json"))?;

    let user = match option_env!("USER_KEYPAIR") {
        Some(base58) => Keypair::from_base58_string(base58),
        None => {
            let k = Keypair::new();
            let inst = system_instruction::transfer(&payer.pubkey(), &k.pubkey(), 1_000_000_000);
            let mut tx = Transaction::new_with_payer(&[inst], Some(&payer.pubkey()));
            tx.sign(&[&payer], cli.get_latest_blockhash()?);

            cli.send_and_confirm_transaction(&tx)?;

            println!("Use below command next time");
            println!("USER_KEYPAIR={} make run", k.to_base58_string());
            k
        }
    };

    let account_state: AccountState = get_or_create_account(&cli, &program_id, &user, &payer)?;
    println!("Counter: {:?}", account_state.counter);
    let pda = AccountState::get_pda(&program_id, &user.pubkey())?;

    say_hello(&cli, &program_id, &user, &pda, &payer)?;

    let account_state: AccountState = get_or_create_account(&cli, &program_id, &user, &payer)?;
    println!("Counter: {:?}", account_state.counter);

    sub(&cli, &program_id, &user, &pda, &payer)?;

    let account_state: AccountState = get_or_create_account(&cli, &program_id, &user, &payer)?;
    println!("Counter: {:?}", account_state.counter);
    Ok(())
}

fn say_hello(
    cli: &RpcClient,
    program_id: &Pubkey,
    user: &Keypair,
    pda: &Pubkey,
    payer: &Keypair,
) -> Result<()> {
    let mut writer = &mut vec![];
    CustomInstruction::Add(10).serialize(&mut writer)?;
    let instruction = Instruction::new_with_bytes(
        program_id.clone(),
        &writer,
        vec![
            AccountMeta::new(pda.clone(), false),
            AccountMeta::new_readonly(user.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer, user],
        cli.get_latest_blockhash()?,
    );
    cli.send_and_confirm_transaction(&tx)?;

    Ok(())
}

fn sub(
    cli: &RpcClient,
    program_id: &Pubkey,
    user: &Keypair,
    pda: &Pubkey,
    payer: &Keypair,
) -> Result<()> {
    let mut writer = &mut vec![];
    CustomInstruction::Sub(4).serialize(&mut writer)?;
    let instruction = Instruction::new_with_bytes(
        program_id.clone(),
        &writer,
        vec![
            AccountMeta::new(pda.clone(), false),
            AccountMeta::new_readonly(user.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer, user],
        cli.get_latest_blockhash()?,
    );
    cli.send_and_confirm_transaction(&tx)?;

    Ok(())
}

pub fn get_or_create_account<T>(
    cli: &solana_client::rpc_client::RpcClient,
    program_id: &solana_sdk::pubkey::Pubkey,
    user: &solana_sdk::signature::Keypair,
    payer: &solana_sdk::signature::Keypair,
) -> Result<T>
where
    T: State + BorshDeserialize + BorshSerialize,
{
    use solana_sdk::signer::Signer;

    let pda = AccountState::get_pda(program_id, &user.pubkey())?;

    Ok(match cli.get_account(&pda) {
        Ok(account) => T::try_from_slice(&account.data),
        Err(_) => {
            create_account::<T>(cli, program_id, user, payer, &pda, T::state_type())?;
            let account = cli.get_account(&pda)?;
            T::try_from_slice(&account.data)
        }
    }?)
}

fn create_account<T>(
    cli: &solana_client::rpc_client::RpcClient,
    program_id: &solana_sdk::pubkey::Pubkey,
    user: &solana_sdk::signature::Keypair,
    payer: &solana_sdk::signature::Keypair,
    pda: &solana_sdk::pubkey::Pubkey,
    data_type: &str,
) -> Result<()>
where
    T: State,
{
    use solana_sdk::signer::Signer;

    let size: usize = T::size();
    let rent_fee = cli.get_minimum_balance_for_rent_exemption(size)?;
    let create_account_instruction = solana_sdk::system_instruction::create_account_with_seed(
        &payer.pubkey(),
        pda,
        &user.pubkey(),
        data_type,
        rent_fee,
        size as u64,
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
