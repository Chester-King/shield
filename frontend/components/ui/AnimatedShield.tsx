'use client';

import { motion } from 'framer-motion';
import { Shield } from 'lucide-react';

export function AnimatedShield() {
  return (
    <div className="relative w-full h-full flex items-center justify-center">
      {/* Outer glow rings */}
      <motion.div
        className="absolute w-64 h-64 md:w-96 md:h-96 rounded-full border-2 border-brand-primary/20"
        animate={{
          scale: [1, 1.2, 1],
          opacity: [0.3, 0.1, 0.3],
        }}
        transition={{
          duration: 3,
          repeat: Infinity,
          ease: 'easeInOut',
        }}
      />

      <motion.div
        className="absolute w-48 h-48 md:w-72 md:h-72 rounded-full border-2 border-brand-secondary/20"
        animate={{
          scale: [1, 1.15, 1],
          opacity: [0.4, 0.2, 0.4],
        }}
        transition={{
          duration: 2.5,
          repeat: Infinity,
          ease: 'easeInOut',
          delay: 0.5,
        }}
      />

      {/* Center shield icon */}
      <motion.div
        className="relative z-10"
        animate={{
          y: [0, -10, 0],
        }}
        transition={{
          duration: 3,
          repeat: Infinity,
          ease: 'easeInOut',
        }}
      >
        <div className="relative">
          <Shield className="w-32 h-32 md:w-40 md:h-40 text-brand-primary drop-shadow-glow" />

          {/* Glow effect */}
          <motion.div
            className="absolute inset-0 bg-brand-primary blur-2xl"
            animate={{
              opacity: [0.3, 0.6, 0.3],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              ease: 'easeInOut',
            }}
          />
        </div>
      </motion.div>

      {/* Particle effects */}
      {[...Array(6)].map((_, i) => (
        <motion.div
          key={i}
          className="absolute w-2 h-2 bg-brand-primary rounded-full"
          style={{
            left: '50%',
            top: '50%',
          }}
          animate={{
            x: [0, (Math.cos((i * Math.PI * 2) / 6) * 120)],
            y: [0, (Math.sin((i * Math.PI * 2) / 6) * 120)],
            opacity: [0, 1, 0],
            scale: [0, 1, 0],
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: 'easeOut',
            delay: i * 0.2,
          }}
        />
      ))}
    </div>
  );
}
