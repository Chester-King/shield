# Shield - Zcash Embedded Wallet

A production-ready web-based embedded wallet for Zcash shielded transactions. Shield provides a secure, privacy-focused wallet solution with seamless authentication and real-time fee calculation.

## Features

### Wallet Management
- **Automatic Wallet Creation**: Wallets are automatically created on first login using BIP-39 mnemonic phrases
- **Shielded Addresses**: Full support for Zcash unified addresses (Orchard + Sapling)
- **Birthday Height Optimization**: Wallets track birthday height for faster blockchain scanning
- **Balance Tracking**: Real-time balance updates with blockchain synchronization

### Transactions
- **Send ZEC**: Send shielded transactions with memo support
- **Real-time Fee Calculation**: Accurate ZIP-317 fee estimation using Zcash library
- **Transaction Confirmation**: Two-step confirmation flow with balance validation
- **Transaction History**: Complete history with sent/received status and memo display
- **Block Explorer Integration**: Direct links to Zcash block explorers

### Authentication
- **Google OAuth**: Secure authentication via Google OAuth 2.0
- **JWT Tokens**: Session management with JWT access tokens
- **Auto-redirect**: Expired sessions automatically redirect to login

### User Interface
- **Modern Design**: Clean, responsive UI built with Next.js and Tailwind CSS
- **Transaction History Sidebar**: Slide-out sidebar for viewing transaction history
- **Profile Management**: User profile sidebar with account details
- **Loading States**: Proper loading indicators for all async operations
- **Error Handling**: User-friendly error messages with detailed feedback

## Project Structure

```
Shield-Infra/
├── frontend/              # Next.js frontend application
│   ├── app/              # Next.js app router pages
│   ├── components/       # React components
│   ├── contexts/         # React contexts (Auth)
│   ├── lib/             # API clients and utilities
│   └── public/          # Static assets
├── backend/              # Rust backend with Axum
│   ├── src/
│   │   ├── handlers/    # API endpoint handlers
│   │   ├── middleware/  # Auth middleware
│   │   ├── models/      # Database models
│   │   └── zcash/       # Zcash wallet implementation
│   ├── migrations/      # Database migrations
│   └── Cargo.toml
├── docker-compose.yml    # PostgreSQL database setup
└── README.md
```

## Tech Stack

### Frontend
- **Next.js 15**: React framework with app router
- **React 19**: Latest React with Server Components
- **TypeScript**: Type-safe development
- **Tailwind CSS**: Utility-first styling
- **Framer Motion**: Smooth animations
- **Lucide Icons**: Modern icon library

### Backend
- **Rust**: High-performance, memory-safe backend
- **Axum**: Fast, ergonomic web framework
- **SQLx**: Async SQL toolkit with compile-time query checking
- **Tokio**: Async runtime
- **zcash_client_backend**: Official Zcash wallet SDK
- **zcash_client_sqlite**: Local wallet database
- **zcash_primitives**: Core Zcash primitives

### Database
- **PostgreSQL 16**: Relational database for user data
- **SQLite**: Per-user wallet databases for Zcash data

### Infrastructure
- **Docker**: Containerized PostgreSQL
- **Lightwalletd**: Zcash light client protocol server

## Setup Instructions

### Prerequisites
- **Node.js 20+**: For frontend development
- **Rust (latest stable)**: For backend compilation
- **Docker and Docker Compose**: For PostgreSQL
- **Google OAuth Credentials**: For authentication

### 1. Start PostgreSQL Database

```bash
docker-compose up -d
```

This starts PostgreSQL on port 5432 with:
- Database: `shield`
- User: `postgres`
- Password: `password`

### 2. Configure Environment Variables

#### Backend `.env`
Create `backend/.env`:
```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/shield
PORT=8000

# Google OAuth (get from Google Cloud Console)
GOOGLE_CLIENT_ID=your_google_client_id
GOOGLE_CLIENT_SECRET=your_google_client_secret
GOOGLE_REDIRECT_URI=http://localhost:3000/auth/callback

# JWT Secret (generate a random string)
JWT_SECRET=your_jwt_secret_key

# Zcash Network (mainnet or testnet)
ZCASH_NETWORK=mainnet

# Lightwalletd Servers
LIGHTWALLETD_MAINNET=https://na.zec.rocks:443
LIGHTWALLETD_TESTNET=https://testnet.zec.rocks:443
```

#### Frontend `.env.local`
Create `frontend/.env.local`:
```env
NEXT_PUBLIC_API_URL=http://localhost:8000/api

# Google OAuth (same client ID as backend)
NEXT_PUBLIC_GOOGLE_CLIENT_ID=your_google_client_id
```

### 3. Setup Backend

```bash
cd backend

# Run database migrations
sqlx migrate run

# Build and run
cargo build --release
cargo run --release
```

The backend server will run on `http://localhost:8000`

### 4. Setup Frontend

```bash
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev
```

The frontend will run on `http://localhost:3000`

## API Endpoints

