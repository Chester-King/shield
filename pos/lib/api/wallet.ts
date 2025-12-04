// POS Wallet API client for communicating with Shield backend
const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000/api';

export interface BalanceResponse {
  balance_zec: string;
  synced: boolean;
  last_synced_height: number | null;
  blocks_scanned?: number | null;
  notes_found?: number | null;
  chain_tip?: number | null;
}

export interface Transaction {
  txid: string;
  timestamp: string | null;
  block_height: number | null;
  amount_zec: string;
  direction: 'sent' | 'received';
  memo: string | null;
  fee_zec: string | null;
}

export interface TransactionsResponse {
  transactions: Transaction[];
  total_count: number;
  page: number;
  page_size: number;
  has_more: boolean;
}

class POSWalletAPI {
  private async fetch<T>(url: string, options?: RequestInit): Promise<T> {
    const response = await fetch(`${API_URL}${url}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      cache: 'no-store',
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'An error occurred' }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  async getBalance(userId: string): Promise<BalanceResponse> {
    return this.fetch<BalanceResponse>('/wallet/balance', {
      method: 'POST',
      body: JSON.stringify({ user_id: userId }),
    });
  }

  async getTransactions(userId: string, pageSize: number = 20): Promise<TransactionsResponse> {
    return this.fetch<TransactionsResponse>('/wallet/transactions', {
      method: 'POST',
      body: JSON.stringify({
        user_id: userId,
        page: 1,
        page_size: pageSize,
      }),
    });
  }
}

export const posWalletAPI = new POSWalletAPI();
