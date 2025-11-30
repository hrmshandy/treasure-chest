import { useEffect, useState } from 'react';
import { X, CheckCircle, AlertCircle, Info, Download, Copy, Check } from 'lucide-react';

export type ToastType = 'success' | 'error' | 'info' | 'download';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  duration?: number; // 0 = no auto-dismiss
}

interface ToastProps {
  toast: Toast;
  onDismiss: (id: string) => void;
}

export function ToastComponent({ toast, onDismiss }: ToastProps) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (toast.duration === 0) return;

    const timer = setTimeout(() => {
      onDismiss(toast.id);
    }, toast.duration || 5000);

    return () => clearTimeout(timer);
  }, [toast.id, toast.duration, onDismiss]);

  const handleCopyError = async () => {
    const errorText = `Error: ${toast.title}\n${toast.message || ''}`;
    try {
      await navigator.clipboard.writeText(errorText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy error:', err);
    }
  };

  const getIcon = () => {
    switch (toast.type) {
      case 'success':
        return <CheckCircle size={20} className="text-green-400" />;
      case 'error':
        return <AlertCircle size={20} className="text-red-400" />;
      case 'download':
        return <Download size={20} className="text-indigo-400" />;
      case 'info':
      default:
        return <Info size={20} className="text-blue-400" />;
    }
  };

  const getBorderColor = () => {
    switch (toast.type) {
      case 'success':
        return 'border-green-500';
      case 'error':
        return 'border-red-500';
      case 'download':
        return 'border-indigo-500';
      case 'info':
      default:
        return 'border-blue-500';
    }
  };

  const getBgColor = () => {
    switch (toast.type) {
      case 'success':
        return 'bg-green-900/90';
      case 'error':
        return 'bg-red-900/90';
      case 'download':
        return 'bg-indigo-900/90';
      case 'info':
      default:
        return 'bg-blue-900/90';
    }
  };

  return (
    <div
      className={`flex items-start space-x-3 p-4 ${getBgColor()} border-2 ${getBorderColor()} text-stone-100 shadow-lg rounded min-w-[320px] max-w-md animate-slide-in`}
    >
      <div className="flex-shrink-0 mt-0.5">{getIcon()}</div>

      <div className="flex-1 min-w-0">
        <p className="font-semibold text-sm">{toast.title}</p>
        {toast.message && (
          <p className="text-xs text-stone-300 mt-1 break-words">{toast.message}</p>
        )}
        <div className="flex items-center gap-2 mt-2">
          {toast.action && (
            <button
              onClick={() => {
                toast.action?.onClick();
                onDismiss(toast.id);
              }}
              className="text-xs underline hover:no-underline"
            >
              {toast.action.label}
            </button>
          )}
          {toast.type === 'error' && (
            <button
              onClick={handleCopyError}
              className="flex items-center gap-1 text-xs text-stone-400 hover:text-stone-200 transition-colors"
            >
              {copied ? (
                <>
                  <Check size={12} />
                  <span>Copied!</span>
                </>
              ) : (
                <>
                  <Copy size={12} />
                  <span>Copy Error</span>
                </>
              )}
            </button>
          )}
        </div>
      </div>

      <button
        onClick={() => onDismiss(toast.id)}
        className="flex-shrink-0 text-stone-400 hover:text-stone-200 transition-colors"
      >
        <X size={16} />
      </button>
    </div>
  );
}

interface ToastContainerProps {
  toasts: Toast[];
  onDismiss: (id: string) => void;
}

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  return (
    <div className="fixed bottom-4 right-4 z-50 space-y-2 pointer-events-none">
      {toasts.map((toast) => (
        <div key={toast.id} className="pointer-events-auto">
          <ToastComponent toast={toast} onDismiss={onDismiss} />
        </div>
      ))}
    </div>
  );
}
