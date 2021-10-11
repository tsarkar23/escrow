#!/bin/sh

cargo build-bpf && solana program deploy /Users/mhshafinia/Workplaces/rust/escrow/target/deploy/escrow.so  --keypair keys/MYqwmvi4sXdWaCTMHAQw2Yb4ZtfteXG3JiMtvR2kCyQ.json
