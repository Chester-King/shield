'use client';

import { useEffect, useState, useRef } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';
import { Shield } from 'lucide-react';

export default function AuthCallbackPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { setTokens } = useAuth();
  const [error, setError] = useState('');
  const hasProcessed = useRef(false);

  useEffect(() => {
    // Prevent multiple executions
    if (hasProcessed.current) return;

    const handleCallback = async () => {
      try {
        const accessToken = searchParams.get('access_token');
        const refreshToken = searchParams.get('refresh_token');

        if (!accessToken || !refreshToken) {
          throw new Error('Missing authentication tokens');
        }

        hasProcessed.current = true;

        // Store tokens using AuthContext
        await setTokens(accessToken, refreshToken);

        // Redirect to dashboard
        router.push('/dashboard');
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Authentication failed');
        // Redirect to login after 3 seconds
        setTimeout(() => {
          router.push('/login');
        }, 3000);
      }
    };

    handleCallback();
  }, []);

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="mb-4 text-red-500">
            <p className="text-lg font-semibold">Authentication Error</p>
            <p className="text-sm text-text-secondary mt-2">{error}</p>
          </div>
          <p className="text-sm text-text-tertiary">Redirecting to login...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="text-center">
        <div className="flex justify-center mb-4">
          <Shield className="w-16 h-16 text-primary animate-pulse" />
        </div>
        <h1 className="text-2xl font-bold mb-2">Authenticating...</h1>
        <p className="text-text-secondary">Please wait while we log you in</p>
      </div>
    </div>
  );
}
