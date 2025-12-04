'use client';

import { useEffect, useState, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { useAuth } from '@/contexts/AuthContext';
import { Container } from '@/components/layout/Container';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { GradientText } from '@/components/ui/GradientText';
import { Shield, User, LogOut, Wallet, Copy, Check, History, RefreshCw, ArrowRightLeft, QrCode } from 'lucide-react';
import { walletAPI, BalanceResponse } from '@/lib/api/wallet';
import { solanaAPI, SolanaBalanceResponse } from '@/lib/api/solana';
import { SendTransactionModal } from '@/components/wallet/SendTransactionModal';
import { ScanToPayModal } from '@/components/wallet/ScanToPayModal';
import { BridgeModal } from '@/components/wallet/BridgeModal';
import { TransactionHistorySidebar } from '@/components/wallet/TransactionHistorySidebar';
import { ProfileSidebar } from '@/components/profile/ProfileSidebar';

export default function DashboardPage() {
  const router = useRouter();
  const { user, loading, logout, isAuthenticated } = useAuth();
  const [copied, setCopied] = useState(false);
  const [copiedSolana, setCopiedSolana] = useState(false);
  const [balance, setBalance] = useState<BalanceResponse | null>(null);
  const [balanceLoading, setBalanceLoading] = useState(false);
  const [solanaBalance, setSolanaBalance] = useState<SolanaBalanceResponse | null>(null);
  const [solanaBalanceLoading, setSolanaBalanceLoading] = useState(false);
  const [sendModalOpen, setSendModalOpen] = useState(false);
  const [scanModalOpen, setScanModalOpen] = useState(false);
  const [historySidebarOpen, setHistorySidebarOpen] = useState(false);
  const [profileModalOpen, setProfileModalOpen] = useState(false);
  const [bridgeModalOpen, setBridgeModalOpen] = useState(false);

  // Prefill values for SendTransactionModal (from QR scan)
  const [prefillAddress, setPrefillAddress] = useState('');
  const [prefillAmount, setPrefillAmount] = useState('');
  const [prefillMemo, setPrefillMemo] = useState('');

  useEffect(() => {
    if (!loading && !isAuthenticated) {
      router.push('/login');
    }
  }, [loading, isAuthenticated, router]);

  const fetchBalance = useCallback(async () => {
    if (user?.id) {
      setBalanceLoading(true);
      try {
        const balanceData = await walletAPI.getBalance(user.id);
        setBalance(balanceData);
      } catch (error) {
        console.error('Failed to fetch balance:', error);
      } finally {
        setBalanceLoading(false);
      }
    }
  }, [user?.id]);

  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  // Refresh balance when page regains focus
  useEffect(() => {
    const handleFocus = () => {
      fetchBalance();
    };

    window.addEventListener('focus', handleFocus);
    return () => window.removeEventListener('focus', handleFocus);
  }, [fetchBalance]);

  const handleLogout = async () => {
    await logout();
    router.push('/');
  };

  const copyAddress = async () => {
    if (user?.wallet_address) {
      await navigator.clipboard.writeText(user.wallet_address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };


  const copySolanaAddress = async () => {
    if (user?.solana_address) {
      await navigator.clipboard.writeText(user.solana_address);
      setCopiedSolana(true);
      setTimeout(() => setCopiedSolana(false), 2000);
    }
  };

  const fetchSolanaBalance = useCallback(async () => {
    if (user?.id) {
      setSolanaBalanceLoading(true);
      try {
        const accessToken = localStorage.getItem('shield_access_token') || '';
        const balanceData = await solanaAPI.getBalance(user.id, accessToken);
        setSolanaBalance(balanceData);
      } catch (error) {
        console.error('Failed to fetch Solana balance:', error);
      } finally {
        setSolanaBalanceLoading(false);
      }
    }
  }, [user?.id]);

  useEffect(() => {
    fetchSolanaBalance();
  }, [fetchSolanaBalance]);

  const handleSendSuccess = () => {
    // Refresh balance after successful send
    if (user?.id) {
      setBalanceLoading(true);
      walletAPI.getBalance(user.id)
        .then(setBalance)
        .catch(console.error)
        .finally(() => setBalanceLoading(false));
    }
    // Clear prefill values after successful send
    setPrefillAddress('');
    setPrefillAmount('');
    setPrefillMemo('');
  };

  const handlePaymentScanned = (toAddress: string, amount: string, memo: string) => {
    // Set prefill values and open send modal
    setPrefillAddress(toAddress);
    setPrefillAmount(amount);
    setPrefillMemo(memo);
    setSendModalOpen(true);
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-brand-primary">Loading...</div>
      </div>
    );
  }

  if (!user || !isAuthenticated) {
    // Don't call router.push during render - the useEffect above handles this
    return null;
  }

  return (
    <div className="min-h-screen py-12">
      <Container>
        <div className="max-w-4xl mx-auto">
          {/* Header */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            className="mb-12"
          >
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-3">
                <Shield className="w-10 h-10 text-brand-primary" />
                <h1 className="text-3xl font-black">
                  <GradientText>Dashboard</GradientText>
                </h1>
              </div>
              <div className="flex items-center gap-3">
                <Button variant="outline" onClick={() => setProfileModalOpen(true)} className="gap-2">
                  <User className="w-4 h-4" />
                  Profile
                </Button>
                <Button variant="outline" onClick={handleLogout} className="gap-2">
                  <LogOut className="w-4 h-4" />
                  Logout
                </Button>
              </div>
            </div>

            {/* Welcome Message */}
            <div className="text-2xl font-semibold text-text-secondary">
              Welcome back, <span className="text-brand-primary">{user.full_name || user.email.split('@')[0]}</span>!
            </div>
          </motion.div>

          {/* Wallet Card */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.2 }}
          >
            <Card>
              <div className="flex items-start justify-between mb-6">
                <div>
                  <h2 className="text-2xl font-bold mb-2">Your Wallet</h2>
                  <p className="text-text-secondary">
                    Zcash shielded wallet
                  </p>
                </div>
                <div className="w-16 h-16 rounded-full bg-brand-secondary/10 flex items-center justify-center">
                  <Wallet className="w-8 h-8 text-brand-secondary" />
                </div>
              </div>

              {user?.wallet_address ? (
                <div className="space-y-4">
                  <div className="p-6 rounded-lg bg-gradient-to-br from-brand-primary/10 to-brand-secondary/10 border border-brand-primary/20">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm text-text-tertiary">Shielded Address (Private)</span>
                      <button
                        onClick={copyAddress}
                        className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-brand-primary/10 hover:bg-brand-primary/20 transition-colors"
                      >
                        {copied ? (
                          <>
                            <Check className="w-4 h-4 text-brand-primary" />
                            <span className="text-sm text-brand-primary">Copied!</span>
                          </>
                        ) : (
                          <>
                            <Copy className="w-4 h-4 text-brand-primary" />
                            <span className="text-sm text-brand-primary">Copy</span>
                          </>
                        )}
                      </button>
                    </div>
                    <p className="font-mono text-sm break-all text-text-primary">
                      {user.wallet_address}
                    </p>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-4 rounded-lg bg-bg-secondary">
                      <div className="flex items-center justify-between mb-1">
                        <div className="text-sm text-text-tertiary">Balance</div>
                        <button
                          onClick={fetchBalance}
                          disabled={balanceLoading}
                          className="text-text-tertiary hover:text-brand-primary transition-colors disabled:opacity-50"
                          title="Refresh balance"
                        >
                          <RefreshCw className={`w-4 h-4 ${balanceLoading ? 'animate-spin' : ''}`} />
                        </button>
                      </div>
                      {balanceLoading ? (
                        <div className="text-2xl font-bold text-brand-primary">Loading...</div>
                      ) : (
                        <div className="text-2xl font-bold text-brand-primary">
                          {balance?.balance_zec || '0.00000000'} ZEC
                        </div>
                      )}
                    </div>
                    <div className="p-4 rounded-lg bg-bg-secondary">
                      <div className="text-sm text-text-tertiary mb-1">Network</div>
                      <div className="text-lg font-semibold">Mainnet</div>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-3 mb-3">
                    <Button
                      variant="primary"
                      className="w-full"
                      onClick={() => setSendModalOpen(true)}
                      disabled={balanceLoading || parseFloat(balance?.balance_zec || '0') <= 0}
                    >
                      Send
                    </Button>
                    <Button
                      variant="primary"
                      className="w-full gap-2"
                      onClick={() => setScanModalOpen(true)}
                      disabled={balanceLoading || parseFloat(balance?.balance_zec || '0') <= 0}
                    >
                      <QrCode className="w-4 h-4" />
                      Scan to Pay
                    </Button>
                  </div>
                  <div className="grid grid-cols-2 gap-3">
                    <Button
                      variant="outline"
                      className="w-full"
                      onClick={copyAddress}
                    >
                      {copied ? 'Copied!' : 'Receive'}
                    </Button>
                    <Button
                      variant="outline"
                      className="w-full gap-2"
                      onClick={() => setHistorySidebarOpen(true)}
                    >
                      <History className="w-4 h-4" />
                      History
                    </Button>
                  </div>
                </div>
              ) : null}
            </Card>
          </motion.div>

          {/* Solana Wallet Card */}
          {user?.solana_address && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.4 }}
              className="mt-6"
            >
              <Card>
                <div className="flex items-start justify-between mb-6">
                  <div>
                    <h2 className="text-2xl font-bold mb-2">Solana Wallet</h2>
                    <p className="text-text-secondary">
                      Bridge SOL to your Zcash wallet
                    </p>
                  </div>
                  <div className="w-16 h-16 rounded-full bg-purple-500/10 flex items-center justify-center">
                    <Wallet className="w-8 h-8 text-purple-500" />
                  </div>
                </div>

                <div className="space-y-4">
                  <div className="p-6 rounded-lg bg-gradient-to-br from-purple-500/10 to-pink-500/10 border border-purple-500/20">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm text-text-tertiary">Solana Address</span>
                      <button
                        onClick={copySolanaAddress}
                        className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-purple-500/10 hover:bg-purple-500/20 transition-colors"
                      >
                        {copiedSolana ? (
                          <>
                            <Check className="w-4 h-4 text-purple-500" />
                            <span className="text-sm text-purple-500">Copied!</span>
                          </>
                        ) : (
                          <>
                            <Copy className="w-4 h-4 text-purple-500" />
                            <span className="text-sm text-purple-500">Copy</span>
                          </>
                        )}
                      </button>
                    </div>
                    <p className="font-mono text-sm break-all text-text-primary">
                      {user.solana_address}
                    </p>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-4 rounded-lg bg-bg-secondary">
                      <div className="flex items-center justify-between mb-1">
                        <div className="text-sm text-text-tertiary">Balance</div>
                        <button
                          onClick={fetchSolanaBalance}
                          disabled={solanaBalanceLoading}
                          className="text-text-tertiary hover:text-purple-500 transition-colors disabled:opacity-50"
                          title="Refresh balance"
                        >
                          <RefreshCw className={`w-4 h-4 ${solanaBalanceLoading ? 'animate-spin' : ''}`} />
                        </button>
                      </div>
                      {solanaBalanceLoading ? (
                        <div className="text-2xl font-bold text-purple-500">Loading...</div>
                      ) : (
                        <div className="text-2xl font-bold text-purple-500">
                          {solanaBalance?.balance_sol?.toFixed(4) || '0.0000'} SOL
                        </div>
                      )}
                    </div>
                    <div className="p-4 rounded-lg bg-bg-secondary">
                      <div className="text-sm text-text-tertiary mb-1">Network</div>
                      <div className="text-lg font-semibold">Mainnet</div>
                    </div>
                  </div>

                  <Button
                    variant="primary"
                    className="w-full gap-2"
                    onClick={() => setBridgeModalOpen(true)}
                    disabled={solanaBalanceLoading || (solanaBalance?.balance_sol || 0) <= 0}
                  >
                    <ArrowRightLeft className="w-4 h-4" />
                    Bridge SOL to Zcash
                  </Button>
                </div>
              </Card>
            </motion.div>
          )}
        </div>
      </Container>

      {/* Transaction History Sidebar */}
      {user?.id && (
        <TransactionHistorySidebar
          userId={user.id}
          isOpen={historySidebarOpen}
          onClose={() => setHistorySidebarOpen(false)}
        />
      )}

      {/* Send Transaction Modal */}
      {user?.id && (
        <SendTransactionModal
          isOpen={sendModalOpen}
          onClose={() => {
            setSendModalOpen(false);
            // Clear prefill values when modal closes
            setPrefillAddress('');
            setPrefillAmount('');
            setPrefillMemo('');
          }}
          userId={user.id}
          currentBalance={balance?.balance_zec || '0.00000000'}
          onSuccess={handleSendSuccess}
          initialAddress={prefillAddress}
          initialAmount={prefillAmount}
          initialMemo={prefillMemo}
        />
      )}

      {/* Scan to Pay Modal */}
      {user?.email && (
        <ScanToPayModal
          isOpen={scanModalOpen}
          onClose={() => setScanModalOpen(false)}
          userEmail={user.email}
          onPaymentScanned={handlePaymentScanned}
        />
      )}

      {/* Profile Sidebar */}
      {user && (
        <ProfileSidebar
          isOpen={profileModalOpen}
          onClose={() => setProfileModalOpen(false)}
          user={user}
        />
      )}

      {/* Bridge Modal */}
      {user?.id && user?.wallet_address && user?.solana_address && (
        <BridgeModal
          isOpen={bridgeModalOpen}
          onClose={() => setBridgeModalOpen(false)}
          userId={user.id}
          currentBalance={solanaBalance?.balance_sol || 0}
          zcashAddress={user.wallet_address}
          solanaAddress={user.solana_address}
          onSuccess={() => {
            fetchBalance();
            fetchSolanaBalance();
          }}
        />
      )}
    </div>
  );
}
