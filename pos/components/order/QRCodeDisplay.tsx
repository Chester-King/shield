'use client';

import { QRCodeSVG } from 'qrcode.react';
import { MERCHANT_CONFIG, POSPaymentRequest } from '@/lib/constants/merchant';

interface QRCodeDisplayProps {
  orderId: string;
  amountZec: number;
  itemsSummary: string;
}

export function QRCodeDisplay({ orderId, amountZec, itemsSummary }: QRCodeDisplayProps) {
  const paymentData: POSPaymentRequest = {
    v: 1,
    oid: orderId,
    to: MERCHANT_CONFIG.address,
    amt: amountZec,
    mn: MERCHANT_CONFIG.name,
    sum: itemsSummary,
  };

  return (
    <div className="flex flex-col items-center gap-6">
      {/* QR Code with white background */}
      <div className="p-6 bg-white rounded-2xl shadow-lg">
        <QRCodeSVG
          value={JSON.stringify(paymentData)}
          size={280}
          level="M"
          includeMargin={true}
        />
      </div>

      {/* Payment Details */}
      <div className="text-center">
        <p className="text-3xl font-bold text-brand-primary mb-2">
          {amountZec.toFixed(4)} ZEC
        </p>
        <p className="text-sm text-text-secondary">
          Order #{orderId.slice(0, 8)}
        </p>
      </div>

      {/* Instructions */}
      <div className="text-center text-sm text-text-tertiary max-w-xs">
        <p>Scan with your Shield Wallet to pay</p>
        <p className="mt-1">Your email will be shared for order confirmation</p>
      </div>
    </div>
  );
}
