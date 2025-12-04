'use client';

import { Container } from '../layout/Container';
import { Card } from '../ui/Card';
import { motion } from 'framer-motion';
import { ShieldCheck, Lock, Eye, FileCheck } from 'lucide-react';

export function Security() {
  const securityFeatures = [
    {
      icon: <ShieldCheck className="w-6 h-6" />,
      title: 'End-to-End Encryption',
      description: 'All sensitive data is encrypted client-side before leaving your device.',
    },
    {
      icon: <Lock className="w-6 h-6" />,
      title: 'Non-Custodial',
      description: 'You maintain full control. We never have access to your private keys.',
    },
    {
      icon: <Eye className="w-6 h-6" />,
      title: 'Privacy by Design',
      description: 'Zero-knowledge architecture means we can\'t see your transaction history.',
    },
    {
      icon: <FileCheck className="w-6 h-6" />,
      title: 'Audited & Open Source',
      description: 'Regular security audits and fully transparent, open-source codebase.',
    },
  ];

  return (
    <section id="security" className="relative py-20 md:py-28 bg-gradient-to-b from-transparent via-bg-secondary/30 to-transparent">
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
              Security <span className="text-gradient">Without Compromise</span>
            </h2>
            <p className="text-lg md:text-xl text-text-secondary leading-relaxed max-w-2xl mx-auto">
              Your privacy and security are non-negotiable.
              We've built Shield from the ground up with security-first principles.
            </p>
          </motion.div>

          {/* Security Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 md:gap-8 mb-16">
            {securityFeatures.map((feature, index) => (
              <motion.div
                key={feature.title}
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
              >
                <Card>
                  <div className="flex items-start gap-4">
                    <div className="flex-shrink-0 w-12 h-12 rounded-lg bg-brand-primary/10 flex items-center justify-center text-brand-primary">
                      {feature.icon}
                    </div>
                    <div>
                      <h3 className="text-xl font-semibold text-text-primary mb-2">
                        {feature.title}
                      </h3>
                      <p className="text-text-secondary leading-relaxed">
                        {feature.description}
                      </p>
                    </div>
                  </div>
                </Card>
              </motion.div>
            ))}
          </div>

          {/* Trust Indicators */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.5 }}
            className="relative p-8 md:p-10 rounded-2xl border border-brand-primary/20 bg-gradient-to-br from-bg-secondary/80 to-bg-tertiary/80 backdrop-blur-sm"
          >
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-center">
              <div>
                <h3 className="text-2xl md:text-3xl font-bold mb-4">
                  Battle-Tested <span className="text-gradient">Infrastructure</span>
                </h3>
                <p className="text-text-secondary leading-relaxed mb-6">
                  Shield is built on Zcash, the most proven privacy-preserving blockchain
                  with over 6 years of production use and billions in secure transactions.
                </p>
                <div className="flex flex-wrap gap-3">
                  <div className="px-4 py-2 rounded-full bg-brand-primary/10 border border-brand-primary/20 text-brand-primary text-sm font-medium">
                    SOC 2 Type II
                  </div>
                  <div className="px-4 py-2 rounded-full bg-brand-primary/10 border border-brand-primary/20 text-brand-primary text-sm font-medium">
                    ISO 27001
                  </div>
                  <div className="px-4 py-2 rounded-full bg-brand-primary/10 border border-brand-primary/20 text-brand-primary text-sm font-medium">
                    Open Source
                  </div>
                </div>
              </div>

              {/* Security Stats */}
              <div className="grid grid-cols-2 gap-6">
                <div className="text-center p-6 rounded-xl bg-bg-primary/50">
                  <div className="text-3xl font-bold text-brand-primary mb-2">0</div>
                  <div className="text-sm text-text-tertiary">Security Breaches</div>
                </div>
                <div className="text-center p-6 rounded-xl bg-bg-primary/50">
                  <div className="text-3xl font-bold text-brand-secondary mb-2">100%</div>
                  <div className="text-sm text-text-tertiary">Uptime SLA</div>
                </div>
                <div className="text-center p-6 rounded-xl bg-bg-primary/50">
                  <div className="text-3xl font-bold text-brand-primary mb-2">24/7</div>
                  <div className="text-sm text-text-tertiary">Monitoring</div>
                </div>
                <div className="text-center p-6 rounded-xl bg-bg-primary/50">
                  <div className="text-3xl font-bold text-brand-secondary mb-2">256-bit</div>
                  <div className="text-sm text-text-tertiary">Encryption</div>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </Container>

      {/* Background decoration */}
      <div className="absolute inset-0 -z-10 overflow-hidden pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-brand-primary/5 rounded-full blur-3xl" />
      </div>
    </section>
  );
}
