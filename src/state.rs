/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct EscrowData {
    xval: u64,
    yval: u64,
    a_pub_key: Pubkey,
    b_pub_key: Pubkey,
    mint_x_pub_key: Pubkey,
    mint_y_pub_key: Pubkey,
    vault_x_pub_key: Pubkey,
    vault_y_pub_key: Pubkey,
    init_deposit_status: u64,
    is_a_withdrawed: u8,
    is_b_withdrawed: u8,
}