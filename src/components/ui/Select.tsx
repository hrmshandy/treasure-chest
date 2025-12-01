import React from 'react';
import { ChevronDown } from 'lucide-react';
import clsx from 'clsx';

export interface SelectOption {
    label: string;
    value: string | number;
}

interface SelectProps extends Omit<React.SelectHTMLAttributes<HTMLSelectElement>, 'onChange'> {
    options: SelectOption[];
    onChange: (value: string) => void;
    label?: string;
}

export const Select: React.FC<SelectProps> = ({
    options,
    value,
    onChange,
    label,
    className,
    disabled,
    ...props
}) => {
    return (
        <div className={clsx("flex flex-col gap-1.5", className)}>
            {label && (
                <label className="text-xs font-medium text-stone-400 font-sans">
                    {label}
                </label>
            )}
            <div className="relative group">
                <select
                    value={value}
                    onChange={(e) => onChange(e.target.value)}
                    disabled={disabled}
                    className={clsx(
                        "w-full appearance-none bg-stone-900 border border-stone-700 text-stone-300 text-xs rounded px-3 py-2 pr-8",
                        "focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500/50",
                        "hover:border-stone-600 transition-colors font-sans",
                        disabled && "opacity-50 cursor-not-allowed bg-stone-950"
                    )}
                    {...props}
                >
                    {options.map((option) => (
                        <option key={option.value} value={option.value}>
                            {option.label}
                        </option>
                    ))}
                </select>
                <div className="absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none text-stone-500 group-hover:text-stone-400 transition-colors">
                    <ChevronDown className="w-4 h-4" />
                </div>
            </div>
        </div>
    );
};
