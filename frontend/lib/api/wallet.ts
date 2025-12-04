export interface Wallet {
  id: string;
  user_id: string;
  address: string;
  birthday_height: number;
  created_at: string;
}

export interface GetAddressRequest {
  user_id: string;
}

export interface AddressResponse {
  address: string;
}

export interface GetBalanceRequest {
  user_id: string;
}

export interface BalanceResponse {
  balance_zec: string;
  synced: boolean;
  last_synced_height: number | null;
  blocks_scanned?: number | null;
  notes_found?: number | null;
  chain_tip?: number | null;
}

export interface SendTransactionRequest {
  user_id: string;
  to_address: string;
  amount_zec: number;
  memo?: string;
}

export interface SendTransactionResponse {
  txid: string;
  from_address: string;
  to_address: string;
  amount_zec: number;
  fee_zec: number;
  explorer_url: string;
  message: string;
}

export interface GetTransactionsRequest {
  user_id: string;
  page?: number;
  page_size?: number;
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

export interface EstimateFeeRequest {
  user_id: string;
  to_address: string;
  amount_zec: number;
  memo?: string;
}

export interface EstimateFeeResponse {
  estimated_fee_zec: number;
  total_zec: number;
}

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000/api';

class WalletAPI {
  private async fetch<T>(url: string, options?: RequestInit): Promise<T> {
    const response = await fetch(`${API_URL}${url}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      cache: 'no-store', // Disable caching for all API calls
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'An error occurred' }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  async getAddress(userId: string, accessToken: string): Promise<AddressResponse> {
    return this.fetch<AddressResponse>('/wallet/address', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
      body: JSON.stringify({ user_id: userId }),
    });
  }

  async getBalance(userId: string): Promise<BalanceResponse> {
    return this.fetch<BalanceResponse>('/wallet/balance', {
      method: 'POST',
      body: JSON.stringify({ user_id: userId }),
    });
  }

  async sendTransaction(request: SendTransactionRequest): Promise<SendTransactionResponse> {
    return this.fetch<SendTransactionResponse>('/wallet/send', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async getTransactions(userId: string, page?: number, pageSize?: number): Promise<TransactionsResponse> {
    return this.fetch<TransactionsResponse>('/wallet/transactions', {
      method: 'POST',
      body: JSON.stringify({
        user_id: userId,
        page,
        page_size: pageSize,
      }),
    });
  }

  async estimateFee(request: EstimateFeeRequest): Promise<EstimateFeeResponse> {
    return this.fetch<EstimateFeeResponse>('/wallet/estimate-fee', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }
}

export const walletAPI = new WalletAPI();
