import { cn } from '@/lib/utils';
import { InputHTMLAttributes, forwardRef } from 'react';

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, className, ...props }, ref) => {
    return (
      <div className="w-full">
        {label && (
          <label className="block text-sm font-medium text-text-secondary mb-2">
            {label}
          </label>
        )}
        <input
          ref={ref}
          className={cn(
            'w-full px-4 py-3 rounded-lg',
            'bg-bg-secondary border border-brand-primary/20',
            'text-text-primary placeholder:text-text-tertiary',
            'focus:outline-none focus:border-brand-primary focus:ring-2 focus:ring-brand-primary/20',
            'transition-all duration-200',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            error && 'border-brand-accent focus:border-brand-accent focus:ring-brand-accent/20',
            className
          )}
          {...props}
        />
        {error && (
          <p className="mt-2 text-sm text-brand-accent">{error}</p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
