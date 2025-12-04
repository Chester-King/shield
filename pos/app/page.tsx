'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { MENU_ITEMS } from '@/lib/constants/menu';
import { MenuItem } from '@/components/menu/MenuItem';
import { Cart, getCartSummary, getCartTotal } from '@/components/menu/Cart';
import { MerchantBalance } from '@/components/wallet/MerchantBalance';
import { Coffee, Shield } from 'lucide-react';
import { v4 as uuidv4 } from 'uuid';

export default function MenuPage() {
  const router = useRouter();
  const [cart, setCart] = useState<Record<string, number>>({});

  const handleAddItem = (itemId: string) => {
    setCart(prev => ({
      ...prev,
      [itemId]: (prev[itemId] || 0) + 1,
    }));
  };

  const handleRemoveItem = (itemId: string) => {
    setCart(prev => ({
      ...prev,
      [itemId]: Math.max(0, (prev[itemId] || 0) - 1),
    }));
  };

  const handleClearCart = () => {
    setCart({});
  };

  const handleCheckout = () => {
    const orderId = uuidv4();
    const total = getCartTotal(cart);
    const summary = getCartSummary(cart);

    // Store order data in sessionStorage for the order page
    sessionStorage.setItem(
      `order_${orderId}`,
      JSON.stringify({
        cart,
        total,
        summary,
        createdAt: Date.now(),
      })
    );

    // Navigate to order page
    router.push(`/order/${orderId}`);
  };

  return (
    <div className="min-h-screen bg-bg-primary">
      {/* Header */}
      <header className="border-b border-brand-primary/20 bg-bg-secondary/50 backdrop-blur-lg sticky top-0 z-10">
        <div className="max-w-6xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-lg bg-brand-primary/10 flex items-center justify-center">
                <Coffee className="w-6 h-6 text-brand-primary" />
              </div>
              <div>
                <h1 className="text-xl font-bold text-text-primary">Shield Cafe</h1>
                <p className="text-xs text-text-tertiary">Privacy-First Payments</p>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <MerchantBalance />
              <div className="flex items-center gap-2 text-sm text-text-secondary">
                <Shield className="w-4 h-4 text-brand-primary" />
                <span>Powered by Zcash</span>
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-6xl mx-auto px-4 py-8">
        <div className="grid lg:grid-cols-3 gap-8">
          {/* Menu Items */}
          <div className="lg:col-span-2">
            <h2 className="text-2xl font-bold text-text-primary mb-6">Menu</h2>
            <div className="grid sm:grid-cols-2 md:grid-cols-3 gap-4">
              {MENU_ITEMS.map(item => (
                <MenuItem
                  key={item.id}
                  item={item}
                  quantity={cart[item.id] || 0}
                  onAdd={() => handleAddItem(item.id)}
                  onRemove={() => handleRemoveItem(item.id)}
                />
              ))}
            </div>
          </div>

          {/* Cart Sidebar */}
          <div className="lg:col-span-1">
            <div className="sticky top-24">
              <Cart
                cart={cart}
                onClearCart={handleClearCart}
                onCheckout={handleCheckout}
              />
            </div>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-brand-primary/10 mt-auto">
        <div className="max-w-6xl mx-auto px-4 py-6 text-center text-sm text-text-tertiary">
          <p>All payments are processed privately using Zcash shielded transactions</p>
        </div>
      </footer>
    </div>
  );
}
