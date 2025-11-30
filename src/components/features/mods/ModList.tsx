import React from 'react';
import { ThumbsUp, RefreshCw, Trash2, DownloadCloud } from 'lucide-react';
import { Mod } from '../../../types/mod';
import clsx from 'clsx';

interface ModListProps {
    mods: Mod[];
    onToggleMod: (id: string, enabled: boolean) => void;
    onUpdateMod: (id: string) => void;
    onDeleteMod: (id: string) => void;
    highlightedModId?: string | null;
}

export const ModList: React.FC<ModListProps> = ({ mods, onToggleMod, onUpdateMod, onDeleteMod, highlightedModId }) => {
    return (
        <div className="flex-1 overflow-auto relative bg-stone-950">
            <table className="w-full text-left border-collapse">
                <thead className="sticky top-0 z-10 box-border border-b bg-stone-950 border-stone-800 border-b-4">
                    <tr>
                        <th className="px-6 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider font-mono">Mod Name</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider text-center font-mono">Endorse</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider font-mono">Unique ID</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider font-mono">Author</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider font-mono">Installed</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider font-mono">Ver</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider font-mono">Status</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider text-center font-mono">Enable</th>
                        <th className="px-4 py-3 text-xs font-medium text-stone-500 uppercase tracking-wider text-right font-mono">Actions</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-stone-800/50">
                    {mods.map((mod) => {
                        const isNewlyInstalled = highlightedModId === mod.uniqueId;
                        return (
                        <tr
                            key={mod.id}
                            className={clsx(
                                "group transition-colors",
                                isNewlyInstalled
                                    ? "bg-green-900/20 border-2 border-green-500 animate-pulse"
                                    : "hover:bg-stone-900/40"
                            )}
                        >
                            <td className="px-6 py-4">
                                <div className="flex items-center gap-3">
                                    <div className={clsx(
                                        "w-8 h-8 rounded flex items-center justify-center text-[10px] font-bold shadow-lg font-mono flex-shrink-0",
                                        mod.status === 'working' ? "bg-gradient-to-br from-orange-500 to-red-600 text-white shadow-orange-500/20" : "bg-stone-800 border border-stone-700 text-stone-500"
                                    )}>
                                        {mod.name.substring(0, 2).toUpperCase()}
                                    </div>
                                    <span className="text-sm font-medium text-stone-200 font-mono">{mod.name}</span>
                                    {isNewlyInstalled && (
                                        <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-green-500/20 text-green-300 border border-green-500/30 animate-pulse">
                                            NEW
                                        </span>
                                    )}
                                    {mod.nexusId && (
                                        <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-orange-500/10 text-orange-400 border border-orange-500/20">
                                            NEXUS
                                        </span>
                                    )}
                                </div>
                            </td>
                            <td className="px-4 py-4 text-center">
                                <div className="inline-flex items-center gap-1 px-2 py-1 rounded border bg-stone-900 border-stone-800">
                                    <ThumbsUp className="w-3 h-3 text-stone-600" />
                                    <span className="text-xs text-stone-400 font-mono">{mod.endorsements ? `${(mod.endorsements / 1000).toFixed(1)}k` : '-'}</span>
                                </div>
                            </td>
                            <td className="px-4 py-4 text-xs text-stone-500 font-mono">{mod.uniqueId}</td>
                            <td className="px-4 py-4 text-sm text-stone-400 font-mono">{mod.author}</td>
                            <td className="px-4 py-4 text-xs text-stone-500 font-mono">{mod.installDate}</td>
                            <td className="px-4 py-4 text-xs text-stone-400 font-mono">
                                {mod.version}
                                {mod.status === 'update-available' && <span className="text-orange-500 ml-1" title="Update Available">↑</span>}
                            </td>
                            <td className="px-4 py-4">
                                <StatusBadge status={mod.status} />
                            </td>
                            <td className="px-4 py-4 text-center">
                                <div className="relative inline-block w-8 h-4 align-middle select-none transition duration-200 ease-in">
                                    <input
                                        id={`toggle-${mod.id}`}
                                        type="checkbox"
                                        checked={mod.isEnabled}
                                        onChange={(e) => onToggleMod(mod.id, e.target.checked)}
                                        className="toggle-checkbox absolute block w-4 h-4 rounded-full border-4 appearance-none cursor-pointer transition-all duration-300 bg-white border-stone-700" />
                                    <label
                                        htmlFor={`toggle-${mod.id}`}
                                        className="toggle-label block overflow-hidden h-4 rounded-full cursor-pointer bg-stone-700" />
                                </div>
                            </td>
                            <td className="px-4 py-4 text-right">
                                <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                    {mod.status === 'update-available' ? (
                                        <button onClick={() => onUpdateMod(mod.id)} className="p-1.5 rounded text-orange-500 transition-colors hover:bg-stone-800 hover:text-orange-400" title="Update Now">
                                            <DownloadCloud className="w-3.5 h-3.5" />
                                        </button>
                                    ) : (
                                        <button className="p-1.5 rounded transition-colors hover:bg-stone-800 text-stone-400 hover:text-orange-400" title="Check for updates">
                                            <RefreshCw className="w-3.5 h-3.5" />
                                        </button>
                                    )}
                                    {mod.nexusId && (
                                        <a
                                            href={`https://www.nexusmods.com/stardewvalley/mods/${mod.nexusId}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="p-1.5 rounded transition-colors hover:bg-stone-800 text-stone-400 hover:text-orange-400"
                                            title="View on Nexus"
                                        >
                                            <DownloadCloud className="w-3.5 h-3.5" />
                                        </a>
                                    )}
                                    <button onClick={() => onDeleteMod(mod.id)} className="p-1.5 rounded transition-colors hover:bg-stone-800 text-stone-400 hover:text-red-400" title="Delete">
                                        <Trash2 className="w-3.5 h-3.5" />
                                    </button>
                                </div>
                            </td>
                        </tr>
                        );
                    })}
                </tbody>
            </table>
        </div>
    );
};

const StatusBadge: React.FC<{ status: Mod['status'] }> = ({ status }) => {
    switch (status) {
        case 'working':
            return (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-pink-500/10 border border-pink-500/20 text-pink-400 font-mono">
                    Working
                </span>
            );
        case 'update-available':
            return (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-orange-500/10 border border-orange-500/20 text-orange-400 font-mono">
                    Update
                </span>
            );
        case 'error':
            return (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-500/10 border border-red-500/20 text-red-400 font-mono">
                    Error
                </span>
            );
        default:
            return (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-stone-500/10 border border-stone-500/20 text-stone-400 font-mono">
                    Disabled
                </span>
            );
    }
};
