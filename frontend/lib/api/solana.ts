const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000/api';

export interface SolanaBalanceResponse {
  balance_lamports: number;
  balance_sol: number;
  address: string;
}

export interface BridgeQuoteRequest {
  amount_lamports: number;
  recipient_zcash_address: string;
}

export interface BridgeQuoteResponse {
  amount_in: string;
  amount_in_formatted: string;
  amount_out: string;
  amount_out_formatted: string;
  deposit_address: string;
  time_estimate: number;
}

export interface ExecuteBridgeRequest {
  amount_lamports: number;
  recipient_zcash_address: string;
}

export interface ExecuteBridgeResponse {
  bridge_tx_id: string;
  solana_signature: string;
  deposit_address: string;
  expected_zec: string;
}

export interface BridgeStatusRequest {
  deposit_address: string;
}

export interface BridgeStatusResponse {
  status: string;
  updatedAt?: string;
  swapDetails?: {
    depositedAmount: string;
    depositedAmountFormatted: string;
    amountOut: string;
    amountOutFormatted: string;
    destinationChainTxHashes?: Array<{
      hash: string;
      explorerUrl: string;
    }>;
  };
}

class SolanaAPI {
  private async fetch<T>(
    url: string,
    accessToken: string,
    options?: RequestInit
  ): Promise<T> {
    const response = await fetch(`${API_URL}${url}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${accessToken}`,
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const error = await response
        .json()
        .catch(() => ({ error: 'An error occurred' }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  async getBalance(
    userId: string,
    accessToken: string
  ): Promise<SolanaBalanceResponse> {
    return this.fetch<SolanaBalanceResponse>('/solana/balance', accessToken, {
      method: 'POST',
      body: JSON.stringify({ user_id: userId }),
    });
  }

  async getBridgeQuote(
    request: BridgeQuoteRequest,
    accessToken: string
  ): Promise<BridgeQuoteResponse> {
    return this.fetch<BridgeQuoteResponse>(
      '/solana/bridge/quote',
      accessToken,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  }

  async executeBridge(
    request: ExecuteBridgeRequest,
    accessToken: string
  ): Promise<ExecuteBridgeResponse> {
    return this.fetch<ExecuteBridgeResponse>(
      '/solana/bridge/execute',
      accessToken,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  }

  async getBridgeStatus(
    request: BridgeStatusRequest,
    accessToken: string
  ): Promise<BridgeStatusResponse> {
    return this.fetch<BridgeStatusResponse>(
      '/solana/bridge/status',
      accessToken,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  }
}

export const solanaAPI = new SolanaAPI();
