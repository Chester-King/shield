'use client';

import Link from 'next/link';
import { Container } from './Container';
import { Shield } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useState, useEffect } from 'react';
import { useAuth } from '@/contexts/AuthContext';

export function Navbar() {
  const [isScrolled, setIsScrolled] = useState(false);
  const { isAuthenticated, logout } = useAuth();

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 20);
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <nav
      className={cn(
        'fixed top-0 left-0 right-0 z-50 transition-all duration-300',
        isScrolled
          ? 'bg-background-secondary/80 backdrop-blur-lg border-b border-primary/20 shadow-lg'
          : 'bg-transparent'
      )}
    >
      <Container>
        <div className="flex items-center justify-between h-16 md:h-20">
          {/* Logo */}
          <Link href="/" className="flex items-center gap-2 group">
            <div className="relative">
              <Shield className="w-8 h-8 text-primary transition-transform group-hover:scale-110" />
              <div className="absolute inset-0 bg-primary/20 blur-xl opacity-0 group-hover:opacity-100 transition-opacity" />
            </div>
            <span className="text-xl md:text-2xl font-bold text-gradient">
              Shield
            </span>
          </Link>

          {/* Nav Links - Desktop */}
          <div className="hidden md:flex items-center gap-8">
            <Link
              href="#features"
              className="text-text-secondary hover:text-primary transition-colors text-sm font-medium"
            >
              Features
            </Link>
            <Link
              href="#how-it-works"
              className="text-text-secondary hover:text-primary transition-colors text-sm font-medium"
            >
              How It Works
            </Link>
            <Link
              href="#security"
              className="text-text-secondary hover:text-primary transition-colors text-sm font-medium"
            >
              Security
            </Link>
            <Link
              href="https://github.com"
              target="_blank"
              rel="noopener noreferrer"
              className="text-text-secondary hover:text-primary transition-colors text-sm font-medium"
            >
              GitHub
            </Link>
          </div>

          {/* CTA Buttons */}
          <div className="flex items-center gap-4">
            {isAuthenticated ? (
              <>
                <Link
                  href="/dashboard"
                  className={cn(
                    'px-6 py-2.5 rounded-lg font-semibold text-sm',
                    'bg-primary text-background',
                    'hover:shadow-[0_0_20px_rgba(0,255,204,0.5)] transition-all duration-300',
                    'hover:scale-105 active:scale-95'
                  )}
                >
                  Dashboard
                </Link>
              </>
            ) : (
              <>
                <Link
                  href="/login"
                  className="text-text-secondary hover:text-primary transition-colors text-sm font-medium"
                >
                  Login
                </Link>
                <Link
                  href="/register"
                  className={cn(
                    'px-6 py-2.5 rounded-lg font-semibold text-sm',
                    'bg-primary text-background',
                    'hover:shadow-[0_0_20px_rgba(0,255,204,0.5)] transition-all duration-300',
                    'hover:scale-105 active:scale-95'
                  )}
                >
                  Get Started
                </Link>
              </>
            )}
          </div>
        </div>
      </Container>
    </nav>
  );
}
