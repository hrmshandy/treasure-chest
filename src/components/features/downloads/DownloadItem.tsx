import { DownloadTask, DownloadProgress } from '../../../types/download';
import { XCircle, PauseCircle, AlertCircle, CheckCircle } from 'lucide-react';

interface DownloadItemProps {
    task: DownloadTask;
    progress?: DownloadProgress;
    onCancel: (id: string) => void;
    onRetry?: (id: string) => void; // TODO: Implement retry
}

export function DownloadItem({ task, progress, onCancel }: DownloadItemProps) {
    const isDownloading = task.status === 'downloading';
    const isCompleted = task.status === 'completed';
    const isFailed = typeof task.status === 'object' && 'failed' in task.status;
    const isQueued = task.status === 'queued';

    const formatBytes = (bytes: number) => {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };

    const formatSpeed = (bps: number) => {
        return `${formatBytes(bps)}/s`;
    };

    const formatTime = (seconds: number) => {
        if (seconds < 60) return `${seconds}s`;
        const minutes = Math.floor(seconds / 60);
        const remainingSeconds = seconds % 60;
        return `${minutes}m ${remainingSeconds}s`;
    };

    return (
        <div className="bg-stone-900/50 rounded-lg p-3 mb-2 border border-stone-800 hover:border-stone-700 transition-colors">
            <div className="flex justify-between items-start mb-2">
                <div className="flex-1 min-w-0 mr-4">
                    <h4 className="text-sm font-medium text-stone-200 truncate" title={task.fileName}>
                        {task.modName || task.fileName}
                    </h4>
                    <p className="text-xs text-stone-500 truncate">
                        {isCompleted ? 'Download completed' :
                            isFailed ? 'Download failed' :
                                isQueued ? 'Queued' :
                                    'Downloading...'}
                    </p>
                </div>
                <div className="flex items-center space-x-1">
                    {isDownloading && (
                        <button className="p-1 hover:bg-stone-800 rounded text-stone-400 hover:text-stone-200" title="Pause (Not implemented)">
                            <PauseCircle size={16} />
                        </button>
                    )}
                    {!isCompleted && !isFailed && (
                        <button
                            onClick={() => onCancel(task.id)}
                            className="p-1 hover:bg-stone-800 rounded text-stone-400 hover:text-red-400"
                            title="Cancel"
                        >
                            <XCircle size={16} />
                        </button>
                    )}
                </div>
            </div>

            {/* Progress Bar */}
            {!isCompleted && !isFailed && (
                <div className="w-full bg-stone-800 rounded-full h-1.5 mb-2 overflow-hidden">
                    <div
                        className={`h-full rounded-full transition-all duration-300 ${isQueued ? 'bg-stone-600 w-full animate-pulse' : 'bg-indigo-500'
                            }`}
                        style={{ width: isQueued ? '100%' : `${progress?.progressPercent || 0}%` }}
                    />
                </div>
            )}

            {/* Stats */}
            {isDownloading && progress && (
                <div className="flex justify-between text-xs text-stone-500 font-mono">
                    <span>{formatBytes(progress.bytesDownloaded)} / {progress.bytesTotal ? formatBytes(progress.bytesTotal) : '?'}</span>
                    <div className="flex space-x-3">
                        <span>{formatSpeed(progress.speedBps)}</span>
                        {progress.etaSeconds !== undefined && <span>ETA: {formatTime(progress.etaSeconds)}</span>}
                    </div>
                </div>
            )}

            {/* Error Message */}
            {isFailed && (
                <div className="text-xs text-red-400 flex items-center mt-1">
                    <AlertCircle size={12} className="mr-1" />
                    <span className="truncate">
                        {typeof task.status === 'object' && 'failed' in task.status ? task.status.failed.error : 'Unknown error'}
                    </span>
                </div>
            )}

            {/* Completed State */}
            {isCompleted && (
                <div className="text-xs text-green-400 flex items-center mt-1">
                    <CheckCircle size={12} className="mr-1" />
                    <span>Ready to install</span>
                </div>
            )}
        </div>
    );
}
