import React from 'react';
import { Plus, Search, FolderOpen, RefreshCw } from 'lucide-react';

interface ToolbarProps {
    onAddMod: () => void;
    onOpenMods: () => void;
    onRefresh: () => void;
    searchQuery: string;
    onSearchChange: (query: string) => void;
    filterStatus: 'all' | 'enabled' | 'disabled' | 'updates' | 'config';
    onFilterChange: (status: 'all' | 'enabled' | 'disabled' | 'updates' | 'config') => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
    onAddMod,
    onOpenMods,
    onRefresh,
    searchQuery,
    onSearchChange,
    filterStatus,
    onFilterChange
}) => {
    return (
        <div className="flex-none px-6 py-5 flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-stone-800/50 bg-stone-950 h-22">
            <div className="flex items-center gap-2">
                <button
                    onClick={onAddMod}
                    className="flex items-center gap-2 px-5 py-2 text-xs font-bold uppercase tracking-wider bg-stone-200 text-stone-950 border-2 border-stone-500 border-b-4 border-r-4 hover:bg-white active:border-b-2 active:border-r-2 active:translate-y-1 active:translate-x-1 transition-none"
                >
                    <Plus className="w-4 h-4" strokeWidth={3} />
                    ADD MOD
                </button>
                <button
                    onClick={onOpenMods}
                    className="flex items-center gap-2 px-5 py-2 text-xs font-bold uppercase tracking-wider bg-stone-800 text-stone-200 border-2 border-stone-600 border-b-4 border-r-4 hover:bg-stone-700 active:border-b-2 active:border-r-2 active:translate-y-1 active:translate-x-1 transition-none"
                >
                    <FolderOpen className="w-4 h-4" strokeWidth={3} />
                    OPEN MODS DIR
                </button>
                <button
                    onClick={onRefresh}
                    className="flex items-center gap-2 px-4 py-2 text-xs font-bold uppercase tracking-wider bg-stone-800 text-stone-200 border-2 border-stone-600 border-b-4 border-r-4 hover:bg-stone-700 active:border-b-2 active:border-r-2 active:translate-y-1 active:translate-x-1 transition-none"
                    title="Refresh mod list"
                >
                    <RefreshCw className="w-4 h-4" strokeWidth={3} />
                </button>
            </div>

            <div className="flex items-center gap-4 flex-1 justify-end">
                {/* Filters */}
                <div className="hidden lg:flex p-0 gap-2">
                    <FilterButton
                        label="All"
                        active={filterStatus === 'all'}
                        onClick={() => onFilterChange('all')}
                    />
                    <FilterButton
                        label="Config"
                        active={filterStatus === 'config'}
                        onClick={() => onFilterChange('config')}
                    />
                    <FilterButton
                        label="Enabled"
                        active={filterStatus === 'enabled'}
                        onClick={() => onFilterChange('enabled')}
                    />
                    <FilterButton
                        label="Disabled"
                        active={filterStatus === 'disabled'}
                        onClick={() => onFilterChange('disabled')}
                    />
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

const FilterButton: React.FC<{ label: string; active: boolean; onClick: () => void }> = ({ label, active, onClick }) => (
    <button
        onClick={onClick}
        className={`px-3 py-1 text-xs font-bold uppercase border-2 transition-none ${active
            ? 'bg-stone-800 text-stone-100 border-stone-600 shadow-[2px_2px_0_0_rgba(0,0,0,1)]'
            : 'text-stone-500 hover:text-stone-300 border-transparent hover:border-stone-800 hover:bg-stone-900'
            }`}
    >
        {label}
    </button>
);
