import { NxmUrl } from './nxm';

export type DownloadStatus =
  | 'queued'
  | 'downloading'
  | 'paused'
  | 'completed'
  | { failed: { error: string } };

export interface DownloadTask {
  id: string;
  nxmUrl: NxmUrl;
  modName?: string;
  fileName: string;
  status: DownloadStatus;
  filePath?: string;
  bytesDownloaded: number;
  bytesTotal?: number;
}

export interface DownloadProgress {
  downloadId: string;
  bytesDownloaded: number;
  bytesTotal?: number;
  speedBps: number;
  etaSeconds?: number;
  progressPercent: number;
}

export interface DownloadFailure {
  downloadId: string;
  error: string;
}
