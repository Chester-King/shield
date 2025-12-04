'use client';

import { CheckCircle, Clock, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';

type OrderStatusType = 'pending' | 'paid' | 'expired';

interface OrderStatusProps {
  status: OrderStatusType;
  email?: string;
}

export function OrderStatus({ status, email }: OrderStatusProps) {
  const statusConfig = {
    pending: {
      icon: Clock,
      text: 'Awaiting Payment',
      color: 'text-yellow-400',
      bgColor: 'bg-yellow-400/10',
      borderColor: 'border-yellow-400/30',
    },
    paid: {
      icon: CheckCircle,
      text: 'Payment Received!',
      color: 'text-green-400',
      bgColor: 'bg-green-400/10',
      borderColor: 'border-green-400/30',
    },
    expired: {
      icon: AlertCircle,
      text: 'Order Expired',
      color: 'text-red-400',
      bgColor: 'bg-red-400/10',
      borderColor: 'border-red-400/30',
    },
  };

  const config = statusConfig[status];
  const Icon = config.icon;

  return (
    <div
      className={cn(
        'rounded-lg p-4 border',
        config.bgColor,
        config.borderColor
      )}
    >
      <div className="flex items-center gap-3">
        <Icon className={cn('w-6 h-6', config.color)} />
        <div>
          <p className={cn('font-semibold', config.color)}>{config.text}</p>
          {status === 'paid' && email && (
            <p className="text-sm text-text-secondary">Customer: {email}</p>
          )}
          {status === 'pending' && (
            <p className="text-sm text-text-tertiary">
              Waiting for transaction confirmation...
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
