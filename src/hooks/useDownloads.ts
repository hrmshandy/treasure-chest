import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { DownloadTask, DownloadProgress, DownloadFailure } from '../types/download';
import { NxmUrl } from '../types/nxm';

interface UseDownloadsReturn {
    downloads: DownloadTask[];
    currentProgress: Map<string, DownloadProgress>;
    activeCount: number;
    queuedCount: number;
    cancelDownload: (id: string) => Promise<void>;
    clearCompleted: () => Promise<void>;
    openDownloadFolder: () => Promise<void>;
}

interface UseDownloadsOptions {
    onModInstalled?: (uniqueId: string) => void;
    onToast?: (type: 'success' | 'error' | 'info' | 'download', title: string, message?: string) => void;
}

export function useDownloads(options?: UseDownloadsOptions): UseDownloadsReturn {
    const [downloads, setDownloads] = useState<DownloadTask[]>([]);
    const [currentProgress, setCurrentProgress] = useState<Map<string, DownloadProgress>>(new Map());

    // Load initial state
    useEffect(() => {
        invoke<DownloadTask[]>('get_downloads')
            .then(setDownloads)
            .catch(console.error);
    }, []);

    useEffect(() => {
        // Listen for NXM protocol events
        const unlistenNxm = listen<NxmUrl>('nxm-url-received', (event) => {
            console.log('NXM URL received:', event.payload);
            if (options?.onToast) {
                options.onToast('info', 'NXM Link Received', `Processing mod ${event.payload.mod_id}`);
            }
            // Backend handles queuing, we just wait for download-queued event
        });

        const unlistenDebug = listen<string>('debug-deep-link', (event) => {
            console.log('Raw deep link received:', event.payload);
            // Only for debugging, skip toast
        });

        const unlistenError = listen<string>('nxm-error', (event) => {
            console.error('NXM error:', event.payload);
            if (options?.onToast) {
                options.onToast('error', 'NXM Protocol Error', event.payload);
            }
        });

        // Listen for download events
        const unlistenQueued = listen<DownloadTask>('download-queued', (event) => {
            setDownloads(prev => {
                // Prevent duplicates if backend sends multiple events
                if (prev.some(d => d.id === event.payload.id)) return prev;
                return [...prev, event.payload];
            });

            if (options?.onToast) {
                options.onToast('download', 'Download Queued', event.payload.fileName);
            }
        });

        const unlistenProgress = listen<DownloadProgress>('download-progress', (event) => {
            const progress = event.payload;
            setCurrentProgress(prev => new Map(prev).set(progress.downloadId, progress));

            // Update status in list if needed (e.g. from queued to downloading)
            setDownloads(prev => prev.map(d =>
                d.id === progress.downloadId && d.status === 'queued'
                    ? { ...d, status: 'downloading' }
                    : d
            ));
        });

        const unlistenCompleted = listen<string>('download-completed', (event) => {
            const downloadId = event.payload;
            setDownloads(prev => prev.map(d => {
                if (d.id === downloadId) {
                    // Show toast for completed download
                    if (options?.onToast) {
                        options.onToast('success', 'Download Complete', d.fileName);
                    }
                    return { ...d, status: 'completed' };
                }
                return d;
            }));
            setCurrentProgress(prev => {
                const newMap = new Map(prev);
                newMap.delete(downloadId);
                return newMap;
            });
        });

        const unlistenFailed = listen<DownloadFailure>('download-failed', (event) => {
            const { downloadId, error } = event.payload;
            setDownloads(prev => prev.map(d =>
                d.id === downloadId ? { ...d, status: { failed: { error } } } : d
            ));
            setCurrentProgress(prev => {
                const newMap = new Map(prev);
                newMap.delete(downloadId);
                return newMap;
            });

            if (options?.onToast) {
                options.onToast('error', 'Download Failed', error);
            }
        });

        const unlistenConfirmation = listen<string>('install-confirmation-needed', (event) => {
            const downloadId = event.payload;
            // Find download and maybe update status or show modal
            // For now, we'll just log it, as the UI for confirmation isn't fully designed yet
            console.log("Confirmation needed for:", downloadId);
            // TODO: Trigger confirmation UI
        });

        // Listen for mod installation events
        const unlistenModInstalled = listen<{ mod_name: string; version: string; unique_id: string }>('mod-installed', (event) => {
            const { mod_name, version, unique_id } = event.payload;
            console.log('Mod installed:', mod_name, version, unique_id);

            // Show success notification
            if (options?.onToast) {
                options.onToast('success', 'Mod Installed', `${mod_name} v${version}`);
            }

            // Trigger callback to refresh mod list with unique_id
            if (options?.onModInstalled) {
                options.onModInstalled(unique_id);
            }
        });

        const unlistenModInstallFailed = listen<string>('mod-install-failed', (event) => {
            const error = event.payload;
            console.error('Mod installation failed:', error);

            // Show error notification
            if (options?.onToast) {
                options.onToast('error', 'Installation Failed', error);
            }
        });

        return () => {
            unlistenNxm.then(fn => fn());
            unlistenDebug.then(fn => fn());
            unlistenError.then(fn => fn());
            unlistenQueued.then(fn => fn());
            unlistenProgress.then(fn => fn());
            unlistenCompleted.then(fn => fn());
            unlistenFailed.then(fn => fn());
            unlistenConfirmation.then(fn => fn());
            unlistenModInstalled.then(fn => fn());
            unlistenModInstallFailed.then(fn => fn());
        };
    }, [options]);

    const cancelDownload = useCallback(async (id: string) => {
        try {
            await invoke('cancel_download', { downloadId: id });
        } catch (error) {
            console.error('Failed to cancel download:', error);
        }
    }, []);

    const clearCompleted = useCallback(async () => {
        try {
            await invoke('clear_completed_downloads');
            setDownloads(prev => prev.filter(d => d.status !== 'completed'));
        } catch (error) {
            console.error('Failed to clear completed downloads:', error);
        }
    }, []);

    const openDownloadFolder = useCallback(async () => {
        try {
            await invoke('open_downloads_folder');
        } catch (error) {
            console.error('Failed to open downloads folder:', error);
        }
    }, []);

    const activeCount = downloads.filter(d => d.status === 'downloading').length;
    const queuedCount = downloads.filter(d => d.status === 'queued').length;

    return {
        downloads,
        currentProgress,
        activeCount,
        queuedCount,
        cancelDownload,
        clearCompleted,
        openDownloadFolder
    };
}
