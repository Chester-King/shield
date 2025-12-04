'use client';

import { Container } from '../layout/Container';
import { Button } from '../ui/Button';
import { motion } from 'framer-motion';
import { ArrowRight, Sparkles } from 'lucide-react';

export function CTA() {
  return (
    <section className="relative py-20 md:py-28 overflow-hidden">
      <Container>
        <motion.div
          
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="relative max-w-4xl mx-auto"
        >
          {/* Main CTA Card */}
          <div className="relative p-12 md:p-16 rounded-3xl border border-brand-primary/30 bg-gradient-to-br from-bg-secondary/90 to-bg-tertiary/90 backdrop-blur-xl overflow-hidden">
            {/* Glow Effect */}
            <div className="absolute inset-0 bg-gradient-to-br from-brand-primary/10 via-transparent to-brand-secondary/10" />
            <div className="absolute -top-24 -right-24 w-48 h-48 bg-brand-primary/20 rounded-full blur-3xl" />
            <div className="absolute -bottom-24 -left-24 w-48 h-48 bg-brand-secondary/20 rounded-full blur-3xl" />

            {/* Content */}
            <div className="relative z-10 text-center">
              {/* Badge */}
              <motion.div
                
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: 0.2 }}
                className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-brand-primary/10 border border-brand-primary/20 mb-6"
              >
                <Sparkles className="w-4 h-4 text-brand-primary" />
                <span className="text-sm text-brand-primary font-medium">
                  Early Access Available
                </span>
              </motion.div>

              {/* Headline */}
              <motion.h2
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.3 }}
                className="text-3xl md:text-4xl lg:text-5xl font-black mb-6"
              >
                Ready to Build with{' '}
                <span className="text-gradient">Privacy-First</span> Wallets?
              </motion.h2>

              {/* Description */}
              <motion.p
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.4 }}
                className="text-lg md:text-xl text-text-secondary leading-relaxed mb-10 max-w-2xl mx-auto"
              >
                Join the waitlist and be among the first to integrate
                true privacy into your application. No seed phrases. No complexity.
                Just privacy.
              </motion.p>

              {/* CTA Buttons */}
              <motion.div
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.5 }}
                className="flex flex-col sm:flex-row gap-4 justify-center items-center"
              >
                <Button size="lg" className="group">
                  Join Waitlist
                  <ArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
                </Button>

                <Button variant="outline" size="lg">
                  View Documentation
                </Button>
              </motion.div>

              {/* Trust Line */}
              <motion.div
                
                whileInView={{ opacity: 1 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.6 }}
                className="mt-10 flex items-center justify-center gap-2 text-sm text-text-tertiary"
              >
                <ShieldCheckIcon />
                <span>Free for developers during beta</span>
                <span className="text-text-tertiary/50">•</span>
                <span>No credit card required</span>
              </motion.div>
            </div>
          </div>

          {/* Bottom Note */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.7 }}
            className="mt-8 text-center"
          >
            <p className="text-text-tertiary text-sm">
              Have questions?{' '}
              <a href="#" className="text-brand-primary hover:underline">
                Schedule a demo
              </a>{' '}
              or{' '}
              <a href="#" className="text-brand-primary hover:underline">
                join our Discord
              </a>
            </p>
          </motion.div>
        </motion.div>
      </Container>

      {/* Background Effects */}
      <div className="absolute inset-0 -z-10 overflow-hidden pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full h-full max-w-4xl">
          <div className="absolute top-0 left-1/4 w-64 h-64 bg-brand-primary/10 rounded-full blur-3xl animate-pulse-glow" />
          <div className="absolute bottom-0 right-1/4 w-64 h-64 bg-brand-secondary/10 rounded-full blur-3xl animate-pulse-glow" style={{ animationDelay: '1s' }} />
        </div>
      </div>
    </section>
  );
}

function ShieldCheckIcon() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      className="text-brand-primary"
    >
      <path
        d="M8 1L3 3V7C3 10.5 5.5 13.5 8 14.5C10.5 13.5 13 10.5 13 7V3L8 1Z"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M6 8L7.5 9.5L10.5 6.5"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
