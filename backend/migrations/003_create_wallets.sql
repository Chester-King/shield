-- Create wallets table for Zcash wallet data
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    encrypted_mnemonic TEXT NOT NULL, -- BIP39 mnemonic (encrypted in production)
    address TEXT NOT NULL, -- Unified address (shielded)
    transparent_address TEXT, -- Transparent address (public, like Bitcoin)
    birthday_height BIGINT NOT NULL, -- Block height when wallet was created
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ,
    last_synced_height BIGINT
);

-- Create index on user_id (one wallet per user for now)
CREATE UNIQUE INDEX idx_wallets_user_id ON wallets(user_id);

-- Create updated_at trigger
CREATE TRIGGER update_wallets_updated_at BEFORE UPDATE ON wallets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
