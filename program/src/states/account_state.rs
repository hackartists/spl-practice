use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct AccountState {
    pub counter: u32,
}

impl super::State for AccountState {
    fn size() -> usize {
        4
    }

    fn state_type() -> &'static str {
        "account_state"
    }
}
