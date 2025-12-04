'use client';

import { Container } from '../layout/Container';
import { Card } from '../ui/Card';
import { motion } from 'framer-motion';
import { ShieldAlert, KeyRound, Lock } from 'lucide-react';

export function Problem() {
  const problems = [
    {
      icon: <KeyRound className="w-6 h-6" />,
      title: 'Complex Seed Phrases',
      description: 'Traditional Zcash wallets require users to manage 24-word seed phrases, creating friction and security risks.',
    },
    {
      icon: <ShieldAlert className="w-6 h-6" />,
      title: 'Privacy Without Convenience',
      description: 'Existing embedded wallets prioritize ease-of-use but sacrifice the privacy guarantees that make Zcash unique.',
    },
    {
      icon: <Lock className="w-6 h-6" />,
      title: 'High Barrier to Entry',
      description: 'Users must understand complex cryptographic concepts before they can benefit from shielded transactions.',
    },
  ];

  return (
    <section className="relative py-20 md:py-28">
      <Container>
        <div className="max-w-4xl mx-auto">
          {/* Section Header */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="text-center mb-16"
          >
            <h2 className="text-3xl md:text-4xl lg:text-5xl font-black mb-6">
              The Problem with{' '}
              <span className="text-gradient">Privacy Today</span>
            </h2>
            <p className="text-lg md:text-xl text-text-secondary leading-relaxed max-w-2xl mx-auto">
              Users shouldn't have to choose between privacy and convenience.
              Yet current solutions force this trade-off.
            </p>
          </motion.div>

          {/* Problem Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 md:gap-8">
            {problems.map((problem, index) => (
              <motion.div
                key={problem.title}
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
              >
                <Card className="h-full">
                  <div className="flex flex-col gap-4">
                    <div className="w-12 h-12 rounded-lg bg-brand-accent/10 flex items-center justify-center text-brand-accent">
                      {problem.icon}
                    </div>
                    <h3 className="text-xl font-semibold text-text-primary">
                      {problem.title}
                    </h3>
                    <p className="text-text-secondary leading-relaxed">
                      {problem.description}
                    </p>
                  </div>
                </Card>
              </motion.div>
            ))}
          </div>
        </div>
      </Container>

      {/* Background decoration */}
      <div className="absolute inset-0 -z-10 overflow-hidden pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-brand-accent/5 rounded-full blur-3xl" />
      </div>
    </section>
  );
}
