# accounts
import json
import solana
import time
import sys
import glob
from solana.rpc.api import Client
from solana.blockhash import Blockhash
from solana.publickey import PublicKey
from solana.system_program import CreateAccountWithSeedParams, create_account_with_seed, CreateAccountParams, create_account
from solana.transaction import Transaction, AccountMeta, TransactionInstruction
from solana.system_program import SYS_PROGRAM_ID
from spl.token.client import Token
from solana import keypair
from solana.publickey import PublicKey
from solana.sysvar import SYSVAR_RENT_PUBKEY
from struct import *
import struct


TOKEN_PROGRAM_ID: PublicKey = PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")


payer_loaded_account = solana.keypair.Keypair.generate()
payer_public_key = payer_loaded_account.public_key

http_client = Client("https://api.devnet.solana.com")

# todo: change program_id to system program, change data size =0, change lampord for both alice and bob
alice_keypair = solana.keypair.Keypair.generate()
alice_pubkey = alice_keypair.public_key

bob_keypair = solana.keypair.Keypair.generate()
bob_pubkey = bob_keypair.public_key

print(f"Payer Account: {payer_public_key}")
print(f"alice Account: {alice_pubkey}")
print(f"bob Account: {bob_pubkey}")

import time
http_client.request_airdrop(payer_loaded_account.public_key, 1000000000)
time.sleep(15)
http_client.request_airdrop(alice_pubkey, 1000000000)
time.sleep(15)
http_client.request_airdrop(bob_pubkey, 1000000000)
print(http_client.get_balance(alice_pubkey))


X_mint_account_address = Token.create_mint(conn= http_client, payer = payer_loaded_account,\
                                           mint_authority = payer_loaded_account.public_key,\
                                           decimals = 0, program_id = TOKEN_PROGRAM_ID, )
Y_mint_account_address = Token.create_mint(conn= http_client, payer = payer_loaded_account,\
                                           mint_authority = payer_loaded_account.public_key,\
                                           decimals = 0, program_id = TOKEN_PROGRAM_ID, )
alice_X_token_account = X_mint_account_address.create_associated_token_account(alice_pubkey)
alice_Y_token_account = Y_mint_account_address.create_associated_token_account(alice_pubkey)
bob_X_token_account = X_mint_account_address.create_associated_token_account(bob_pubkey)
bob_Y_token_account = Y_mint_account_address.create_associated_token_account(bob_pubkey)

X_mint_account_address.mint_to(alice_X_token_account,payer_loaded_account,1000)
Y_mint_account_address.mint_to(bob_Y_token_account,payer_loaded_account,100)

# escrow meta-data account
deployed_program_key = glob.glob("../dist/program/*.json")[0]
deployed_program_key_account = solana.keypair.Keypair(json.load(open(deployed_program_key))[:32])
program_id = deployed_program_key_account.public_key

# Seed+password for sending data
password = (20).to_bytes(8,byteorder='little')
x_seeds = [
    b"vault_x",
    password,
    bytes(alice_pubkey),
    bytes(bob_pubkey),
    bytes(X_mint_account_address.pubkey),
    bytes(Y_mint_account_address.pubkey),
]

y_seeds = [
    b"vault_y",
    password,
    bytes(alice_pubkey),
    bytes(bob_pubkey),
    bytes(X_mint_account_address.pubkey),
    bytes(Y_mint_account_address.pubkey),
]

escrow_seeds = [
    b"escrow",
    password,
    bytes(alice_pubkey),
    bytes(bob_pubkey),
    bytes(X_mint_account_address.pubkey),
    bytes(Y_mint_account_address.pubkey),
]

# create vaults, necessary?
vaultx, xseed = PublicKey.find_program_address(seeds=x_seeds,program_id=program_id)
vaulty, yseed = PublicKey.find_program_address(seeds=y_seeds,program_id=program_id)
escrow_address, escrow_seed = PublicKey.find_program_address(seeds=escrow_seeds,program_id=program_id)

print('Start Initializing Accounts')
# initialize transaction
x_val = 13
y_val = 15
data = pack('<BQQ', 0,x_val,y_val)+password
tx = Transaction()
tx_instruction = TransactionInstruction(
    keys=[AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
          AccountMeta(pubkey=X_mint_account_address.pubkey, is_signer=False, is_writable=False),
          AccountMeta(pubkey=Y_mint_account_address.pubkey, is_signer=False, is_writable=False),
          AccountMeta(pubkey=vaultx, is_signer=False, is_writable=True),
          AccountMeta(pubkey=vaulty, is_signer=False, is_writable=True),
          AccountMeta(pubkey=payer_public_key, is_signer=True, is_writable=False),
          AccountMeta(pubkey=alice_pubkey, is_signer=False, is_writable=False),
          AccountMeta(pubkey=bob_pubkey, is_signer=False, is_writable=False),
          AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
          AccountMeta(pubkey=SYSVAR_RENT_PUBKEY, is_signer=False, is_writable=False),
          AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
         ],
    program_id=program_id,
    data=data,
)

tx = tx.add(tx_instruction)
transaction_results = http_client.send_transaction(tx, *[payer_loaded_account])#, *[payer_loaded_account])
print('Initializing Accounts Ended')

time.sleep(30)

# Alice deposit X
print('Alice is depositing X')
data = pack('<B', 1)
tx = Transaction()
tx_instruction = TransactionInstruction(
    keys=[AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
          AccountMeta(pubkey=alice_X_token_account, is_signer=False, is_writable=True), # x_a info
          AccountMeta(pubkey=vaultx, is_signer=False, is_writable=True),
          AccountMeta(pubkey=alice_pubkey, is_signer=True, is_writable=False),
          AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
         ],
    program_id=program_id,
    data=  data,
)

tx = tx.add(tx_instruction)
transaction_results = http_client.send_transaction(tx, *[alice_keypair])

time.sleep(30)

print('Bob is depositing Y')
# Bob deposit Y
data = pack('<B', 1)
tx = Transaction()
tx_instruction = TransactionInstruction(
    keys=[AccountMeta(pubkey=escrow_address, is_signer=False, is_writable=True),
          AccountMeta(pubkey=bob_Y_token_account, is_signer=False, is_writable=True),
          AccountMeta(pubkey=vaulty, is_signer=False, is_writable=True),
          AccountMeta(pubkey=bob_pubkey, is_signer=True, is_writable=False),
          AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
         ],
    program_id=program_id,
    data=  data,
)

tx = tx.add(tx_instruction)


transaction_results = http_client.send_transaction(tx, *[bob_keypair])
