-- Add auth_method column to users table to distinguish between Google OAuth and email/password authentication

-- Create auth method enum type
CREATE TYPE auth_method AS ENUM ('google', 'email');

-- Add auth_method column (default to 'google' for existing users)
ALTER TABLE users
ADD COLUMN auth_method auth_method NOT NULL DEFAULT 'google';

-- Make password_hash nullable since Google OAuth users won't have passwords
ALTER TABLE users
ALTER COLUMN password_hash DROP NOT NULL;

-- Add index on auth_method for faster queries
CREATE INDEX idx_users_auth_method ON users(auth_method);
