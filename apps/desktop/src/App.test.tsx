import { render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockRejectedValue(new Error('no tauri')) }));
import { App } from './App';
it('renders dashboard loading shell', async () => { render(<App />); expect(screen.getByText('Shadow Council Studio')).toBeInTheDocument(); expect(await screen.findByText('Dashboard')).toBeInTheDocument(); });
