import React from 'react';
import { Plus, Search } from 'lucide-react';

interface ToolbarProps {
    onAddMod: () => void;
    searchQuery: string;
    onSearchChange: (query: string) => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({ onAddMod, searchQuery, onSearchChange }) => {
    return (
        <div className="flex-none px-6 py-5 flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-stone-800/50 bg-stone-950 h-22">
            <div className="flex items-center">
                <button
                    onClick={onAddMod}
                    className="flex items-center gap-2 px-5 py-2 text-xs font-bold uppercase tracking-wider bg-stone-200 text-stone-950 border-2 border-stone-500 border-b-4 border-r-4 hover:bg-white active:border-b-2 active:border-r-2 active:translate-y-1 active:translate-x-1 transition-none"
                >
                    <Plus className="w-4 h-4" strokeWidth={3} />
                    ADD MOD
                </button>
            </div>

            <div className="flex items-center gap-4 flex-1 justify-end">
                {/* Filters */}
                <div className="hidden lg:flex p-0 gap-2">
                    <button className="px-3 py-1 text-xs font-bold uppercase bg-stone-800 text-stone-100 border-2 border-stone-600 shadow-[2px_2px_0_0_rgba(0,0,0,1)]">
                        All
                    </button>
                    <button className="px-3 py-1 text-xs font-bold uppercase text-stone-500 hover:text-stone-300 border-2 border-transparent hover:border-stone-800 hover:bg-stone-900">
                        Config
                    </button>
                    <button className="px-3 py-1 text-xs font-bold uppercase text-stone-500 hover:text-stone-300 border-2 border-transparent hover:border-stone-800 hover:bg-stone-900">
                        Enabled
                    </button>
                    <button className="px-3 py-1 text-xs font-bold uppercase text-stone-500 hover:text-stone-300 border-2 border-transparent hover:border-stone-800 hover:bg-stone-900">
                        Disabled
                    </button>
                </div>

                {/* Search */}
                <div className="relative group w-64">
                    <Search className="absolute left-3 top-2.5 w-4 h-4 text-stone-500 group-focus-within:text-orange-400 transition-colors" />
                    <input
                        type="text"
                        placeholder="SEARCH..."
                        value={searchQuery}
                        onChange={(e) => onSearchChange(e.target.value)}
                        className="w-full border-2 border-stone-700 bg-stone-950 text-stone-200 text-xs font-bold uppercase pl-9 pr-3 py-2 focus:outline-none focus:border-orange-500 placeholder:text-stone-700 transition-none shadow-inner"
                    />
                </div>
            </div>
        </div>
    );
};
