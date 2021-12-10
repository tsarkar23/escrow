use borsh::{BorshSerialize, BorshDeserialize};


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum EscrowInstruction {
    /// Starts the trade by creating and populating an escrow account and transferring ownership of the given temp token account to the PDA
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the escrow
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The token program
    InitEscrow {
        amount_x: u64, //amounts[0]:x_val, amounts[1]:y_val, amounts[2]:pass
        amount_y: u64,
        pass: [u8; 32],
    },
    Deposit{
        pass: [u8; 32],
    },
    Withdrawal {
        pass: [u8; 32],
    },
}