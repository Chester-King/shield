'use client';

import { useEffect, useState, useRef, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { walletAPI, Transaction } from '@/lib/api/wallet';
import { ArrowUpRight, ArrowDownLeft, Clock, ExternalLink, X, Loader2 } from 'lucide-react';

interface TransactionHistorySidebarProps {
  userId: string;
  isOpen: boolean;
  onClose: () => void;
}

export function TransactionHistorySidebar({ userId, isOpen, onClose }: TransactionHistorySidebarProps) {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [totalCount, setTotalCount] = useState(0);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const observerTarget = useRef<HTMLDivElement>(null);

  // Reset state when sidebar opens
  useEffect(() => {
    if (isOpen && userId) {
      setTransactions([]);
      setPage(0);
      setHasMore(false);
      setError(null);
      fetchTransactions(0);
    }
  }, [isOpen, userId]);

  // Disable body scroll when sidebar is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = 'unset';
    }
    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isOpen]);

  const fetchTransactions = async (pageNum: number) => {
    if (pageNum === 0) {
      setLoading(true);
    } else {
      setLoadingMore(true);
    }
    setError(null);

    try {
      const data = await walletAPI.getTransactions(userId, pageNum, 20);

      if (pageNum === 0) {
        setTransactions(data.transactions);
      } else {
        setTransactions(prev => [...prev, ...data.transactions]);
      }

      setHasMore(data.has_more);
      setTotalCount(data.total_count);
      setPage(pageNum);
    } catch (err) {
      console.error('Failed to fetch transactions:', err);
      setError('Failed to load transaction history');
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  };

  // Infinite scroll observer
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !loadingMore && !loading) {
          fetchTransactions(page + 1);
        }
      },
      { threshold: 0.1 }
    );

    const currentTarget = observerTarget.current;
    if (currentTarget) {
      observer.observe(currentTarget);
    }

    return () => {
      if (currentTarget) {
        observer.unobserve(currentTarget);
      }
    };
  }, [hasMore, loadingMore, loading, page]);

  const getExplorerUrl = (txid: string) => {
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

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 bg-black/80 backdrop-blur-md z-40"
            onClick={onClose}
          />

          {/* Sidebar */}
          <motion.div
            initial={{ x: '100%' }}
            animate={{ x: 0 }}
            exit={{ x: '100%' }}
            transition={{ type: 'spring', damping: 25, stiffness: 200 }}
            className="fixed right-0 top-0 h-full w-full sm:w-[480px] bg-bg-primary border-l border-brand-primary/20 shadow-2xl z-50 flex flex-col"
          >
            {/* Header */}
            <div className="flex items-center justify-between p-6 border-b border-brand-primary/20">
              <div>
                <h2 className="text-2xl font-bold">Transaction History</h2>
                <p className="text-sm text-text-secondary mt-1">
                  {totalCount > 0
                    ? `Showing ${transactions.length} of ${totalCount} transaction${totalCount !== 1 ? 's' : ''}`
                    : 'No transactions'}
                </p>
              </div>
              <button
                onClick={onClose}
                className="w-10 h-10 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors flex items-center justify-center"
              >
                <X className="w-5 h-5 text-text-secondary" />
              </button>
            </div>

            {/* Content */}
            <div ref={scrollContainerRef} className="flex-1 overflow-y-auto p-6">
              {loading && transactions.length === 0 ? (
                <div className="flex items-center justify-center h-full">
                  <div className="text-text-secondary">Loading transactions...</div>
                </div>
              ) : error ? (
                <div className="flex items-center justify-center h-full">
                  <div className="text-red-500">{error}</div>
                </div>
              ) : transactions.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-center">
                  <div className="w-16 h-16 rounded-full bg-brand-primary/10 flex items-center justify-center mb-4">
                    <Clock className="w-8 h-8 text-brand-primary" />
                  </div>
                  <div className="text-text-tertiary">No transactions yet</div>
                  <p className="text-sm text-text-tertiary mt-2">
                    Your transaction history will appear here
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  {transactions.map((tx, index) => (
                    <motion.div
                      key={tx.txid}
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ duration: 0.3, delay: index * 0.05 }}
                      className="p-4 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors border border-transparent hover:border-brand-primary/20"
                    >
                      {/* Direction and Amount */}
                      <div className="flex items-start justify-between mb-3">
                        <div className="flex items-center gap-3">
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
                          <div>
                            <div className="font-semibold capitalize">
                              {tx.direction}
                            </div>
                            <div className="flex items-center gap-2 text-xs text-text-tertiary">
                              <Clock className="w-3 h-3" />
                              <span>{formatDate(tx.timestamp)}</span>
                            </div>
                          </div>
                        </div>
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
                            <div className="text-xs text-text-tertiary">
                              Fee: {parseFloat(tx.fee_zec).toFixed(8)}
                            </div>
                          )}
                        </div>
                      </div>

                      {/* Transaction Details */}
                      <div className="space-y-2">
                        {tx.block_height && (
                          <div className="text-xs text-text-tertiary">
                            Block: {tx.block_height.toLocaleString()}
                          </div>
                        )}

                        {tx.memo && (
                          <div className="text-xs text-text-secondary p-2 rounded bg-bg-primary">
                            <span className="text-text-tertiary">Memo:</span> {tx.memo}
                          </div>
                        )}

                        <div className="flex items-center justify-between">
                          <div className="text-xs text-text-tertiary font-mono truncate flex-1 mr-2">
                            {tx.txid.substring(0, 8)}...{tx.txid.substring(tx.txid.length - 8)}
                          </div>
                          <a
                            href={getExplorerUrl(tx.txid)}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="flex items-center gap-1 text-xs text-brand-primary hover:text-brand-secondary transition-colors"
                          >
                            <span>View</span>
                            <ExternalLink className="w-3 h-3" />
                          </a>
                        </div>
                      </div>
                    </motion.div>
                  ))}

                  {/* Infinite scroll trigger */}
                  <div ref={observerTarget} className="h-10 flex items-center justify-center">
                    {loadingMore && (
                      <div className="flex items-center gap-2 text-text-tertiary">
                        <Loader2 className="w-4 h-4 animate-spin" />
                        <span className="text-sm">Loading more...</span>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
