'use client';

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Send, Loader2, CheckCircle, AlertCircle, ExternalLink, ArrowLeft, AlertTriangle } from 'lucide-react';
import { walletAPI, SendTransactionResponse } from '@/lib/api/wallet';
import { Button } from '@/components/ui/Button';

interface SendTransactionModalProps {
  isOpen: boolean;
  onClose: () => void;
  userId: string;
  currentBalance: string;
  onSuccess?: () => void;
  initialAddress?: string;
  initialAmount?: string;
  initialMemo?: string;
}

export function SendTransactionModal({
  isOpen,
  onClose,
  userId,
  currentBalance,
  onSuccess,
  initialAddress = '',
  initialAmount = '',
  initialMemo = '',
}: SendTransactionModalProps) {
  const [toAddress, setToAddress] = useState(initialAddress);
  const [amount, setAmount] = useState(initialAmount);
  const [memo, setMemo] = useState(initialMemo);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<SendTransactionResponse | null>(null);
  const [showConfirmation, setShowConfirmation] = useState(false);
  const [calculatedFee, setCalculatedFee] = useState<number | null>(null);
  const [feeLoading, setFeeLoading] = useState(false);

  // Sync state when initial values change (e.g., from QR scan)
  useEffect(() => {
    if (initialAddress) setToAddress(initialAddress);
    if (initialAmount) setAmount(initialAmount);
    if (initialMemo) setMemo(initialMemo);
  }, [initialAddress, initialAmount, initialMemo]);

  const handleClose = () => {
    if (!isLoading) {
      setToAddress('');
      setAmount('');
      setMemo('');
      setError(null);
      setSuccess(null);
      setShowConfirmation(false);
      setCalculatedFee(null);
      onClose();
    }
  };

  const handleContinue = async () => {
    if (!toAddress || !amount) {
      setError('Please fill in all required fields');
      return;
    }

    const amountNum = parseFloat(amount);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Please enter a valid amount');
      return;
    }

    // Fetch actual fee from backend
    setFeeLoading(true);
    setError(null);

    try {
      const feeResponse = await walletAPI.estimateFee({
        user_id: userId,
        to_address: toAddress,
        amount_zec: amountNum,
        memo: memo || undefined,
      });

      setCalculatedFee(feeResponse.estimated_fee_zec);

      // Validate balance with calculated fee
      const balance = parseFloat(currentBalance);
      const totalRequired = feeResponse.total_zec;

      if (totalRequired > balance) {
        setError(`Insufficient balance. You need ${totalRequired.toFixed(8)} ZEC (amount + fee), but only have ${balance.toFixed(8)} ZEC`);
        setFeeLoading(false);
        return;
      }

      setShowConfirmation(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to calculate fee');
    } finally {
      setFeeLoading(false);
    }
  };

  const handleBack = () => {
    setShowConfirmation(false);
    setError(null);
  };

  const handleConfirmSend = async () => {
    const amountNum = parseFloat(amount);

    setIsLoading(true);
    setError(null);

    try {
      const response = await walletAPI.sendTransaction({
        user_id: userId,
        to_address: toAddress,
        amount_zec: amountNum,
        memo: memo || undefined,
      });

      setSuccess(response);
      setShowConfirmation(false);
      onSuccess?.();

      // Auto-close after 5 seconds
      setTimeout(() => {
        handleClose();
      }, 5000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send transaction');
      setShowConfirmation(false);
    } finally {
      setIsLoading(false);
    }
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
            onClick={handleClose}
            className="fixed inset-0 bg-black/80 backdrop-blur-md z-50"
          />

          {/* Modal */}
          <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            <motion.div
              initial={{ opacity: 0, scale: 0.95, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.95, y: 20 }}
              transition={{ duration: 0.2 }}
              className="bg-bg-primary border border-brand-primary/20 rounded-2xl shadow-2xl max-w-lg w-full overflow-hidden"
            >
              {/* Header */}
              <div className="flex items-center justify-between p-6 border-b border-brand-primary/10">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-full bg-brand-primary/10 flex items-center justify-center">
                    <Send className="w-5 h-5 text-brand-primary" />
                  </div>
                  <h2 className="text-2xl font-bold">Send ZEC</h2>
                </div>
                <button
                  onClick={handleClose}
                  disabled={isLoading}
                  className="p-2 hover:bg-brand-primary/10 rounded-lg transition-colors disabled:opacity-50"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>

              {/* Content */}
              <div className="p-6">
                {success ? (
                  /* Success State */
                  <div className="text-center py-4">
                    <div className="w-16 h-16 rounded-full bg-green-500/10 flex items-center justify-center mx-auto mb-4">
                      <CheckCircle className="w-8 h-8 text-green-500" />
                    </div>
                    <h3 className="text-xl font-bold mb-2">Transaction Sent!</h3>
                    <p className="text-text-secondary mb-6">
                      Your transaction has been broadcast to the network
                    </p>

                    <div className="bg-bg-secondary rounded-lg p-4 mb-4 text-left">
                      <div className="space-y-3 text-sm">
                        <div>
                          <div className="text-text-tertiary mb-1">Amount</div>
                          <div className="font-mono font-bold text-brand-primary">
                            {success.amount_zec} ZEC
                          </div>
                        </div>
                        <div>
                          <div className="text-text-tertiary mb-1">Network Fee</div>
                          <div className="font-mono text-sm">
                            {success.fee_zec.toFixed(8)} ZEC
                          </div>
                        </div>
                        <div>
                          <div className="text-text-tertiary mb-1">To</div>
                          <div className="font-mono text-xs break-all">
                            {success.to_address}
                          </div>
                        </div>
                        <div>
                          <div className="text-text-tertiary mb-1">Transaction ID</div>
                          <div className="font-mono text-xs break-all text-brand-secondary">
                            {success.txid}
                          </div>
                        </div>
                      </div>
                    </div>

                    <a
                      href={success.explorer_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-2 text-brand-primary hover:text-brand-secondary transition-colors"
                    >
                      View on Explorer
                      <ExternalLink className="w-4 h-4" />
                    </a>

                    <div className="mt-6">
                      <Button onClick={handleClose} variant="primary" className="w-full">
                        Close
                      </Button>
                    </div>
                  </div>
                ) : showConfirmation ? (
                  /* Confirmation State */
                  <>
                    <div className="mb-6">
                      <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4 flex items-start gap-3 mb-6">
                        <AlertTriangle className="w-5 h-5 text-yellow-500 flex-shrink-0 mt-0.5" />
                        <div className="text-sm text-yellow-500">
                          Please review the transaction details carefully before confirming. This action cannot be undone.
                        </div>
                      </div>

                      <h3 className="text-lg font-bold mb-4">Confirm Transaction</h3>

                      <div className="space-y-3">
                        {/* Amount */}
                        <div className="bg-bg-secondary rounded-lg p-4">
                          <div className="text-sm text-text-tertiary mb-1">Amount</div>
                          <div className="text-2xl font-bold text-brand-primary">
                            {amount} ZEC
                          </div>
                        </div>

                        {/* Recipient */}
                        <div className="bg-bg-secondary rounded-lg p-4">
                          <div className="text-sm text-text-tertiary mb-1">To</div>
                          <div className="font-mono text-sm break-all">{toAddress}</div>
                        </div>

                        {/* Memo */}
                        {memo && (
                          <div className="bg-bg-secondary rounded-lg p-4">
                            <div className="text-sm text-text-tertiary mb-1">Memo</div>
                            <div className="text-sm">{memo}</div>
                          </div>
                        )}

                        {/* Fee */}
                        <div className="bg-bg-secondary rounded-lg p-4">
                          <div className="text-sm text-text-tertiary mb-1">Network Fee</div>
                          <div className="font-mono text-sm">{calculatedFee?.toFixed(8) || '0.00000000'} ZEC</div>
                        </div>

                        {/* Total */}
                        <div className="bg-gradient-to-br from-brand-primary/10 to-brand-secondary/10 border border-brand-primary/20 rounded-lg p-4">
                          <div className="text-sm text-text-tertiary mb-1">Total (Amount + Fee)</div>
                          <div className="text-xl font-bold text-brand-primary">
                            {(parseFloat(amount) + (calculatedFee || 0)).toFixed(8)} ZEC
                          </div>
                        </div>
                      </div>
                    </div>

                    {/* Error Message */}
                    {error && (
                      <motion.div
                        initial={{ opacity: 0, y: -10 }}
                        animate={{ opacity: 1, y: 0 }}
                        className="bg-red-500/10 border border-red-500/20 rounded-lg p-4 flex items-start gap-3 mb-4"
                      >
                        <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
                        <div className="text-sm text-red-500">{error}</div>
                      </motion.div>
                    )}

                    {/* Actions */}
                    <div className="flex gap-3">
                      <Button
                        onClick={handleBack}
                        variant="outline"
                        disabled={isLoading}
                        className="flex-1 gap-2"
                      >
                        <ArrowLeft className="w-4 h-4" />
                        Back
                      </Button>
                      <Button
                        onClick={handleConfirmSend}
                        variant="primary"
                        disabled={isLoading}
                        className="flex-1 gap-2"
                      >
                        {isLoading ? (
                          <>
                            <Loader2 className="w-4 h-4 animate-spin" />
                            Sending...
                          </>
                        ) : (
                          <>
                            <Send className="w-4 h-4" />
                            Confirm & Send
                          </>
                        )}
                      </Button>
                    </div>
                  </>
                ) : (
                  /* Form State */
                  <>
                    <div className="space-y-4">
                      {/* Balance Display */}
                      <div className="bg-gradient-to-br from-brand-primary/10 to-brand-secondary/10 border border-brand-primary/20 rounded-lg p-4">
                        <div className="text-sm text-text-tertiary mb-1">Available Balance</div>
                        <div className="text-2xl font-bold text-brand-primary">
                          {currentBalance} ZEC
                        </div>
                      </div>

                      {/* To Address */}
                      <div>
                        <label className="block text-sm font-medium mb-2">
                          Recipient Address <span className="text-red-500">*</span>
                        </label>
                        <input
                          type="text"
                          value={toAddress}
                          onChange={(e) => setToAddress(e.target.value)}
                          placeholder="u1..."
                          disabled={isLoading}
                          className="w-full px-4 py-3 bg-bg-secondary border border-brand-primary/10 rounded-lg focus:outline-none focus:border-brand-primary/50 font-mono text-sm disabled:opacity-50"
                        />
                      </div>

                      {/* Amount */}
                      <div>
                        <label className="block text-sm font-medium mb-2">
                          Amount (ZEC) <span className="text-red-500">*</span>
                        </label>
                        <input
                          type="number"
                          step="0.00000001"
                          min="0"
                          value={amount}
                          onChange={(e) => setAmount(e.target.value)}
                          placeholder="0.0001"
                          disabled={isLoading}
                          className="w-full px-4 py-3 bg-bg-secondary border border-brand-primary/10 rounded-lg focus:outline-none focus:border-brand-primary/50 font-mono disabled:opacity-50"
                        />
                        <p className="text-xs text-text-tertiary mt-1">
                          Network fee: 0.0001 ZEC (ZIP-317 standard)
                        </p>
                      </div>

                      {/* Memo */}
                      <div>
                        <label className="block text-sm font-medium mb-2">
                          Memo (Optional)
                        </label>
                        <textarea
                          value={memo}
                          onChange={(e) => setMemo(e.target.value)}
                          placeholder="Add a note..."
                          rows={3}
                          disabled={isLoading}
                          className="w-full px-4 py-3 bg-bg-secondary border border-brand-primary/10 rounded-lg focus:outline-none focus:border-brand-primary/50 resize-none disabled:opacity-50"
                        />
                      </div>

                      {/* Error Message */}
                      {error && (
                        <motion.div
                          initial={{ opacity: 0, y: -10 }}
                          animate={{ opacity: 1, y: 0 }}
                          className="bg-red-500/10 border border-red-500/20 rounded-lg p-4 flex items-start gap-3"
                        >
                          <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
                          <div className="text-sm text-red-500">{error}</div>
                        </motion.div>
                      )}
                    </div>

                    {/* Actions */}
                    <div className="flex gap-3 mt-6">
                      <Button
                        onClick={handleClose}
                        variant="outline"
                        disabled={isLoading}
                        className="flex-1"
                      >
                        Cancel
                      </Button>
                      <Button
                        onClick={handleContinue}
                        variant="primary"
                        disabled={isLoading || feeLoading || !toAddress || !amount}
                        className="flex-1 gap-2"
                      >
                        {feeLoading ? (
                          <>
                            <Loader2 className="w-4 h-4 animate-spin" />
                            Calculating...
                          </>
                        ) : (
                          'Continue'
                        )}
                      </Button>
                    </div>
                  </>
                )}
              </div>
            </motion.div>
          </div>
        </>
      )}
    </AnimatePresence>
  );
}
