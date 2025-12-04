-- Create blockchain data tables to store transaction history in PostgreSQL
-- This replaces the per-user SQLite files for better scalability

-- Transactions table - stores all wallet transactions
CREATE TABLE IF NOT EXISTS transactions (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    txid TEXT NOT NULL,
    block_height BIGINT,
    tx_index INTEGER,
    created_at TIMESTAMPTZ,
    mined_at TIMESTAMPTZ,
    expiry_height BIGINT,
    fee_zatoshis BIGINT,
    UNIQUE(user_id, txid)
);

-- Received notes table - stores all notes received by the wallet
CREATE TABLE IF NOT EXISTS received_notes (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    transaction_id BIGINT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    note_index INTEGER NOT NULL,
    value_zatoshis BIGINT NOT NULL,
    memo BYTEA,
    is_change BOOLEAN NOT NULL DEFAULT false,
    spent_in_tx_id BIGINT REFERENCES transactions(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, transaction_id, note_index)
);

-- Sent notes table - stores all notes sent by the wallet
CREATE TABLE IF NOT EXISTS sent_notes (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    transaction_id BIGINT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    to_address TEXT NOT NULL,
    value_zatoshis BIGINT NOT NULL,
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions(user_id, block_height DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_received_notes_user_id ON received_notes(user_id);
CREATE INDEX IF NOT EXISTS idx_received_notes_transaction ON received_notes(transaction_id);
CREATE INDEX IF NOT EXISTS idx_received_notes_spent ON received_notes(spent_in_tx_id);
CREATE INDEX IF NOT EXISTS idx_sent_notes_user_id ON sent_notes(user_id);
CREATE INDEX IF NOT EXISTS idx_sent_notes_transaction ON sent_notes(transaction_id);
