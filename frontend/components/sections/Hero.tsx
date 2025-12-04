'use client';

import { Container } from '../layout/Container';
import { Button } from '../ui/Button';
import { GradientText } from '../ui/GradientText';
import { AnimatedShield } from '../ui/AnimatedShield';
import { ArrowRight, Play } from 'lucide-react';

export function Hero() {
  return (
    <section className="relative min-h-screen flex items-center justify-center pt-20 pb-16 md:pt-24 md:pb-20 overflow-hidden">
      <Container>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-16 items-center">
          {/* Left: Text Content */}
          <div className="relative z-10 ">
            {/* Badge */}
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-brand-primary/10 border border-brand-primary/20 mb-6">
              <span className="w-2 h-2 bg-brand-primary rounded-full animate-pulse" />
              <span className="text-sm text-brand-primary font-medium">
                First Embedded Wallet with True Privacy
              </span>
            </div>

            {/* Headline */}
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-black leading-tight mb-6">
              <GradientText>Privacy-First</GradientText>
              <br />
              Wallet Infrastructure
              <br />
              <span className="text-text-secondary">for Zcash</span>
            </h1>

            {/* Subheadline */}
            <p className="text-lg md:text-xl text-text-secondary leading-relaxed mb-8 max-w-xl">
              Shielded transactions as simple as email login.{' '}
              <span className="text-brand-primary">Zero-knowledge privacy</span>{' '}
              without the complexity.
            </p>

            {/* CTAs */}
            <div className="flex flex-col sm:flex-row gap-4">
              <Button size="lg" className="group">
                Get Started
                <ArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </Button>

              <Button variant="outline" size="lg" className="group">
                <Play className="mr-2 w-5 h-5" />
                View Demo
              </Button>
            </div>

            {/* Stats */}
            <div className="mt-12 grid grid-cols-3 gap-6 md:gap-8">
              <div>
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">
                  100%
                </div>
                <div className="text-sm text-text-tertiary">Private</div>
              </div>
              <div>
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">
                  10s
                </div>
                <div className="text-sm text-text-tertiary">To Onboard</div>
              </div>
              <div>
                <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">
                  0
                </div>
                <div className="text-sm text-text-tertiary">Seed Phrases</div>
              </div>
            </div>
          </div>

          {/* Right: Animated Visual */}
          <div className="relative h-[400px] md:h-[500px] lg:h-[600px]">
            <AnimatedShield />
          </div>
        </div>
      </Container>

      {/* Background decoration */}
      <div className="absolute inset-0 -z-10 overflow-hidden">
        <div className="absolute top-1/4 -right-32 w-96 h-96 bg-brand-primary/10 rounded-full blur-3xl" />
        <div className="absolute bottom-1/4 -left-32 w-96 h-96 bg-brand-secondary/10 rounded-full blur-3xl" />
      </div>
    </section>
  );
}
