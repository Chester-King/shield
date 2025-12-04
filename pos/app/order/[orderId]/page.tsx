'use client';

import { useEffect, useState, useCallback } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { QRCodeDisplay } from '@/components/order/QRCodeDisplay';
import { OrderStatus } from '@/components/order/OrderStatus';
import { Button } from '@/components/ui/Button';
import { ArrowLeft, RefreshCw, Zap } from 'lucide-react';
import { MENU_ITEMS } from '@/lib/constants/menu';
import { posWalletAPI, Transaction } from '@/lib/api/wallet';
import { MERCHANT_CONFIG } from '@/lib/constants/merchant';

// Demo mode flag - set to true to enable manual simulate button
const DEMO_MODE = false;

interface OrderData {
  cart: Record<string, number>;
  total: number;
  summary: string;
  createdAt: number;
}

type PaymentStatus = 'pending' | 'paid' | 'expired';

export default function OrderPage() {
  const params = useParams();
  const router = useRouter();
  const orderId = params.orderId as string;

  const [orderData, setOrderData] = useState<OrderData | null>(null);
  const [paymentStatus, setPaymentStatus] = useState<PaymentStatus>('pending');
  const [customerEmail, setCustomerEmail] = useState<string | undefined>();
  const [isPolling, setIsPolling] = useState(false);
  const [demoEmail, setDemoEmail] = useState('customer@example.com');

  // Load order data from sessionStorage
  useEffect(() => {
    const stored = sessionStorage.getItem(`order_${orderId}`);
    if (stored) {
      setOrderData(JSON.parse(stored));
    }
  }, [orderId]);

  // Check for payment by polling the transactions API
  const checkPayment = useCallback(async () => {
    setIsPolling(true);
    try {
      const response = await posWalletAPI.getTransactions(MERCHANT_CONFIG.user_id);

      // Look for a transaction with memo containing this order ID
      const matchingTx = response.transactions.find((tx: Transaction) =>
        tx.direction === 'received' &&
        tx.memo?.includes(`ORDER:${orderId}`)
      );

      if (matchingTx) {
        setPaymentStatus('paid');
        // Extract email from memo: ORDER:<id>|EMAIL:<email>
        const emailMatch = matchingTx.memo?.match(/EMAIL:([^|]+)/);
        if (emailMatch) {
          setCustomerEmail(emailMatch[1]);
        }
      }
    } catch (error) {
      console.error('Failed to check payment:', error);
    } finally {
      setIsPolling(false);
    }
  }, [orderId]);

  // Poll for payment status periodically
  useEffect(() => {
    if (paymentStatus === 'pending') {
      // Check immediately on mount
      checkPayment();
      // Then poll every 10 seconds
      const interval = setInterval(checkPayment, 10000);
      return () => clearInterval(interval);
    }
  }, [paymentStatus, checkPayment]);

  // Simulate payment (demo mode only)
  const simulatePayment = () => {
    // Store the demo payment in sessionStorage
    sessionStorage.setItem(
      `payment_${orderId}`,
      JSON.stringify({ email: demoEmail, timestamp: Date.now() })
    );
    // Trigger check
    checkPayment();
  };

  if (!orderData) {
    return (
      <div className="min-h-screen bg-bg-primary flex items-center justify-center">
        <div className="text-center">
          <p className="text-text-secondary mb-4">Order not found</p>
          <Button onClick={() => router.push('/')}>
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back to Menu
          </Button>
        </div>
      </div>
    );
  }

  const cartItems = Object.entries(orderData.cart)
    .filter(([, qty]) => qty > 0)
    .map(([itemId, qty]) => ({
      item: MENU_ITEMS.find(m => m.id === itemId)!,
      quantity: qty,
    }));

  return (
    <div className="min-h-screen bg-bg-primary">
      {/* Header */}
      <header className="border-b border-brand-primary/20 bg-bg-secondary/50 backdrop-blur-lg">
        <div className="max-w-4xl mx-auto px-4 py-4">
          <Button variant="ghost" onClick={() => router.push('/')}>
            <ArrowLeft className="w-4 h-4 mr-2" />
            New Order
          </Button>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-4xl mx-auto px-4 py-8">
        <div className="grid md:grid-cols-2 gap-8">
          {/* Left: QR Code */}
          <div className="flex flex-col items-center">
            {paymentStatus === 'pending' ? (
              <QRCodeDisplay
                orderId={orderId}
                amountZec={orderData.total}
                itemsSummary={orderData.summary}
              />
            ) : paymentStatus === 'paid' ? (
              <div className="text-center">
                <div className="w-32 h-32 mx-auto mb-6 rounded-full bg-green-400/20 flex items-center justify-center">
                  <span className="text-6xl">✓</span>
                </div>
                <h2 className="text-3xl font-bold text-green-400 mb-2">
                  Payment Received!
                </h2>
                <p className="text-text-secondary">
                  Thank you for your order
                </p>
              </div>
            ) : null}
          </div>

          {/* Right: Order Details */}
          <div className="space-y-6">
            {/* Order Status */}
            <OrderStatus status={paymentStatus} email={customerEmail} />

            {/* Order Summary */}
            <div className="glass-card p-6">
              <h3 className="text-lg font-semibold text-text-primary mb-4">
                Order Summary
              </h3>
              <div className="space-y-3 mb-4">
                {cartItems.map(({ item, quantity }) => (
                  <div
                    key={item.id}
                    className="flex justify-between items-center text-sm"
                  >
                    <span className="text-text-secondary">
                      {quantity}x {item.name}
                    </span>
                    <span className="text-text-primary font-medium">
                      {(item.price_zec * quantity).toFixed(4)} ZEC
                    </span>
                  </div>
                ))}
              </div>
              <div className="border-t border-brand-primary/20 pt-4">
                <div className="flex justify-between items-center">
                  <span className="font-semibold text-text-primary">Total</span>
                  <span className="text-xl font-bold text-brand-primary">
                    {orderData.total.toFixed(4)} ZEC
                  </span>
                </div>
              </div>
            </div>

            {/* Refresh Button */}
            {paymentStatus === 'pending' && (
              <Button
                variant="outline"
                className="w-full"
                onClick={checkPayment}
                disabled={isPolling}
              >
                <RefreshCw
                  className={`w-4 h-4 mr-2 ${isPolling ? 'animate-spin' : ''}`}
                />
                {isPolling ? 'Checking...' : 'Check Payment Status'}
              </Button>
            )}

            {/* New Order Button */}
            {paymentStatus === 'paid' && (
              <Button
                variant="primary"
                className="w-full"
                onClick={() => router.push('/')}
              >
                Start New Order
              </Button>
            )}

            {/* Demo Mode Controls */}
            {DEMO_MODE && paymentStatus === 'pending' && (
              <div className="mt-6 p-4 border border-yellow-500/30 bg-yellow-500/5 rounded-lg">
                <div className="flex items-center gap-2 mb-3">
                  <Zap className="w-4 h-4 text-yellow-500" />
                  <span className="text-sm font-semibold text-yellow-500">Demo Mode</span>
                </div>
                <p className="text-xs text-text-tertiary mb-3">
                  Simulate a payment to test the flow
                </p>
                <input
                  type="email"
                  value={demoEmail}
                  onChange={(e) => setDemoEmail(e.target.value)}
                  placeholder="Customer email"
                  className="w-full px-3 py-2 mb-3 bg-bg-secondary border border-brand-primary/10 rounded-lg text-sm focus:outline-none focus:border-brand-primary/50"
                />
                <Button
                  variant="outline"
                  className="w-full gap-2 border-yellow-500/30 text-yellow-500 hover:bg-yellow-500/10"
                  onClick={simulatePayment}
                >
                  <Zap className="w-4 h-4" />
                  Simulate Payment
                </Button>
              </div>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
