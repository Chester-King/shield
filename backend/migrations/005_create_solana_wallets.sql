-- Create solana_wallets table to store Solana keypairs
CREATE TABLE solana_wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    encrypted_keypair BYTEA NOT NULL,      -- Ed25519 keypair (64 bytes) - currently unencrypted
    public_key TEXT NOT NULL UNIQUE,        -- Base58 encoded Solana address
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)  -- One Solana wallet per user
);

CREATE INDEX idx_solana_wallets_user_id ON solana_wallets(user_id);
CREATE INDEX idx_solana_wallets_public_key ON solana_wallets(public_key);

-- Create bridge_transactions table to track SOL → ZEC swaps via NEAR Intents
CREATE TABLE bridge_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Transaction details
    solana_tx_signature TEXT,               -- Solana transaction signature
    deposit_address TEXT NOT NULL,          -- NEAR Intents deposit address
    amount_sol_lamports BIGINT NOT NULL,    -- Amount in lamports (1 SOL = 1,000,000,000 lamports)
    expected_zec_zatoshis BIGINT,           -- Expected ZEC output in zatoshis

    -- Bridge status tracking
    status TEXT NOT NULL DEFAULT 'PENDING', -- PENDING, PROCESSING, SUCCESS, FAILED, REFUNDED
    error_message TEXT,                      -- Error details if failed

    -- NEAR Intents data
    near_intent_hashes TEXT[],               -- Array of NEAR intent transaction hashes
    near_tx_hashes TEXT[],                   -- Array of NEAR transaction hashes

    -- Destination transaction
    zec_tx_hash TEXT,                        -- Zcash transaction hash when complete
    actual_zec_zatoshis BIGINT,             -- Actual ZEC received

    -- Recipient addresses
    refund_address TEXT NOT NULL,            -- Solana address for refunds
    recipient_address TEXT NOT NULL,         -- Zcash shielded address

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ                -- When bridge completed (success or failure)
);

CREATE INDEX idx_bridge_transactions_user_id ON bridge_transactions(user_id);
CREATE INDEX idx_bridge_transactions_status ON bridge_transactions(status);
CREATE INDEX idx_bridge_transactions_deposit_address ON bridge_transactions(deposit_address);
CREATE INDEX idx_bridge_transactions_created_at ON bridge_transactions(created_at DESC);

-- Add trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_solana_wallets_updated_at
    BEFORE UPDATE ON solana_wallets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_bridge_transactions_updated_at
    BEFORE UPDATE ON bridge_transactions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
