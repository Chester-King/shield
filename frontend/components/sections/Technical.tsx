'use client';

import { Container } from '../layout/Container';
import { Card } from '../ui/Card';
import { GradientText } from '../ui/GradientText';
import { motion } from 'framer-motion';
import { Terminal, Boxes, Network } from 'lucide-react';

export function Technical() {
  const techHighlights = [
    {
      icon: <Terminal className="w-6 h-6" />,
      title: 'Orchard Shielded Pool',
      description: 'Leverages Zcash\'s latest privacy protocol with recursive zero-knowledge proofs.',
      specs: ['Halo 2 proving system', 'No trusted setup', 'Constant-size proofs'],
    },
    {
      icon: <Boxes className="w-6 h-6" />,
      title: 'Unified Addresses',
      description: 'Support for multi-protocol addresses enabling seamless cross-pool transactions.',
      specs: ['Orchard + Sapling', 'Transparent fallback', 'Forward compatible'],
    },
    {
      icon: <Network className="w-6 h-6" />,
      title: 'MPC Key Management',
      description: 'Distributed key generation and threshold signatures for enhanced security.',
      specs: ['No single point of failure', '2-of-3 threshold', 'Client-side encryption'],
    },
  ];

  return (
    <section className="relative py-20 md:py-28 overflow-hidden">
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
              <GradientText>Technical</GradientText> Excellence
            </h2>
            <p className="text-lg md:text-xl text-text-secondary leading-relaxed max-w-2xl mx-auto">
              Built on cutting-edge cryptography and battle-tested infrastructure.
            </p>
          </motion.div>

          {/* Tech Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 md:gap-8 mb-16">
            {techHighlights.map((tech, index) => (
              <motion.div
                key={tech.title}
                
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
              >
                <Card className="h-full">
                  <div className="flex flex-col gap-4">
                    <div className="w-12 h-12 rounded-lg bg-brand-primary/10 flex items-center justify-center text-brand-primary">
                      {tech.icon}
                    </div>
                    <h3 className="text-xl font-semibold text-text-primary">
                      {tech.title}
                    </h3>
                    <p className="text-text-secondary leading-relaxed">
                      {tech.description}
                    </p>
                    <ul className="space-y-2 mt-2">
                      {tech.specs.map((spec) => (
                        <li key={spec} className="flex items-center gap-2 text-sm text-text-tertiary">
                          <span className="w-1 h-1 bg-brand-primary rounded-full" />
                          {spec}
                        </li>
                      ))}
                    </ul>
                  </div>
                </Card>
              </motion.div>
            ))}
          </div>

          {/* Code Example */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.4 }}
            className="relative"
          >
            <Card className="bg-bg-tertiary/50">
              <div className="flex items-center gap-2 mb-4 pb-4 border-b border-brand-primary/10">
                <div className="w-3 h-3 rounded-full bg-brand-accent" />
                <div className="w-3 h-3 rounded-full bg-brand-secondary" />
                <div className="w-3 h-3 rounded-full bg-brand-primary" />
                <span className="ml-2 text-sm text-text-tertiary font-mono">Quick Start</span>
              </div>
              <div className="space-y-2 font-mono text-sm">
                <div className="text-text-tertiary">
                  <span className="text-brand-secondary">import</span>{' '}
                  <span className="text-brand-primary">{'{ ShieldWallet }'}</span>{' '}
                  <span className="text-brand-secondary">from</span>{' '}
                  <span className="text-text-primary">'@shield/sdk'</span>;
                </div>
                <div className="h-px" />
                <div className="text-text-tertiary">
                  <span className="text-brand-secondary">const</span>{' '}
                  <span className="text-text-primary">wallet</span>{' '}
                  <span className="text-brand-secondary">=</span>{' '}
                  <span className="text-brand-secondary">new</span>{' '}
                  <span className="text-brand-primary">ShieldWallet</span>
                  <span className="text-text-primary">{'({'}</span>
                </div>
                <div className="text-text-tertiary pl-4">
                  <span className="text-text-primary">appId:</span>{' '}
                  <span className="text-brand-primary">'your-app-id'</span>,
                </div>
                <div className="text-text-tertiary">
                  <span className="text-text-primary">{'});'}</span>
                </div>
                <div className="h-px" />
                <div className="text-text-tertiary">
                  <span className="text-brand-secondary">await</span>{' '}
                  <span className="text-text-primary">wallet</span>
                  <span className="text-brand-secondary">.</span>
                  <span className="text-brand-primary">login</span>
                  <span className="text-text-primary">();</span>
                  <span className="text-text-tertiary ml-6">// That's it!</span>
                </div>
              </div>
            </Card>
          </motion.div>

          {/* Stats Row */}
          <motion.div
            
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.6 }}
            className="mt-16 grid grid-cols-2 md:grid-cols-4 gap-6"
          >
            {[
              { value: '< 2KB', label: 'SDK Size' },
              { value: '< 500ms', label: 'Login Time' },
              { value: '99.9%', label: 'Uptime' },
              { value: 'Open', label: 'Source' },
            ].map((stat, index) => (
              <div key={stat.label} className="text-center">
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-2">
                  {stat.value}
                </div>
                <div className="text-sm text-text-tertiary">{stat.label}</div>
              </div>
            ))}
          </motion.div>
        </div>
      </Container>

      {/* Background grid */}
      <div className="absolute inset-0 -z-10 opacity-20">
        <div className="absolute inset-0" style={{
          backgroundImage: 'linear-gradient(rgba(0, 255, 204, 0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(0, 255, 204, 0.1) 1px, transparent 1px)',
          backgroundSize: '50px 50px',
        }} />
      </div>
    </section>
  );
}
