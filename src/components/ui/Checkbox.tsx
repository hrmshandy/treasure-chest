import React from 'react';
import { Check, Minus } from 'lucide-react';
import clsx from 'clsx';

interface CheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onChange'> {
    label?: React.ReactNode;
    checked?: boolean;
    indeterminate?: boolean;
    onChange?: (checked: boolean) => void;
}

export const Checkbox: React.FC<CheckboxProps> = ({
    label,
    checked,
    indeterminate,
    onChange,
    className,
    disabled,
    ...props
}) => {
    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        onChange?.(e.target.checked);
    };

    return (
        <label className={clsx(
            "inline-flex items-center gap-2 cursor-pointer select-none group",
            disabled && "opacity-50 cursor-not-allowed",
            className
        )}>
            <div className="relative flex items-center">
                <input
                    type="checkbox"
                    className="peer sr-only"
                    checked={checked}
                    onChange={handleChange}
                    disabled={disabled}
                    ref={input => {
                        if (input) input.indeterminate = !!indeterminate;
                    }}
                    {...props}
                />
                <div className={clsx(
                    "w-4 h-4 rounded border transition-all duration-200 flex items-center justify-center",
                    "border-stone-600 bg-stone-800 group-hover:border-stone-500",
                    "peer-focus:ring-2 peer-focus:ring-orange-500/30 peer-focus:border-orange-500",
                    (checked || indeterminate)
                        ? "bg-orange-500 border-orange-500 text-white"
                        : "text-transparent"
                )}>
                    {indeterminate ? (
                        <Minus strokeWidth={3} className="w-3 h-3" />
                    ) : (
                        <Check strokeWidth={3} className="w-3 h-3" />
                    )}
                </div>
            </div>
            {label && (
                <span className="text-xs font-medium text-stone-300 group-hover:text-stone-200 transition-colors">
                    {label}
                </span>
            )}
        </label>
    );
};
