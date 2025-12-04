'use client';

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, ArrowRightLeft, Loader2, CheckCircle, AlertCircle, RefreshCw, ExternalLink } from 'lucide-react';
import { solanaAPI, BridgeQuoteResponse, ExecuteBridgeResponse, BridgeStatusResponse } from '@/lib/api/solana';
import { Button } from '@/components/ui/Button';

interface BridgeModalProps {
  isOpen: boolean;
  onClose: () => void;
  userId: string;
  currentBalance: number;
  zcashAddress: string;
  solanaAddress: string;
  onSuccess?: () => void;
}

type BridgeStep = 'input' | 'quote' | 'executing' | 'monitoring' | 'success' | 'error';

export function BridgeModal({
  isOpen,
  onClose,
  userId,
  currentBalance,
  zcashAddress,
  solanaAddress,
  onSuccess,
}: BridgeModalProps) {
  const [amount, setAmount] = useState('');
  const [step, setStep] = useState<BridgeStep>('input');
  const [error, setError] = useState<string | null>(null);
  const [quote, setQuote] = useState<BridgeQuoteResponse | null>(null);
  const [bridgeResult, setBridgeResult] = useState<ExecuteBridgeResponse | null>(null);
  const [bridgeStatus, setBridgeStatus] = useState<BridgeStatusResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleClose = () => {
    if (!isLoading && step !== 'executing' && step !== 'monitoring') {
      setAmount('');
      setStep('input');
      setError(null);
      setQuote(null);
      setBridgeResult(null);
      setBridgeStatus(null);
      onClose();
    }
  };

  const handleGetQuote = async () => {
    if (!amount) {
      setError('Please enter an amount');
      return;
    }

    const amountNum = parseFloat(amount);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Please enter a valid amount');
      return;
    }

    if (amountNum > currentBalance) {
      setError(`Insufficient balance. You have ${currentBalance.toFixed(4)} SOL`);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const accessToken = localStorage.getItem('shield_access_token') || '';
      const amountLamports = Math.floor(amountNum * 1_000_000_000);

      const quoteResponse = await solanaAPI.getBridgeQuote({
        amount_lamports: amountLamports,
        recipient_zcash_address: zcashAddress,
      }, accessToken);

      setQuote(quoteResponse);
      setStep('quote');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to get quote');
    } finally {
      setIsLoading(false);
    }
  };

  const handleExecuteBridge = async () => {
    if (!quote || !amount) return;

    setIsLoading(true);
    setError(null);
    setStep('executing');

    try {
      const accessToken = localStorage.getItem('shield_access_token') || '';
      const amountLamports = Math.floor(parseFloat(amount) * 1_000_000_000);

      const executeResponse = await solanaAPI.executeBridge({
        amount_lamports: amountLamports,
        recipient_zcash_address: zcashAddress,
      }, accessToken);

      setBridgeResult(executeResponse);
      setStep('monitoring');

      // Start monitoring the bridge status
      startMonitoring(executeResponse.deposit_address, accessToken);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to execute bridge');
      setStep('error');
    } finally {
      setIsLoading(false);
    }
  };

  const startMonitoring = async (depositAddress: string, accessToken: string) => {
    let attempts = 0;
    const maxAttempts = 60; // Monitor for up to 5 minutes (5 second intervals)

    const checkStatus = async () => {
      try {
        const statusResponse = await solanaAPI.getBridgeStatus({
          deposit_address: depositAddress,
        }, accessToken);

        setBridgeStatus(statusResponse);

        if (statusResponse.status === 'COMPLETED' || statusResponse.status === 'SUCCESS') {
          setStep('success');
          onSuccess?.();
          return true;
        } else if (statusResponse.status === 'FAILED' || statusResponse.status === 'ERROR') {
          setError('Bridge transaction failed');
          setStep('error');
          return true;
        }

        return false;
      } catch (err) {
        console.error('Failed to check bridge status:', err);
        return false;
      }
    };

    const poll = async () => {
      attempts++;
      const isDone = await checkStatus();

      if (!isDone && attempts < maxAttempts) {
        setTimeout(poll, 5000); // Check every 5 seconds
      } else if (attempts >= maxAttempts) {
        setError('Bridge monitoring timeout. Please check status manually.');
        setStep('error');
      }
    };

    poll();
  };

  const handleBack = () => {
    setStep('input');
    setError(null);
    setQuote(null);
  };

  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          className="relative w-full max-w-md bg-bg-primary rounded-2xl shadow-xl border border-border-primary overflow-hidden"
        >
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-border-primary">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-purple-500/10 flex items-center justify-center">
                <ArrowRightLeft className="w-5 h-5 text-purple-500" />
              </div>
              <h2 className="text-xl font-bold">Bridge SOL to Zcash</h2>
            </div>
            <button
              onClick={handleClose}
              disabled={isLoading || step === 'executing' || step === 'monitoring'}
              className="text-text-tertiary hover:text-text-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Content */}
          <div className="p-6">
            {/* Input Step */}
            {step === 'input' && (
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-text-secondary mb-2">
                    Amount (SOL)
                  </label>
                  <input
                    type="number"
                    step="0.0001"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    placeholder="0.0000"
                    className="w-full px-4 py-3 bg-bg-secondary border border-border-primary rounded-lg focus:outline-none focus:border-purple-500 transition-colors"
                  />
                  <div className="mt-2 text-sm text-text-tertiary">
                    Available: {currentBalance.toFixed(4)} SOL
                  </div>
                </div>

                <div className="p-4 bg-purple-500/5 rounded-lg border border-purple-500/20">
                  <div className="text-sm text-text-secondary mb-1">Recipient (Zcash)</div>
                  <div className="font-mono text-xs break-all text-text-primary">
                    {zcashAddress}
                  </div>
                </div>

                {error && (
                  <div className="flex items-start gap-2 p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
                    <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
                    <p className="text-sm text-red-500">{error}</p>
                  </div>
                )}

                <Button
                  onClick={handleGetQuote}
                  disabled={isLoading || !amount}
                  className="w-full"
                  variant="primary"
                >
                  {isLoading ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      Getting Quote...
                    </>
                  ) : (
                    'Get Quote'
                  )}
                </Button>
              </div>
            )}

            {/* Quote Step */}
            {step === 'quote' && quote && (
              <div className="space-y-4">
                <div className="space-y-3">
                  <div className="flex justify-between items-center p-4 bg-bg-secondary rounded-lg">
                    <span className="text-sm text-text-tertiary">You Send</span>
                    <span className="font-bold text-purple-500">{quote.amount_in_formatted} SOL</span>
                  </div>

                  <div className="flex justify-center">
                    <ArrowRightLeft className="w-5 h-5 text-text-tertiary" />
                  </div>

                  <div className="flex justify-between items-center p-4 bg-brand-primary/5 rounded-lg border border-brand-primary/20">
                    <span className="text-sm text-text-tertiary">You Receive (est.)</span>
                    <span className="font-bold text-brand-primary">{quote.amount_out_formatted} ZEC</span>
                  </div>

                  <div className="p-4 bg-bg-secondary rounded-lg space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-text-tertiary">Estimated Time</span>
                      <span className="text-text-primary">{quote.time_estimate} minutes</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-text-tertiary">Deposit Address</span>
                      <span className="font-mono text-xs text-text-primary truncate max-w-[150px]">
                        {quote.deposit_address}
                      </span>
                    </div>
                  </div>
                </div>

                {error && (
                  <div className="flex items-start gap-2 p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
                    <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
                    <p className="text-sm text-red-500">{error}</p>
                  </div>
                )}

                <div className="flex gap-3">
                  <Button
                    onClick={handleBack}
                    disabled={isLoading}
                    className="flex-1"
                    variant="outline"
                  >
                    Back
                  </Button>
                  <Button
                    onClick={handleExecuteBridge}
                    disabled={isLoading}
                    className="flex-1"
                    variant="primary"
                  >
                    {isLoading ? (
                      <>
                        <Loader2 className="w-4 h-4 animate-spin" />
                        Executing...
                      </>
                    ) : (
                      'Execute Bridge'
                    )}
                  </Button>
                </div>
              </div>
            )}

            {/* Monitoring Step */}
            {step === 'monitoring' && bridgeResult && (
              <div className="space-y-4">
                <div className="text-center py-6">
                  <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-purple-500/10 flex items-center justify-center">
                    <Loader2 className="w-8 h-8 text-purple-500 animate-spin" />
                  </div>
                  <h3 className="text-lg font-bold mb-2">Bridge in Progress</h3>
                  <p className="text-sm text-text-secondary">
                    Your SOL is being bridged to Zcash. This may take several minutes.
                  </p>
                </div>

                <div className="space-y-3 p-4 bg-bg-secondary rounded-lg text-sm">
                  <div className="flex justify-between">
                    <span className="text-text-tertiary">Transaction ID</span>
                    <span className="font-mono text-xs text-text-primary truncate max-w-[150px]">
                      {bridgeResult.bridge_tx_id}
                    </span>
                  </div>
                  {bridgeResult.solana_signature && (
                    <div className="flex justify-between items-center">
                      <span className="text-text-tertiary">Solana TX</span>
                      <a
                        href={`https://solscan.io/tx/${bridgeResult.solana_signature}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-1 text-purple-500 hover:text-purple-400"
                      >
                        View <ExternalLink className="w-3 h-3" />
                      </a>
                    </div>
                  )}
                  {bridgeStatus && (
                    <div className="flex justify-between">
                      <span className="text-text-tertiary">Status</span>
                      <span className="text-text-primary capitalize">{bridgeStatus.status}</span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Success Step */}
            {step === 'success' && bridgeResult && (
              <div className="space-y-4">
                <div className="text-center py-6">
                  <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-green-500/10 flex items-center justify-center">
                    <CheckCircle className="w-8 h-8 text-green-500" />
                  </div>
                  <h3 className="text-lg font-bold mb-2">Bridge Completed!</h3>
                  <p className="text-sm text-text-secondary">
                    Your SOL has been successfully bridged to Zcash
                  </p>
                </div>

                {bridgeStatus?.swapDetails && (
                  <div className="space-y-3 p-4 bg-bg-secondary rounded-lg text-sm">
                    <div className="flex justify-between">
                      <span className="text-text-tertiary">Deposited</span>
                      <span className="text-text-primary">{bridgeStatus.swapDetails.depositedAmountFormatted}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-text-tertiary">Received</span>
                      <span className="text-brand-primary font-bold">{bridgeStatus.swapDetails.amountOutFormatted}</span>
                    </div>
                    {bridgeStatus.swapDetails.destinationChainTxHashes?.map((tx, idx) => (
                      <div key={idx} className="flex justify-between items-center">
                        <span className="text-text-tertiary">Zcash TX</span>
                        <a
                          href={tx.explorerUrl}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="flex items-center gap-1 text-brand-primary hover:text-brand-secondary"
                        >
                          View <ExternalLink className="w-3 h-3" />
                        </a>
                      </div>
                    ))}
                  </div>
                )}

                <Button
                  onClick={handleClose}
                  className="w-full"
                  variant="primary"
                >
                  Close
                </Button>
              </div>
            )}

            {/* Error Step */}
            {step === 'error' && (
              <div className="space-y-4">
                <div className="text-center py-6">
                  <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-red-500/10 flex items-center justify-center">
                    <AlertCircle className="w-8 h-8 text-red-500" />
                  </div>
                  <h3 className="text-lg font-bold mb-2">Bridge Failed</h3>
                  {error && (
                    <p className="text-sm text-red-500">{error}</p>
                  )}
                </div>

                <Button
                  onClick={handleClose}
                  className="w-full"
                  variant="outline"
                >
                  Close
                </Button>
              </div>
            )}
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
