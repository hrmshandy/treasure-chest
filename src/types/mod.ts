export interface Mod {
    id: string;
    name: string;
    author: string;
    version: string;
    description?: string;
    uniqueId: string;
    dependencies?: string[];
    isEnabled: boolean;
    path: string;
    installDate: string;
    updateDate?: string;
    status: 'working' | 'update-available' | 'error' | 'disabled';
    endorsements?: number;
    nexusId?: number;
    nexusFileId?: number;
    downloadUrl?: string;
}
