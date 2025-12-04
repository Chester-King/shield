'use client';

import { useState, useEffect, useCallback } from 'react';
import { RefreshCw, Wallet } from 'lucide-react';
import { posWalletAPI, BalanceResponse } from '@/lib/api/wallet';
import { MERCHANT_CONFIG } from '@/lib/constants/merchant';

export function MerchantBalance() {
  const [balance, setBalance] = useState<BalanceResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await posWalletAPI.getBalance(MERCHANT_CONFIG.user_id);
      setBalance(data);
    } catch (err) {
      console.error('Failed to fetch balance:', err);
      setError('Unable to fetch balance');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBalance();
    // Poll every 30 seconds for balance updates
    const interval = setInterval(fetchBalance, 30000);
    return () => clearInterval(interval);
  }, [fetchBalance]);

  return (
    <div className="flex items-center gap-3 px-4 py-2 rounded-lg bg-bg-secondary border border-brand-primary/20">
      <Wallet className="w-5 h-5 text-brand-primary" />
      <div className="min-w-0 flex-1">
        <div className="text-xs text-text-tertiary">Merchant Balance</div>
        <div className="font-mono font-bold text-brand-primary truncate">
          {loading && !balance ? (
            <span className="text-text-tertiary">Loading...</span>
          ) : error ? (
            <span className="text-red-400 text-sm">{error}</span>
          ) : (
            <>{balance?.balance_zec || '0.00000000'} ZEC</>
          )}
        </div>
        {balance && !balance.synced && (
          <div className="text-xs text-yellow-500">Syncing...</div>
        )}
      </div>
      <button
        onClick={fetchBalance}
        disabled={loading}
        className="p-2 hover:bg-brand-primary/10 rounded-lg transition-colors disabled:opacity-50"
        title="Refresh balance"
      >
        <RefreshCw className={`w-4 h-4 text-brand-primary ${loading ? 'animate-spin' : ''}`} />
      </button>
    </div>
  );
}
