use borsh::{BorshDeserialize, BorshSerialize};

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
        solana_sdk::pubkey::Pubkey::create_with_seed(user, "account_state", program_id)
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
