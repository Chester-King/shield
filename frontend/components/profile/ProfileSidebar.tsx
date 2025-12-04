'use client';

import { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { User, Mail, Shield, X, Calendar } from 'lucide-react';

interface ProfileSidebarProps {
  isOpen: boolean;
  onClose: () => void;
  user: {
    email: string;
    full_name?: string | null;
    created_at: string;
  };
}

export function ProfileSidebar({ isOpen, onClose, user }: ProfileSidebarProps) {
  // Disable body scroll when sidebar is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = 'unset';
    }
    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isOpen]);

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 bg-black/80 backdrop-blur-md z-50"
            onClick={onClose}
          />

          {/* Sidebar */}
          <motion.div
            initial={{ x: '100%' }}
            animate={{ x: 0 }}
            exit={{ x: '100%' }}
            transition={{ type: 'spring', damping: 25, stiffness: 200 }}
            className="fixed right-0 top-0 h-full w-full sm:w-[480px] bg-bg-primary border-l border-brand-primary/20 shadow-2xl z-50 flex flex-col"
          >
            {/* Header */}
            <div className="flex items-center justify-between p-6 border-b border-brand-primary/20">
              <div>
                <h2 className="text-2xl font-bold">Profile</h2>
                <p className="text-sm text-text-secondary mt-1">Your account details</p>
              </div>
              <button
                onClick={onClose}
                className="w-10 h-10 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors flex items-center justify-center"
              >
                <X className="w-5 h-5 text-text-secondary" />
              </button>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto p-6 space-y-4">
              {/* Email */}
              <div className="flex items-center gap-3 p-4 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors">
                <div className="w-10 h-10 rounded-full bg-brand-primary/10 flex items-center justify-center flex-shrink-0">
                  <Mail className="w-5 h-5 text-brand-primary" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-sm text-text-tertiary mb-1">Email Address</div>
                  <div className="font-medium truncate">{user.email}</div>
                </div>
              </div>

              {/* Full Name */}
              {user.full_name && (
                <div className="flex items-center gap-3 p-4 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors">
                  <div className="w-10 h-10 rounded-full bg-brand-secondary/10 flex items-center justify-center flex-shrink-0">
                    <User className="w-5 h-5 text-brand-secondary" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-text-tertiary mb-1">Full Name</div>
                    <div className="font-medium truncate">{user.full_name}</div>
                  </div>
                </div>
              )}

              {/* Account Created */}
              <div className="flex items-center gap-3 p-4 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors">
                <div className="w-10 h-10 rounded-full bg-green-500/10 flex items-center justify-center flex-shrink-0">
                  <Calendar className="w-5 h-5 text-green-500" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-sm text-text-tertiary mb-1">Account Created</div>
                  <div className="font-medium">
                    {new Date(user.created_at).toLocaleDateString('en-US', {
                      year: 'numeric',
                      month: 'long',
                      day: 'numeric',
                    })}
                  </div>
                </div>
              </div>

              {/* Account ID */}
              <div className="flex items-center gap-3 p-4 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors">
                <div className="w-10 h-10 rounded-full bg-purple-500/10 flex items-center justify-center flex-shrink-0">
                  <Shield className="w-5 h-5 text-purple-500" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-sm text-text-tertiary mb-1">Account Status</div>
                  <div className="font-medium text-green-500">Active</div>
                </div>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
