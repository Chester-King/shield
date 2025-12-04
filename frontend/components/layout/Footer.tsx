import Link from 'next/link';
import { Container } from './Container';
import { Shield, Github, Twitter } from 'lucide-react';

export function Footer() {
  return (
    <footer className="relative border-t border-border-primary bg-bg-secondary/50 backdrop-blur-glass">
      <Container>
        <div className="py-12 md:py-16">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8 md:gap-12">
            {/* Brand Column */}
            <div className="col-span-1 md:col-span-2">
              <Link href="/" className="flex items-center gap-2 mb-4">
                <Shield className="w-6 h-6 text-brand-primary" />
                <span className="text-xl font-bold text-gradient">Shield</span>
              </Link>
              <p className="text-text-secondary text-sm max-w-xs mb-6">
                Privacy-first embedded wallet for Zcash. Shielded transactions as simple as email login.
              </p>
              <div className="flex items-center gap-4">
                <Link
                  href="https://github.com"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-text-secondary hover:text-brand-primary transition-colors"
                  aria-label="GitHub"
                >
                  <Github className="w-5 h-5" />
                </Link>
                <Link
                  href="https://twitter.com"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-text-secondary hover:text-brand-primary transition-colors"
                  aria-label="Twitter"
                >
                  <Twitter className="w-5 h-5" />
                </Link>
              </div>
            </div>

            {/* Links Column 1 */}
            <div>
              <h3 className="text-text-primary font-semibold mb-4">Product</h3>
              <ul className="space-y-3">
                <li>
                  <Link href="#features" className="text-text-secondary hover:text-brand-primary transition-colors text-sm">
                    Features
                  </Link>
                </li>
                <li>
                  <Link href="#how-it-works" className="text-text-secondary hover:text-brand-primary transition-colors text-sm">
                    How It Works
                  </Link>
                </li>
                <li>
                  <Link href="#security" className="text-text-secondary hover:text-brand-primary transition-colors text-sm">
                    Security
                  </Link>
                </li>
              </ul>
            </div>

            {/* Links Column 2 */}
            <div>
              <h3 className="text-text-primary font-semibold mb-4">Resources</h3>
              <ul className="space-y-3">
                <li>
                  <Link href="/docs" className="text-text-secondary hover:text-brand-primary transition-colors text-sm">
                    Documentation
                  </Link>
                </li>
                <li>
                  <Link href="https://z.cash" target="_blank" rel="noopener noreferrer" className="text-text-secondary hover:text-brand-primary transition-colors text-sm">
                    Zcash
                  </Link>
                </li>
                <li>
                  <Link href="https://github.com" target="_blank" rel="noopener noreferrer" className="text-text-secondary hover:text-brand-primary transition-colors text-sm">
                    GitHub
                  </Link>
                </li>
              </ul>
            </div>
          </div>

          {/* Bottom Bar */}
          <div className="mt-12 pt-8 border-t border-border-primary flex flex-col md:flex-row items-center justify-between gap-4">
            <p className="text-text-tertiary text-sm">
              © {new Date().getFullYear()} Shield. Built for Zcash privacy.
            </p>
            <div className="flex items-center gap-6">
              <Link href="/privacy" className="text-text-tertiary hover:text-brand-primary transition-colors text-sm">
                Privacy
              </Link>
              <Link href="/terms" className="text-text-tertiary hover:text-brand-primary transition-colors text-sm">
                Terms
              </Link>
            </div>
          </div>
        </div>
      </Container>
    </footer>
  );
}
