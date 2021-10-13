use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum EscrowState {
    Unintialized,
    Initialized,
    DepositAlice,
    DepositBob,
    Committed,
    WithdrawAlice,
    WithdrawBob,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EscrowData {
    pub size_x: u64,
    pub size_y: u64,
    pub pubkey_alice: Pubkey,
    pub pubkey_bob: Pubkey,
    pub pubkey_mint_x: Pubkey,
    pub pubkey_mint_y: Pubkey,
    pub state: EscrowState,
    pub escrow_bump: u8,
    pub vault_x_bump: u8,
    pub vault_y_bump: u8,
}

impl EscrowData {
    pub const LEN: usize = 8 // size_x
    + 8 // size_y
    + 32 // pubkey_alice
    + 32 // pubkey_bob
    + 32 // pubkey_mint_x
    + 32 // pubkey_mint_y
    + 1 // state
    + 1 // escrow_bump
    + 1 // vault_x_bump
    + 1 // vault_y_bump
    ;
}