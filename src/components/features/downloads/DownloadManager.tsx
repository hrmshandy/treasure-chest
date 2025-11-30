import { useState } from 'react';
import { DownloadItem } from './DownloadItem';
import { useDownloads } from '../../../hooks/useDownloads';
import { X, FolderOpen, Trash2, DownloadCloud } from 'lucide-react';

interface DownloadManagerProps {
    isOpen: boolean;
    onClose: () => void;
    onModInstalled?: (uniqueId: string) => void;
    onToast?: (type: 'success' | 'error' | 'info' | 'download', title: string, message?: string) => void;
}

export function DownloadManager({ isOpen, onClose, onModInstalled, onToast }: DownloadManagerProps) {
    const {
        downloads,
        currentProgress,
        cancelDownload,
        clearCompleted,
        openDownloadFolder
    } = useDownloads({ onModInstalled, onToast });

    const [filter, setFilter] = useState<'all' | 'active' | 'completed' | 'failed'>('all');

    if (!isOpen) return null;

    const filteredDownloads = downloads.filter(d => {
        if (filter === 'all') return true;
        if (filter === 'active') return d.status === 'downloading' || d.status === 'queued';
        if (filter === 'completed') return d.status === 'completed';
        if (filter === 'failed') return typeof d.status === 'object' && 'failed' in d.status;
        return true;
    });

    // Sort: Active first, then by time (newest first - assuming array order is chronological)
    const sortedDownloads = [...filteredDownloads].reverse();

    return (
        <div className="fixed inset-y-0 right-0 w-96 bg-stone-950 border-l border-stone-800 shadow-2xl transform transition-transform duration-300 z-40 flex flex-col">
            {/* Header */}
            <div className="p-4 border-b border-stone-800 flex justify-between items-center bg-stone-900/50">
                <div className="flex items-center space-x-2">
                    <DownloadCloud size={20} className="text-indigo-400" />
                    <h2 className="font-semibold text-stone-200">Downloads</h2>
                    <span className="bg-stone-800 text-stone-400 text-xs px-2 py-0.5 rounded-full">
                        {downloads.length}
                    </span>
                </div>
                <button onClick={onClose} className="text-stone-500 hover:text-stone-300 p-1 rounded hover:bg-stone-800">
                    <X size={20} />
                </button>
            </div>

            {/* Toolbar */}
            <div className="p-2 border-b border-stone-800 flex space-x-2 overflow-x-auto">
                {(['all', 'active', 'completed', 'failed'] as const).map((f) => (
                    <button
                        key={f}
                        onClick={() => setFilter(f)}
                        className={`px-3 py-1 text-xs rounded-full capitalize whitespace-nowrap transition-colors ${filter === f
                                ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30'
                                : 'text-stone-500 hover:text-stone-300 hover:bg-stone-900'
                            }`}
                    >
                        {f}
                    </button>
                ))}
            </div>

            {/* List */}
            <div className="flex-1 overflow-y-auto p-4 space-y-2">
                {sortedDownloads.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-stone-600 space-y-2">
                        <DownloadCloud size={48} className="opacity-20" />
                        <p className="text-sm">No downloads found</p>
                    </div>
                ) : (
                    sortedDownloads.map(task => (
                        <DownloadItem
                            key={task.id}
                            task={task}
                            progress={currentProgress.get(task.id)}
                            onCancel={cancelDownload}
                        />
                    ))
                )}
            </div>

            {/* Footer */}
            <div className="p-4 border-t border-stone-800 bg-stone-900/30 space-y-2">
                <button
                    onClick={openDownloadFolder}
                    className="w-full flex items-center justify-center space-x-2 p-2 rounded bg-stone-800 hover:bg-stone-700 text-stone-300 text-sm transition-colors"
                >
                    <FolderOpen size={16} />
                    <span>Open Downloads Folder</span>
                </button>

                {downloads.some(d => d.status === 'completed') && (
                    <button
                        onClick={clearCompleted}
                        className="w-full flex items-center justify-center space-x-2 p-2 rounded border border-stone-800 hover:bg-stone-800 text-stone-500 hover:text-stone-300 text-sm transition-colors"
                    >
                        <Trash2 size={16} />
                        <span>Clear Completed</span>
                    </button>
                )}
            </div>
        </div>
    );
}
