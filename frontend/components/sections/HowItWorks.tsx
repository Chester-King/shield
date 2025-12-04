'use client';

import { Container } from '../layout/Container';
import { motion } from 'framer-motion';
import { Mail, Wallet, Send } from 'lucide-react';

export function HowItWorks() {
  const steps = [
    {
      number: '01',
      icon: <Mail className="w-8 h-8" />,
      title: 'Simple Login',
      description: 'Sign in with email or Google. No seed phrases, no complex setup. Just like any modern app.',
    },
    {
      number: '02',
      icon: <Wallet className="w-8 h-8" />,
      title: 'Instant Wallet',
      description: 'We generate a shielded Zcash wallet for you automatically using secure MPC technology.',
    },
    {
      number: '03',
      icon: <Send className="w-8 h-8" />,
      title: 'Private Transactions',
      description: 'Send and receive ZEC with full privacy. All transactions use Zcash shielded pools by default.',
    },
  ];

  return (
    <section id="how-it-works" className="relative py-20 md:py-28 bg-gradient-to-b from-transparent via-bg-secondary/50 to-transparent">
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
              How <span className="text-gradient">Shield</span> Works
            </h2>
            <p className="text-lg md:text-xl text-text-secondary leading-relaxed max-w-2xl mx-auto">
              Privacy-first wallet creation in three simple steps.
              No compromises, no complexity.
            </p>
          </motion.div>

          {/* Steps */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 md:gap-12">
            {steps.map((step, index) => (
              <motion.div
                key={step.number}
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: index * 0.15 }}
                className="relative"
              >
                {/* Connector Line */}
                {index < steps.length - 1 && (
                  <div className="hidden md:block absolute top-16 left-full w-full h-0.5 bg-gradient-to-r from-brand-primary/50 to-transparent -z-10" />
                )}

                {/* Step Card */}
                <div className="relative">
                  {/* Step Number */}
                  <div className="absolute -top-6 -left-4 text-6xl font-black text-brand-primary/10">
                    {step.number}
                  </div>

                  {/* Content */}
                  <div className="relative z-10 flex flex-col items-center text-center">
                    {/* Icon */}
                    <motion.div
                      whileHover={{ scale: 1.1, rotate: 5 }}
                      className="w-16 h-16 rounded-2xl bg-gradient-to-br from-brand-primary to-brand-secondary p-0.5 mb-6"
                    >
                      <div className="w-full h-full rounded-2xl bg-bg-primary flex items-center justify-center text-brand-primary">
                        {step.icon}
                      </div>
                    </motion.div>

                    {/* Title */}
                    <h3 className="text-2xl font-bold mb-4 text-text-primary">
                      {step.title}
                    </h3>

                    {/* Description */}
                    <p className="text-text-secondary leading-relaxed">
                      {step.description}
                    </p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>

          {/* Bottom CTA */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.5 }}
            className="mt-16 text-center"
          >
            <p className="text-text-secondary mb-4">
              Ready to experience true privacy?
            </p>
            <div className="inline-flex items-center gap-2 px-6 py-3 rounded-full border border-brand-primary/30 bg-brand-primary/5">
              <span className="w-2 h-2 bg-brand-primary rounded-full animate-pulse" />
              <span className="text-brand-primary font-medium">
                Under 10 seconds to get started
              </span>
            </div>
          </motion.div>
        </div>
      </Container>
    </section>
  );
}
