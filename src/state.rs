use solana_program::{
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EscrowData {
    pub xval: u64,
    pub yval: u64,
    pub a_pub_key: Pubkey,
    pub b_pub_key: Pubkey,
    pub mint_x_pub_key: Pubkey,
    pub mint_y_pub_key: Pubkey,
    pub vault_x_pub_key: Pubkey,
    pub vault_y_pub_key: Pubkey,
    pub init_deposit_status: u64,
    pub is_a_withdrawed: u8,
    pub is_b_withdrawed: u8,
}