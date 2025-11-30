export type Theme = 'System' | 'Dark' | 'Light';
export type Language = 'English' | 'Bahasa Indonesia';
export type ModGroups = 'None' | 'Folder' | 'Pack';

export interface Settings {
  gamePath: string;
  smapiPath: string;
  nexusAuthCookie: string;
  nexusApiKey: string;
  theme: Theme;
  language: Language;
  modGroups: ModGroups;
  autoInstall: boolean;
  confirmBeforeInstall: boolean;
  deleteAfterInstall: boolean;
}

export const defaultSettings: Settings = {
  gamePath: '',
  smapiPath: '',
  nexusAuthCookie: '',
  nexusApiKey: '',
  theme: 'System',
  language: 'English',
  modGroups: 'Folder',
  autoInstall: true,
  confirmBeforeInstall: false,
  deleteAfterInstall: false,
};
