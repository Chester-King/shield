'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { Container } from '@/components/layout/Container';
import { GradientText } from '@/components/ui/GradientText';
import { Shield, AlertCircle, Eye, EyeOff } from 'lucide-react';
import { authAPI } from '@/lib/api/auth';
import { useAuth } from '@/contexts/AuthContext';

export default function LoginPage() {
  const router = useRouter();
  const { setTokens } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!email || !password) {
      setError('Please fill in all fields');
      return;
    }

    setLoading(true);
    try {
      const response = await authAPI.login(email, password);
      await setTokens(response.access_token, response.refresh_token);
      router.push('/dashboard');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center py-12 px-4">
      <Container size="sm">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="w-full max-w-md mx-auto"
        >
          {/* Logo */}
          <div className="flex justify-center mb-8">
            <div className="flex items-center gap-2">
              <Shield className="w-10 h-10 text-brand-primary" />
              <span className="text-2xl font-black">
                <GradientText>Shield</GradientText>
              </span>
            </div>
          </div>

          {/* Card */}
          <div className="glass-card p-8">
            <h1 className="text-3xl font-black text-center mb-2">
              Welcome Back
            </h1>
            <p className="text-text-secondary text-center mb-8">
              Login to access your private wallet
            </p>

            {error && (
              <div className="mb-6 p-4 rounded-lg bg-brand-accent/10 border border-brand-accent/20 flex items-start gap-3">
                <AlertCircle className="w-5 h-5 text-brand-accent flex-shrink-0 mt-0.5" />
                <p className="text-sm text-brand-accent">{error}</p>
              </div>
            )}

            <form onSubmit={handleSubmit} className="space-y-4">
              {/* Email */}
              <div>
                <label htmlFor="email" className="block text-sm font-medium text-text-secondary mb-2">
                  Email Address
                </label>
                <input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="you@example.com"
                  required
                  className="w-full px-4 py-3 rounded-lg border border-text-tertiary/20 bg-bg-secondary text-text-primary placeholder-text-tertiary focus:outline-none focus:border-brand-primary transition-colors"
                />
              </div>

              {/* Password */}
              <div>
                <label htmlFor="password" className="block text-sm font-medium text-text-secondary mb-2">
                  Password
                </label>
                <div className="relative">
                  <input
                    id="password"
                    type={showPassword ? 'text' : 'password'}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="Enter your password"
                    required
                    className="w-full px-4 py-3 rounded-lg border border-text-tertiary/20 bg-bg-secondary text-text-primary placeholder-text-tertiary focus:outline-none focus:border-brand-primary transition-colors pr-12"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-text-tertiary hover:text-text-primary transition-colors"
                  >
                    {showPassword ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                  </button>
                </div>
              </div>

              {/* Submit Button */}
              <button
                type="submit"
                disabled={loading}
                className="w-full px-6 py-3 rounded-lg bg-brand-primary text-white font-medium hover:bg-brand-secondary transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? 'Logging in...' : 'Log In'}
              </button>
            </form>
          </div>

          {/* Sign Up Link */}
          <div className="mt-6 text-center">
            <p className="text-text-tertiary text-sm">
              Don't have an account?{' '}
              <Link href="/signup" className="text-brand-primary hover:text-brand-secondary transition-colors font-medium">
                Sign up
              </Link>
            </p>
          </div>

          {/* Back to home */}
          <div className="mt-4 text-center">
            <Link href="/" className="text-text-tertiary hover:text-brand-primary text-sm transition-colors">
              ← Back to home
            </Link>
          </div>
        </motion.div>
      </Container>
    </div>
  );
}
