'use client';

import { Container } from '../layout/Container';
import { FeatureCard } from '../ui/Card';
import { motion } from 'framer-motion';
import { Shield, Zap, UserCheck, Code } from 'lucide-react';

export function Features() {
  const features = [
    {
      icon: <Shield className="w-6 h-6" />,
      title: 'True Privacy by Default',
      description: 'All transactions use Zcash shielded pools with zk-SNARK proofs. Your financial data stays completely private.',
    },
    {
      icon: <UserCheck className="w-6 h-6" />,
      title: 'No Seed Phrases',
      description: 'Login with email or social auth. We handle wallet security using distributed MPC technology.',
    },
    {
      icon: <Zap className="w-6 h-6" />,
      title: 'Instant Onboarding',
      description: 'Get a fully functional shielded wallet in under 10 seconds. No downloads, no complexity.',
    },
    {
      icon: <Code className="w-6 h-6" />,
      title: 'Developer-Friendly SDK',
      description: 'Integrate privacy-preserving wallets into your app with just a few lines of code.',
    },
  ];

  return (
    <section id="features" className="relative py-20 md:py-28">
      <Container>
        <div className="max-w-6xl mx-auto">
          {/* Section Header */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="text-center mb-16"
          >
            <h2 className="text-3xl md:text-4xl lg:text-5xl font-black mb-6">
              Built for <span className="text-gradient">Privacy & Speed</span>
            </h2>
            <p className="text-lg md:text-xl text-text-secondary leading-relaxed max-w-2xl mx-auto">
              Enterprise-grade privacy meets consumer-grade simplicity.
              The best of both worlds.
            </p>
          </motion.div>

          {/* Features Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 md:gap-8">
            {features.map((feature, index) => (
              <motion.div
                key={feature.title}
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
              >
                <FeatureCard
                  icon={feature.icon}
                  title={feature.title}
                  description={feature.description}
                />
              </motion.div>
            ))}
          </div>

          {/* Additional Info */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.5 }}
            className="mt-16 p-8 md:p-10 rounded-2xl border border-brand-primary/20 bg-gradient-to-br from-brand-primary/5 to-brand-secondary/5"
          >
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-center">
              <div>
                <h3 className="text-2xl md:text-3xl font-bold mb-4">
                  Powered by <span className="text-gradient">Zcash Protocol</span>
                </h3>
                <p className="text-text-secondary leading-relaxed">
                  Shield leverages Zcash's Orchard shielded pool and Unified Addresses
                  to provide cryptographically guaranteed privacy for all transactions.
                </p>
              </div>
              <div className="grid grid-cols-2 gap-6">
                <div className="text-center">
                  <div className="text-3xl font-bold text-brand-primary mb-2">256-bit</div>
                  <div className="text-sm text-text-tertiary">Encryption</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-brand-secondary mb-2">zk-SNARK</div>
                  <div className="text-sm text-text-tertiary">Proofs</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-brand-primary mb-2">MPC</div>
                  <div className="text-sm text-text-tertiary">Key Management</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-brand-secondary mb-2">Open</div>
                  <div className="text-sm text-text-tertiary">Source</div>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </Container>

      {/* Background decoration */}
      <div className="absolute inset-0 -z-10 overflow-hidden pointer-events-none">
        <div className="absolute top-1/3 -left-32 w-96 h-96 bg-brand-secondary/10 rounded-full blur-3xl" />
        <div className="absolute bottom-1/3 -right-32 w-96 h-96 bg-brand-primary/10 rounded-full blur-3xl" />
      </div>
    </section>
  );
}
