import { cn } from '@/lib/utils';
import { ReactNode } from 'react';

interface GradientTextProps {
  children: ReactNode;
  className?: string;
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'p' | 'span';
}

export function GradientText({ children, className, as: Component = 'span' }: GradientTextProps) {
  return (
    <Component className={cn('text-gradient', className)}>
      {children}
    </Component>
  );
}
