import base58

# Replace with your mint addresses
usdt_mint_str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
usdc_mint_str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

# Convert base-58 to byte array
usdt_mint_bytes = base58.b58decode(usdt_mint_str)
usdc_mint_bytes = base58.b58decode(usdc_mint_str)

print("USDT Mint Byte Array:", list(usdt_mint_bytes))
print("USDC Mint Byte Array:", list(usdc_mint_bytes))
