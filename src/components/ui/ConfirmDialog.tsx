import React from 'react';
import { AlertTriangle } from 'lucide-react';

interface ConfirmDialogProps {
    isOpen: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
    variant?: 'danger' | 'warning' | 'info';
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
    isOpen,
    title,
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    onConfirm,
    onCancel,
    variant = 'warning',
}) => {
    if (!isOpen) return null;

    const variantStyles = {
        danger: 'border-red-500 bg-red-900/20',
        warning: 'border-orange-500 bg-orange-900/20',
        info: 'border-blue-500 bg-blue-900/20',
    };

    const buttonStyles = {
        danger: 'bg-red-600 hover:bg-red-700 border-red-500',
        warning: 'bg-orange-600 hover:bg-orange-700 border-orange-500',
        info: 'bg-blue-600 hover:bg-blue-700 border-blue-500',
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
            <div className={`max-w-md w-full mx-4 border-2 ${variantStyles[variant]} shadow-2xl`}>
                {/* Header */}
                <div className="px-6 py-4 border-b border-stone-800">
                    <div className="flex items-center gap-3">
                        <AlertTriangle className="w-6 h-6 text-orange-400" strokeWidth={2.5} />
                        <h2 className="text-lg font-bold text-stone-200 font-mono uppercase tracking-wider">
                            {title}
                        </h2>
                    </div>
                </div>

                {/* Content */}
                <div className="px-6 py-6">
                    <p className="text-sm text-stone-300 leading-relaxed whitespace-pre-line">
                        {message}
                    </p>
                </div>

                {/* Actions */}
                <div className="px-6 py-4 border-t border-stone-800 flex gap-3 justify-end">
                    <button
                        onClick={onCancel}
                        className="px-4 py-2 text-xs font-bold uppercase tracking-wider bg-stone-800 text-stone-300 border-2 border-stone-600 border-b-4 border-r-4 hover:bg-stone-700 active:border-b-2 active:border-r-2 active:translate-y-1 active:translate-x-1 transition-none"
                    >
                        {cancelLabel}
                    </button>
                    <button
                        onClick={onConfirm}
                        className={`px-4 py-2 text-xs font-bold uppercase tracking-wider text-white border-2 border-b-4 border-r-4 active:border-b-2 active:border-r-2 active:translate-y-1 active:translate-x-1 transition-none ${buttonStyles[variant]}`}
                    >
                        {confirmLabel}
                    </button>
                </div>
            </div>
        </div>
    );
};
