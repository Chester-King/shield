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
