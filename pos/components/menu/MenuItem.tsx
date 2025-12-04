'use client';

import { cn } from '@/lib/utils';
import { MenuItem as MenuItemType } from '@/lib/constants/menu';
import { Plus, Minus } from 'lucide-react';
import { Button } from '@/components/ui/Button';

interface MenuItemProps {
  item: MenuItemType;
  quantity: number;
  onAdd: () => void;
  onRemove: () => void;
}

export function MenuItem({ item, quantity, onAdd, onRemove }: MenuItemProps) {
  return (
    <div className={cn(
      'glass-card p-4 transition-all duration-300',
      quantity > 0 && 'border-brand-primary/50 shadow-glow'
    )}>
      {/* Item Image Placeholder */}
      <div className="w-full h-32 rounded-lg bg-bg-tertiary flex items-center justify-center mb-4 overflow-hidden">
        <span className="text-4xl">{getItemEmoji(item.id)}</span>
      </div>

      {/* Item Details */}
      <h3 className="text-lg font-semibold text-text-primary mb-1">{item.name}</h3>
      <p className="text-sm text-text-secondary mb-3 line-clamp-2">{item.description}</p>

      {/* Price and Quantity Controls */}
      <div className="flex items-center justify-between">
        <span className="text-brand-primary font-bold">
          {item.price_zec} ZEC
        </span>

        <div className="flex items-center gap-2">
          {quantity > 0 && (
            <>
              <Button
                variant="outline"
                size="sm"
                onClick={onRemove}
                className="w-8 h-8 p-0"
              >
                <Minus className="w-4 h-4" />
              </Button>
              <span className="w-6 text-center font-semibold text-text-primary">
                {quantity}
              </span>
            </>
          )}
          <Button
            variant="primary"
            size="sm"
            onClick={onAdd}
            className="w-8 h-8 p-0"
          >
            <Plus className="w-4 h-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}

function getItemEmoji(id: string): string {
  switch (id) {
    case 'sunny-side-up':
      return '🍳';
    case 'latte':
      return '☕';
    case 'croissant':
      return '🥐';
    default:
      return '🍽️';
  }
}
