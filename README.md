# Shield - Private Crypto Payments

**Pay anywhere. Dox nothing.**

Shield is a privacy-first payment system that enables crypto payments without exposing customer wallet addresses. Built on Zcash shielded transactions.

## The Problem

Every crypto payment reveals your wallet address. Once exposed:
- Your entire transaction history is visible
- Your balance is public
- Anyone can trace your financial life

## The Solution

Shield uses **Zcash shielded transactions** with zero-knowledge proofs. When you pay:
- Merchant receives payment confirmation
- Merchant **never sees** your wallet address
- No transaction trail linking you to the purchase

## What's Built

### Shield Wallet (`frontend/`)
Customer-facing wallet application:
- Email/password and Google OAuth authentication
- Zcash shielded wallet with unified addresses
- QR code scanning for payments
- Real-time balance with blockchain sync
- Transaction history with memos

### Shield POS (`pos/`)
Merchant point-of-sale system:
- Order creation with item management
- QR code generation for payments
- Real-time payment detection via memo matching
- Merchant wallet balance display
- Transaction history for reconciliation

### Backend (`backend/`)
Rust backend with Zcash integration:
- User authentication (email/password + Google OAuth)
- Wallet management via `zcash_client_backend`
- Shielded transactions with ZIP-317 fees
- Real-time blockchain scanning via lightwalletd
- PostgreSQL for user data, SQLite for wallet state

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust, Actix-web, SQLx, zcash_client_backend |
| Frontend | Next.js 14, TypeScript, Tailwind CSS |
| POS | Next.js 14, TypeScript, Tailwind CSS |
| Database | PostgreSQL (users), SQLite (wallet state) |
| Privacy | Zcash shielded transactions (Orchard + Sapling) |

## Project Structure

```
Shield-Infra/
├── backend/           # Rust backend
│   ├── src/
│   │   ├── handlers/  # API endpoints
│   │   ├── zcash/     # Wallet implementation
│   │   └── solana/    # Solana wallet (for bridging)
│   └── migrations/    # Database migrations
├── frontend/          # Shield Wallet (customer app)
│   ├── app/           # Next.js pages
│   ├── components/    # React components
│   └── lib/           # API clients
└── pos/               # Shield POS (merchant app)
    ├── app/           # Next.js pages
    ├── components/    # React components
    └── lib/           # API clients
```

## Running Locally

### Prerequisites
- Rust (latest stable)
- Node.js 20+
- PostgreSQL
- Docker (optional)

### 1. Start Database
```bash
docker-compose up -d
```

### 2. Start Backend
```bash
cd backend
cp .env.example .env  # Configure your environment
cargo build --release
source .env && ./target/release/shield-backend
```
Backend runs on `http://localhost:8000`

### 3. Start Shield Wallet
```bash
cd frontend
npm install
npm run dev
```
Wallet runs on `http://localhost:3000`

### 4. Start Shield POS
```bash
cd pos
npm install
npm run dev
```
POS runs on `http://localhost:3001`

## Payment Flow

1. **Merchant** creates order in POS → QR code generated
2. **Customer** scans QR with Shield Wallet
3. **Customer** confirms payment (amount + memo with order ID)
4. **Shield** sends shielded transaction to merchant
5. **POS** detects payment via memo matching → shows confirmation
6. **Merchant** sees funds received, **never sees customer's wallet**

## Privacy Guarantee

| What | Visible to Merchant? |
|------|---------------------|
| Payment received | Yes |
| Order details (memo) | Yes |
| Customer wallet address | **No** |
| Customer balance | **No** |
| Customer transaction history | **No** |

## License

MIT
