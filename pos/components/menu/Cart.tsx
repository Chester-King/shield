'use client';

import { MENU_ITEMS, MenuItem } from '@/lib/constants/menu';
import { Button } from '@/components/ui/Button';
import { ShoppingCart, Trash2 } from 'lucide-react';

interface CartProps {
  cart: Record<string, number>;
  onClearCart: () => void;
  onCheckout: () => void;
}

export function Cart({ cart, onClearCart, onCheckout }: CartProps) {
  const cartItems = Object.entries(cart)
    .filter(([, qty]) => qty > 0)
    .map(([itemId, qty]) => ({
      item: MENU_ITEMS.find(m => m.id === itemId)!,
      quantity: qty,
    }));

  const totalItems = cartItems.reduce((sum, { quantity }) => sum + quantity, 0);
  const totalZec = cartItems.reduce(
    (sum, { item, quantity }) => sum + item.price_zec * quantity,
    0
  );

  if (totalItems === 0) {
    return (
      <div className="glass-card p-6 text-center">
        <ShoppingCart className="w-12 h-12 mx-auto mb-3 text-text-tertiary" />
        <p className="text-text-secondary">Your cart is empty</p>
        <p className="text-sm text-text-tertiary mt-1">Add items to get started</p>
      </div>
    );
  }

  return (
    <div className="glass-card p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold text-text-primary flex items-center gap-2">
          <ShoppingCart className="w-5 h-5" />
          Cart ({totalItems})
        </h2>
        <button
          onClick={onClearCart}
          className="text-text-tertiary hover:text-red-400 transition-colors"
        >
          <Trash2 className="w-5 h-5" />
        </button>
      </div>

      {/* Cart Items */}
      <div className="space-y-3 mb-4">
        {cartItems.map(({ item, quantity }) => (
          <div key={item.id} className="flex justify-between items-center text-sm">
            <span className="text-text-secondary">
              {quantity}x {item.name}
            </span>
            <span className="text-text-primary font-medium">
              {(item.price_zec * quantity).toFixed(4)} ZEC
            </span>
          </div>
        ))}
      </div>

      {/* Divider */}
      <div className="border-t border-brand-primary/20 my-4" />

      {/* Total */}
      <div className="flex justify-between items-center mb-6">
        <span className="text-lg font-semibold text-text-primary">Total</span>
        <span className="text-2xl font-bold text-brand-primary">
          {totalZec.toFixed(4)} ZEC
        </span>
      </div>

      {/* Checkout Button */}
      <Button
        variant="primary"
        size="lg"
        className="w-full"
        onClick={onCheckout}
      >
        Generate Payment QR
      </Button>
    </div>
  );
}

export function getCartSummary(cart: Record<string, number>): string {
  return Object.entries(cart)
    .filter(([, qty]) => qty > 0)
    .map(([itemId, qty]) => {
      const item = MENU_ITEMS.find(m => m.id === itemId);
      return `${qty}x ${item?.name || itemId}`;
    })
    .join(', ');
}

export function getCartTotal(cart: Record<string, number>): number {
  return Object.entries(cart)
    .filter(([, qty]) => qty > 0)
    .reduce((sum, [itemId, qty]) => {
      const item = MENU_ITEMS.find(m => m.id === itemId);
      return sum + (item?.price_zec || 0) * qty;
    }, 0);
}
