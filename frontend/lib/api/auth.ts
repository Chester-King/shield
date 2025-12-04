export interface User {
  id: string;
  email: string;
  full_name: string | null;
  email_verified: boolean;
  created_at: string;
  wallet_address?: string | null;
  solana_address?: string | null;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
}

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000/api';

class AuthAPI {
  private async fetch<T>(url: string, options?: RequestInit): Promise<T> {
    const response = await fetch(`${API_URL}${url}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'An error occurred' }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  async signup(email: string, password: string, full_name?: string): Promise<AuthResponse> {
    return this.fetch<AuthResponse>('/auth/signup', {
      method: 'POST',
      body: JSON.stringify({ email, password, full_name }),
    });
  }

  async login(email: string, password: string): Promise<AuthResponse> {
    return this.fetch<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
  }

  async refresh(refreshToken: string): Promise<AuthResponse> {
    return this.fetch<AuthResponse>('/auth/refresh', {
      method: 'POST',
      body: JSON.stringify(refreshToken),
    });
  }

  async logout(refreshToken: string): Promise<void> {
    await this.fetch('/auth/logout', {
      method: 'POST',
      body: JSON.stringify(refreshToken),
    });
  }

  async getMe(accessToken: string): Promise<User> {
    return this.fetch<User>('/users/me', {
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    });
  }
}

export const authAPI = new AuthAPI();
