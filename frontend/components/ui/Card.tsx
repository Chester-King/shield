import { cn } from '@/lib/utils';
import { ReactNode } from 'react';

interface CardProps {
  children: ReactNode;
  className?: string;
  hover?: boolean;
}

export function Card({ children, className, hover = true }: CardProps) {
  return (
    <div
      className={cn(
        'glass-card p-6 md:p-8',
        hover && 'transition-all duration-300 hover:border-brand-primary/50 hover:shadow-glow hover:scale-[1.02]',
        className
      )}
    >
      {children}
    </div>
  );
}

interface FeatureCardProps {
  icon: ReactNode;
  title: string;
  description: string;
  className?: string;
}

export function FeatureCard({ icon, title, description, className }: FeatureCardProps) {
  return (
    <Card className={className}>
      <div className="flex flex-col gap-4">
        <div className="w-12 h-12 rounded-lg bg-brand-primary/10 flex items-center justify-center text-brand-primary">
          {icon}
        </div>
        <h3 className="text-xl font-semibold text-text-primary">{title}</h3>
        <p className="text-text-secondary leading-relaxed">{description}</p>
      </div>
    </Card>
  );
}
