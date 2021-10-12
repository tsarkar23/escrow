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
from spl.token.constants import TOKEN_PROGRAM_ID
from solana import keypair
from solana.publickey import PublicKey
from solana.sysvar import SYSVAR_RENT_PUBKEY

# Payer account
http_client = Client("https://api.devnet.solana.com")
first_key_in_key_folder = glob.glob("../keys/M*")[0]
payer_loaded_account = solana.keypair.Keypair(json.load(open(first_key_in_key_folder))[:32])
payer_public_key = payer_loaded_account.public_key

# Create alice account
alice_keypair = solana.keypair.Keypair.generate()
alice_pubkey = alice_keypair.public_key

# Create bob account
bob_keypair = solana.keypair.Keypair.generate()
bob_pubkey = bob_keypair.public_key

print(f"Payer Account: {payer_public_key}")
print(f"alice Account: {alice_pubkey}")
print(f"bob Account: {bob_pubkey}")

http_client.request_airdrop(payer_public_key, 1000000000)
time.sleep(20)
http_client.request_airdrop(alice_pubkey, 1000000000)
time.sleep(20)
http_client.request_airdrop(bob_pubkey, 1000000000)
# mention desgin choice between alice/bob vs payer

# Get program id
deployed_program_key = glob.glob("../dist/program/*.json")[0]
deployed_program_key_account = solana.keypair.Keypair(json.load(open(deployed_program_key))[:32])
program_id = deployed_program_key_account.public_key

# Writing keys to file
with open("../keys/alice_pubkey.json", 'w') as a:
    json.dump(str(alice_pubkey), a)

with open("../keys/bob_pubkey.json", 'w') as a:
    json.dump(str(bob_pubkey), a)

with open("../keys/program_id.json", 'w') as a:
    json.dump(str(program_id), a)