### Authentication
- `POST /api/auth/google` - Initiate Google OAuth flow
- `GET /api/auth/callback` - OAuth callback handler
- `POST /api/auth/verify` - Verify JWT token

### Wallet Management
- `POST /api/wallet/address` - Get wallet address (creates wallet if needed)
- `POST /api/wallet/balance` - Get wallet balance with blockchain sync

### Transactions
- `POST /api/wallet/estimate-fee` - Estimate transaction fee in real-time
- `POST /api/wallet/send` - Send shielded transaction
- `POST /api/wallet/transactions` - Get transaction history with pagination

## Database Schema

### PostgreSQL Tables

#### `users`
- `id` (UUID, PK): User ID
- `email` (TEXT, UNIQUE): User email from OAuth
- `full_name` (TEXT): User's full name
- `created_at` (TIMESTAMP): Account creation time

#### `wallets`
- `id` (UUID, PK): Wallet ID
- `user_id` (UUID, FK): Reference to users
- `encrypted_mnemonic` (TEXT): BIP-39 mnemonic phrase (encrypted)
- `birthday_height` (INTEGER): Blockchain birthday height
- `address` (TEXT): Unified address
- `created_at` (TIMESTAMP): Wallet creation time

#### `transactions`
- `id` (UUID, PK): Transaction ID
- `user_id` (UUID, FK): Reference to users
- `txid` (TEXT): Zcash transaction ID
- `block_height` (INTEGER): Block height (NULL if unconfirmed)
- `timestamp` (TIMESTAMP): Transaction timestamp
- `created_at` (TIMESTAMP): Record creation time

#### `sent_notes`
- Transaction outputs sent by user
- Tracks amount, memo, recipient address

#### `received_notes`
- Transaction outputs received by user
- Tracks amount, memo, sender (if available)

### SQLite (Per-User Wallet Database)
Each user has a separate SQLite database managed by `zcash_client_sqlite`:
- Shielded notes (Sapling and Orchard)
- Nullifiers
- Block scanning state
- Transaction data

## Development

### Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend
npm test
```

### Building for Production

```bash
# Backend
cd backend
cargo build --release

# Frontend
cd frontend
npm run build
npm start
```

## Security Considerations

### Implemented
- ✅ JWT-based authentication with expiration
- ✅ Google OAuth 2.0 integration
- ✅ Mnemonic phrases stored in PostgreSQL (should be encrypted in production)
- ✅ Shielded transactions for privacy
- ✅ ZIP-317 fee calculation to prevent fee overpayment
- ✅ Balance validation before transaction broadcast

### TODO for Production
- 🔒 Encrypt mnemonic phrases at rest
- 🔒 Implement key derivation for mnemonic encryption
- 🔒 Add rate limiting for API endpoints
- 🔒 Enable HTTPS/TLS for all connections
- 🔒 Implement CORS restrictions
- 🔒 Add request signing for sensitive operations
- 🔒 Implement backup/recovery mechanisms

## Architecture

### Wallet Flow
1. **User Login**: Google OAuth authentication
2. **Wallet Creation**: Automatic creation on first access
   - Generate BIP-39 mnemonic
   - Derive unified spending key
   - Calculate unified address
   - Record birthday height
3. **Balance Check**:
   - Connect to lightwalletd
   - Scan blockchain from birthday height
   - Calculate spendable balance
   - Sync to PostgreSQL
4. **Send Transaction**:
   - Estimate fee using Zcash proposal
   - Validate balance (amount + fee)
   - Build transaction with zk-SNARKs
   - Broadcast to network

### Transaction Fee Calculation
Shield implements real-time fee calculation using ZIP-317:
- **Proposal Creation**: Creates a Zcash transaction proposal
- **Fee Extraction**: Extracts calculated fee from proposal
- **Pre-validation**: Validates balance before building full transaction
- **Accurate Display**: Shows exact fee that will be charged

## Roadmap

### Phase 1 (Completed)
- ✅ User authentication with Google OAuth
- ✅ Automatic wallet creation
- ✅ Send shielded transactions
- ✅ Transaction history
- ✅ Real-time fee calculation
- ✅ Balance tracking
- ✅ Transaction confirmation flow

### Phase 2 (Planned)
- [ ] QR code generation for addresses
- [ ] QR code scanning for sending
- [ ] USD price integration
- [ ] Transaction status updates
- [ ] Wallet recovery from mnemonic
- [ ] Multi-account support

### Phase 3 (Future)
- [ ] Mobile app (React Native)
- [ ] Hardware wallet integration
- [ ] DeFi integrations
- [ ] Cross-chain swaps

## Contributing

This is a private project. For questions or contributions, please contact the maintainer.

## License

Proprietary - All Rights Reserved

## Acknowledgments

- **Electric Coin Company**: For the Zcash protocol and libraries
- **Zcash Foundation**: For lightwalletd infrastructure
- **Zcash Community**: For ongoing support and development

---

**Note**: This wallet is designed for mainnet use but should undergo security audit before handling significant funds. Always backup your mnemonic phrase securely.
