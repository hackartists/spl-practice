use std::error::Error;

use borsh::BorshDeserialize;
use expiry_token::AccountState;
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

    let pda = get_pda_account(&program_id, &user.pubkey(), "greeting")?;
    if let Err(_) = cli.get_account(&pda) {
        create_account(&cli, &program_id, &payer, &user, &pda, "greeting")?;
    }
    say_hello(&cli, &program_id, &user, &pda, &payer)?;

    let counter = cli.get_account(&pda)?;
    println!(
        "Counter: {:?}",
        AccountState::try_from_slice(&counter.data)?.counter
    );
    Ok(())
}

fn say_hello(
    cli: &RpcClient,
    program_id: &Pubkey,
    user: &Keypair,
    pda: &Pubkey,
    payer: &Keypair,
) -> Result<()> {
    let instruction = Instruction::new_with_bytes(
        program_id.clone(),
        &[],
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

fn create_account(
    cli: &RpcClient,
    program_id: &Pubkey,
    payer: &Keypair,
    user: &Keypair,
    pda: &Pubkey,
    data_type: &str,
) -> Result<()> {
    let rent_fee = cli.get_minimum_balance_for_rent_exemption(4)?;
    let create_account_instruction = system_instruction::create_account_with_seed(
        &payer.pubkey(),
        pda,
        &user.pubkey(),
        data_type,
        rent_fee,
        4,
        program_id,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction],
        Some(&payer.pubkey()),
        &[payer, user],
        cli.get_latest_blockhash()?,
    );
    cli.send_and_confirm_transaction(&transaction)?;

    println!("Created account: {:?}", pda);
    Ok(())
}

fn get_pda_account(program_id: &Pubkey, user: &Pubkey, data_type: &str) -> Result<Pubkey> {
    Ok(Pubkey::create_with_seed(user, data_type, program_id)?)
}
