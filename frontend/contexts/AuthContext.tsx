'use client';

import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { authAPI, User } from '@/lib/api/auth';

interface AuthContextType {
  user: User | null;
  loading: boolean;
  logout: () => Promise<void>;
  setTokens: (accessToken: string, refreshToken: string) => Promise<void>;
  isAuthenticated: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const TOKEN_KEY = 'shield_access_token';
const REFRESH_TOKEN_KEY = 'shield_refresh_token';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Check if user is already logged in
    const initAuth = async () => {
      const accessToken = localStorage.getItem(TOKEN_KEY);
      if (accessToken) {
        try {
          const userData = await authAPI.getMe(accessToken);
          setUser(userData);
        } catch (error) {
          // Token might be expired, try refreshing
          const refreshToken = localStorage.getItem(REFRESH_TOKEN_KEY);
          if (refreshToken) {
            try {
              const authResponse = await authAPI.refresh(refreshToken);
              localStorage.setItem(TOKEN_KEY, authResponse.access_token);
              localStorage.setItem(REFRESH_TOKEN_KEY, authResponse.refresh_token);
              setUser(authResponse.user);
            } catch (refreshError) {
              // Refresh failed, clear tokens
              localStorage.removeItem(TOKEN_KEY);
              localStorage.removeItem(REFRESH_TOKEN_KEY);
            }
          }
        }
      }
      setLoading(false);
    };

    initAuth();
  }, []);

  const logout = async () => {
    const refreshToken = localStorage.getItem(REFRESH_TOKEN_KEY);
    if (refreshToken) {
      try {
        await authAPI.logout(refreshToken);
      } catch (error) {
        console.error('Logout error:', error);
      }
    }
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    setUser(null);
  };

  const setTokens = useCallback(async (accessToken: string, refreshToken: string) => {
    localStorage.setItem(TOKEN_KEY, accessToken);
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
    // Fetch user data with the access token
    try {
      const userData = await authAPI.getMe(accessToken);
      setUser(userData);
    } catch (error) {
      console.error('Failed to fetch user data:', error);
      throw error;
    }
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user,
        loading,
        logout,
        setTokens,
        isAuthenticated: !!user,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
