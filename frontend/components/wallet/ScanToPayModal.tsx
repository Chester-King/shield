'use client';

import { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Camera, AlertCircle, QrCode } from 'lucide-react';
import { Html5Qrcode } from 'html5-qrcode';
import { Button } from '@/components/ui/Button';

interface POSPaymentData {
  v: number;
  oid: string;
  to: string;
  amt: number;
  mn: string;
  sum: string;
}

interface ScanToPayModalProps {
  isOpen: boolean;
  onClose: () => void;
  userEmail: string;
  onPaymentScanned: (toAddress: string, amount: string, memo: string) => void;
}

export function ScanToPayModal({
  isOpen,
  onClose,
  userEmail,
  onPaymentScanned,
}: ScanToPayModalProps) {
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const scannerRef = useRef<Html5Qrcode | null>(null);
  const scannerContainerId = 'qr-scanner-container';

  const startScanner = async () => {
    setError(null);
    setScanning(true);

    try {
      const html5Qrcode = new Html5Qrcode(scannerContainerId);
      scannerRef.current = html5Qrcode;

      await html5Qrcode.start(
        { facingMode: 'environment' },
        {
          fps: 10,
          qrbox: { width: 250, height: 250 },
        },
        (decodedText) => {
          handleQRCodeScanned(decodedText);
        },
        () => {
          // QR code not found - ignore
        }
      );
    } catch (err) {
      console.error('Scanner error:', err);
      setError('Failed to access camera. Please ensure camera permissions are granted.');
      setScanning(false);
    }
  };

  const stopScanner = async () => {
    if (scannerRef.current) {
      try {
        await scannerRef.current.stop();
        scannerRef.current = null;
      } catch (err) {
        console.error('Error stopping scanner:', err);
      }
    }
    setScanning(false);
  };

  const handleQRCodeScanned = async (decodedText: string) => {
    try {
      const paymentData: POSPaymentData = JSON.parse(decodedText);

      // Validate required fields
      if (!paymentData.v || !paymentData.oid || !paymentData.to || !paymentData.amt) {
        throw new Error('Invalid payment QR code');
      }

      // Generate memo with order ID and email
      const memo = `ORDER:${paymentData.oid}|EMAIL:${userEmail}`;

      // Stop scanner before callback
      await stopScanner();

      // Call the callback with parsed data
      onPaymentScanned(
        paymentData.to,
        paymentData.amt.toString(),
        memo
      );

      onClose();
    } catch (err) {
      setError('Invalid QR code. Please scan a valid Shield Cafe payment QR.');
    }
  };

  // Start scanner when modal opens
  useEffect(() => {
    if (isOpen) {
      // Small delay to ensure DOM is ready
      const timer = setTimeout(() => {
        startScanner();
      }, 100);
      return () => clearTimeout(timer);
    } else {
      stopScanner();
    }
  }, [isOpen]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stopScanner();
    };
  }, []);

  const handleClose = () => {
    stopScanner();
    onClose();
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
                    <QrCode className="w-5 h-5 text-brand-primary" />
                  </div>
                  <h2 className="text-2xl font-bold">Scan to Pay</h2>
                </div>
                <button
                  onClick={handleClose}
                  className="p-2 hover:bg-brand-primary/10 rounded-lg transition-colors"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>

              {/* Content */}
              <div className="p-6">
                {/* Scanner Container */}
                <div className="relative mb-4">
                  <div
                    id={scannerContainerId}
                    className="w-full aspect-square rounded-lg overflow-hidden bg-bg-secondary"
                  />

                  {!scanning && !error && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-bg-secondary rounded-lg">
                      <Camera className="w-16 h-16 text-text-tertiary mb-4" />
                      <p className="text-text-secondary">Starting camera...</p>
                    </div>
                  )}
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

                {/* Instructions */}
                <div className="text-center text-sm text-text-tertiary">
                  <p>Point your camera at the POS payment QR code</p>
                  <p className="mt-1">Your email will be shared for order confirmation</p>
                </div>

                {/* Retry Button */}
                {error && (
                  <Button
                    onClick={startScanner}
                    variant="outline"
                    className="w-full mt-4 gap-2"
                  >
                    <Camera className="w-4 h-4" />
                    Try Again
                  </Button>
                )}
              </div>
            </motion.div>
          </div>
        </>
      )}
    </AnimatePresence>
  );
}
