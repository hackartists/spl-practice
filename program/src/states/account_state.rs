use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::{Pubkey, PubkeyError};

#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct AccountState {
    pub counter: u32,
}

impl AccountState {
    pub fn size() -> usize {
        4
    }

    pub fn state_type() -> &'static str {
        "account_state"
    }

    pub fn get_pda(program_id: &Pubkey, user: &Pubkey) -> Result<Pubkey, PubkeyError> {
        Pubkey::create_with_seed(user, AccountState::state_type(), program_id)
    }
}
