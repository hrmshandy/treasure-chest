import { useState, useCallback } from 'react';
import { Toast, ToastType } from '../components/ui/Toast';

export function useToast() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const showToast = useCallback((
    type: ToastType,
    title: string,
    options?: {
      message?: string;
      action?: { label: string; onClick: () => void };
      duration?: number;
    }
  ) => {
    const id = `toast-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    const toast: Toast = {
      id,
      type,
      title,
      message: options?.message,
      action: options?.action,
      duration: options?.duration ?? (type === 'error' ? 0 : 5000),
    };

    setToasts(prev => [...prev, toast]);
    return id;
  }, []);

  const dismissToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  const success = useCallback((title: string, message?: string) => {
    return showToast('success', title, { message });
  }, [showToast]);

  const error = useCallback((title: string, message?: string, action?: { label: string; onClick: () => void }) => {
    return showToast('error', title, { message, action, duration: 0 });
  }, [showToast]);

  const info = useCallback((title: string, message?: string) => {
    return showToast('info', title, { message });
  }, [showToast]);

  const download = useCallback((title: string, message?: string) => {
    return showToast('download', title, { message });
  }, [showToast]);

  return {
    toasts,
    showToast,
    dismissToast,
    success,
    error,
    info,
    download,
  };
}
