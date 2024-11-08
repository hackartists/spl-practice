use solana_program::pubkey::{Pubkey, PubkeyError};

pub mod account_state;

pub trait State {
    fn size() -> usize;
    fn state_type() -> &'static str;
    fn get_pda(program_id: &Pubkey, user: &Pubkey) -> Result<Pubkey, PubkeyError> {
        Pubkey::create_with_seed(user, Self::state_type(), program_id)
    }
}
