
Solana based on chain escrow project

cargo build-bpf --bpf-out-dir=dist/program
solana program deploy dist/program/escrow.so --keypair <key_pair.json>

After these steps, a Program ID is generated; store that.

## Initialization
python3 client/setup2.py --pid <program_id> \
--payerKP <alice_key_pair.json> \
--apubk <alice_public_key> \
--bpubk <bob_public_key> \
--user alice \
--op_type init

In our case only alice can initiate. So, `payerKP=alice_key_pair.json`. This step generates 

- alice_x_token_account_public_key,
- alice_y_token_account_public_key, 
- bob_x_token_account_public_key, 
- bob_y_token_account_public_key, 
- escrow_address

and store it locally. Send these to Bob for transactions of deposit/withdraw.

## Deposit
python3 client/setup2.py --pid <program_id> \
--userKP <user_key_pair.json> \
--apubk <alice_public_key> \
--bpubk <bob_public_key> \
--op_type deposit \
--escrow <escrow_address> \
--token <user_token_account_public_key> \
--user <user>
  
`user_token_account_public_key`: public key of token account from where the tokens are deposited. \
For example: if Alice wants to send X tokens, then `user_token_account_public_key = alice_x_token_account_public_key`. 
  
## Withdraw
python3 client/setup2.py --pid <program_id> \
--userKP <user_key_pair.json> \
--apubk <alice_public_key> \
--bpubk <bob_public_key> \
--op_type deposit \
--escrow <escrow_address> \
--token <user_token_account_public_key> \
--user <user>  

`user_token_account_public_key`: public key of token account to where the tokens are submitted. \
For example: if Alice wants Y tokens from Bob, then `user_token_account_public_key = alice_y_token_account_public_key`
