'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Card } from '@/components/ui/Card';
import { walletAPI, Transaction } from '@/lib/api/wallet';
import { ArrowUpRight, ArrowDownLeft, Clock, ExternalLink } from 'lucide-react';

interface TransactionHistoryProps {
  userId: string;
}

export function TransactionHistory({ userId }: TransactionHistoryProps) {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchTransactions = async () => {
      setLoading(true);
      setError(null);
      try {
        const data = await walletAPI.getTransactions(userId);
        setTransactions(data.transactions);
      } catch (err) {
        console.error('Failed to fetch transactions:', err);
        setError('Failed to load transaction history');
      } finally {
        setLoading(false);
      }
    };

    if (userId) {
      fetchTransactions();
    }
  }, [userId]);

  const getExplorerUrl = (txid: string) => {
    // Assuming mainnet for now
    return `https://mainnet.zcashexplorer.app/transactions/${txid}`;
  };

  const formatDate = (timestamp: string | null) => {
    if (!timestamp) return 'Pending';
    const date = new Date(timestamp);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const formatAmount = (amount: string, direction: 'sent' | 'received') => {
    const num = parseFloat(amount);
    const sign = direction === 'sent' ? '-' : '+';
    return `${sign}${num.toFixed(8)} ZEC`;
  };

  if (loading && transactions.length === 0) {
    return (
      <Card>
        <div className="text-center py-8">
          <div className="text-text-secondary">Loading transactions...</div>
        </div>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <div className="text-center py-8">
          <div className="text-red-500">{error}</div>
        </div>
      </Card>
    );
  }

  if (transactions.length === 0) {
    return (
      <Card>
        <div className="text-center py-8">
          <div className="text-text-tertiary">No transactions yet</div>
        </div>
      </Card>
    );
  }

  return (
    <Card>
      <div className="mb-6">
        <h2 className="text-2xl font-bold">Transaction History</h2>
        <p className="text-text-secondary text-sm mt-1">
          Your recent wallet activity
        </p>
      </div>

      <div className="space-y-3">
        {transactions.map((tx, index) => (
          <motion.div
            key={tx.txid}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3, delay: index * 0.05 }}
            className="flex items-center gap-4 p-4 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors"
          >
            {/* Direction Icon */}
            <div
              className={`w-10 h-10 rounded-full flex items-center justify-center ${
                tx.direction === 'sent'
                  ? 'bg-red-500/10'
                  : 'bg-green-500/10'
              }`}
            >
              {tx.direction === 'sent' ? (
                <ArrowUpRight className="w-5 h-5 text-red-500" />
              ) : (
                <ArrowDownLeft className="w-5 h-5 text-green-500" />
              )}
            </div>

            {/* Transaction Details */}
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-semibold capitalize">
                  {tx.direction}
                </span>
                {tx.block_height && (
                  <span className="text-xs text-text-tertiary">
                    Block {tx.block_height.toLocaleString()}
                  </span>
                )}
              </div>

              <div className="flex items-center gap-2 text-sm text-text-tertiary mt-1">
                <Clock className="w-3 h-3" />
                <span>{formatDate(tx.timestamp)}</span>
              </div>

              {tx.memo && (
                <div className="text-xs text-text-tertiary mt-1 truncate">
                  Memo: {tx.memo}
                </div>
              )}

              <div className="text-xs text-text-tertiary mt-1 font-mono truncate">
                {tx.txid.substring(0, 16)}...{tx.txid.substring(tx.txid.length - 16)}
              </div>
            </div>

            {/* Amount and Actions */}
            <div className="text-right">
              <div
                className={`text-lg font-bold ${
                  tx.direction === 'sent'
                    ? 'text-red-500'
                    : 'text-green-500'
                }`}
              >
                {formatAmount(tx.amount_zec, tx.direction)}
              </div>

              {tx.fee_zec && (
                <div className="text-xs text-text-tertiary mt-1">
                  Fee: {parseFloat(tx.fee_zec).toFixed(8)} ZEC
                </div>
              )}

              <a
                href={getExplorerUrl(tx.txid)}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 text-xs text-brand-primary hover:text-brand-secondary transition-colors mt-2"
              >
                <span>View</span>
                <ExternalLink className="w-3 h-3" />
              </a>
            </div>
          </motion.div>
        ))}
      </div>
    </Card>
  );
}
