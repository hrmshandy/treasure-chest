import React, { useState, useEffect } from 'react';
import { X, FolderOpen } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { Settings, Theme, Language, ModGroups } from '../../../types/settings';
import { Checkbox } from '../../ui/Checkbox';
import { Select } from '../../ui/Select';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  settings: Settings;
  onSave: (settings: Settings) => void;
  onAutoDetect: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  settings: initialSettings,
  onSave,
}) => {
  const [settings, setSettings] = useState<Settings>(initialSettings);
  const [errors, setErrors] = useState<{ [key: string]: string }>({});

  useEffect(() => {
    setSettings(initialSettings);
  }, [initialSettings, isOpen]);

  const handleGamePathPick = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Stardew Valley Game Directory',
      });

      if (selected) {
        const path = Array.isArray(selected) ? selected[0] : selected;
        const isValid = await invoke<boolean>('validate_game_path_cmd', { path });

        if (isValid) {
          setSettings({ ...settings, gamePath: path });
          setErrors({ ...errors, gamePath: '' });

          // Auto-detect SMAPI path when game path changes
          try {
            const [, smapiPath] = await invoke<[string | null, string | null]>('auto_detect_paths');
            if (smapiPath) {
              setSettings(prev => ({ ...prev, smapiPath }));
            }
          } catch (error) {
            console.error('Failed to auto-detect SMAPI:', error);
          }
        } else {
          setErrors({ ...errors, gamePath: 'Invalid game directory. Please select the Stardew Valley installation folder.' });
        }
      }
    } catch (error) {
      console.error('Failed to pick game path:', error);
    }
  };

  const handleSmapiPathPick = async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: 'Select SMAPI Executable',
        filters: [
          { name: 'Executable', extensions: ['exe', 'sh', ''] }
        ],
      });

      if (selected) {
        const path = Array.isArray(selected) ? selected[0] : selected;
        const isValid = await invoke<boolean>('validate_smapi_path_cmd', { path });

        if (isValid) {
          setSettings({ ...settings, smapiPath: path });
          setErrors({ ...errors, smapiPath: '' });
        } else {
          setErrors({ ...errors, smapiPath: 'Invalid SMAPI executable.' });
        }
      }
    } catch (error) {
      console.error('Failed to pick SMAPI path:', error);
    }
  };

  const handleSave = () => {
    // Validate required fields
    const newErrors: { [key: string]: string } = {};

    if (!settings.gamePath) {
      newErrors.gamePath = 'Game path is required';
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    onSave(settings);
    onClose();
  };

  const handleBackdropClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50" role="dialog" aria-modal="true">
      <div
        className="fixed inset-0 transition-opacity bg-black/60"
        onClick={handleBackdropClick}
      />

      <div className="fixed inset-y-0 right-0 w-full max-w-md border-l shadow-2xl transform transition-transform duration-300 flex flex-col bg-stone-950 border-stone-800">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-5 border-b border-stone-800">
          <h2 className="text-lg font-medium tracking-tight text-stone-100 font-sans">
            Settings
          </h2>
          <button
            onClick={onClose}
            className="text-stone-500 transition-colors hover:text-stone-200"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-8">
          {/* Game Path Section */}
          <div className="space-y-4">
            <div>
              <h3 className="text-sm font-medium text-stone-100 font-sans">
                Game Path
              </h3>
              <p className="text-xs text-stone-500 mt-1 font-sans">
                Set the file paths for your game and SMAPI
              </p>
            </div>

            <div className="space-y-4">
              {/* Game Path */}
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-stone-400 font-sans">
                  Game Directory
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={settings.gamePath}
                    onChange={(e) => setSettings({ ...settings, gamePath: e.target.value })}
                    className="flex-1 border text-xs px-3 py-2 focus:outline-none focus:border-orange-500/50 transition-colors font-mono bg-stone-900 border-stone-800 text-stone-300"
                    placeholder="C:\Program Files (x86)\Steam\steamapps\common\Stardew Valley"
                  />
                  <button
                    onClick={handleGamePathPick}
                    className="px-3 py-2 border transition-colors bg-stone-900 border-stone-800 hover:bg-stone-800 text-stone-400"
                  >
                    <FolderOpen className="w-4 h-4" />
                  </button>
                </div>
                {errors.gamePath && (
                  <p className="text-xs text-red-400 mt-1">{errors.gamePath}</p>
                )}
              </div>

              {/* SMAPI Path */}
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-stone-400 font-sans">
                  SMAPI Path
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={settings.smapiPath}
                    onChange={(e) => setSettings({ ...settings, smapiPath: e.target.value })}
                    className="flex-1 border text-xs px-3 py-2 focus:outline-none focus:border-orange-500/50 transition-colors font-mono bg-stone-900 border-stone-800 text-stone-300"
                    placeholder="StardewModdingAPI.exe"
                  />
                  <button
                    onClick={handleSmapiPathPick}
                    className="px-3 py-2 border transition-colors bg-stone-900 border-stone-800 hover:bg-stone-800 text-stone-400"
                  >
                    <FolderOpen className="w-4 h-4" />
                  </button>
                </div>
                {errors.smapiPath && (
                  <p className="text-xs text-red-400 mt-1">{errors.smapiPath}</p>
                )}
              </div>

              {/* Nexus Auth Cookie */}
              {/* <div className="space-y-1.5">
                <label className="text-xs font-medium text-stone-400 font-sans">
                  Nexus Auth Cookie <span className="text-stone-600">(optional)</span>
                </label>
                <input
                  type="text"
                  value={settings.nexusAuthCookie}
                  onChange={(e) => setSettings({ ...settings, nexusAuthCookie: e.target.value })}
                  className="w-full border text-xs px-3 py-2 focus:outline-none focus:border-orange-500/50 transition-colors font-mono bg-stone-900 border-stone-800 text-stone-300"
                  placeholder="Your Nexus Mods auth cookie"
                />
              </div> */}

              {/* Nexus API Key */}
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-stone-400 font-sans">
                  Nexus API Key <span className="text-orange-500">(required for NXM downloads)</span>
                </label>
                <input
                  type="password"
                  value={settings.nexusApiKey}
                  onChange={(e) => setSettings({ ...settings, nexusApiKey: e.target.value })}
                  className="w-full border text-xs px-3 py-2 focus:outline-none focus:border-orange-500/50 transition-colors font-mono bg-stone-900 border-stone-800 text-stone-300"
                  placeholder="Your Nexus Mods API key"
                />
                <p className="text-xs text-stone-500 font-sans">
                  Get your API key from{' '}
                  <a
                    href="https://www.nexusmods.com/users/myaccount?tab=api"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-orange-400 hover:text-orange-300 underline"
                  >
                    Nexus Mods Account Settings
                  </a>
                </p>
              </div>
            </div>
          </div>

          <div className="h-px bg-stone-800/50" />

          {/* Downloads Section */}
          <div className="space-y-4">
            <div>
              <h3 className="text-sm font-medium text-stone-100 font-sans">
                Downloads
              </h3>
              <p className="text-xs text-stone-500 mt-1 font-sans">
                Manage download behavior
              </p>
            </div>

            <div className="space-y-3">
              <Checkbox
                label="Automatically install mods after download"
                checked={settings.autoInstall}
                onChange={(checked) => setSettings({ ...settings, autoInstall: checked })}
              />

              <Checkbox
                label="Ask for confirmation before installing"
                checked={settings.confirmBeforeInstall}
                onChange={(checked) => setSettings({ ...settings, confirmBeforeInstall: checked })}
              />

              <Checkbox
                label="Delete downloaded archive after successful install"
                checked={settings.deleteAfterInstall}
                onChange={(checked) => setSettings({ ...settings, deleteAfterInstall: checked })}
              />
            </div>
          </div>

          <div className="h-px bg-stone-800/50" />

          {/* Preferences Section */}
          <div className="space-y-4">
            <div>
              <h3 className="text-sm font-medium text-stone-100 font-sans">
                Preferences
              </h3>
              <p className="text-xs text-stone-500 mt-1 font-sans">
                Customize the look and feel of the application
              </p>
            </div>

            <div className="grid grid-cols-1 gap-4">
              {/* Theme */}
              <Select
                label="Theme"
                value={settings.theme}
                onChange={(value) => setSettings({ ...settings, theme: value as Theme })}
                options={[
                  { label: 'System', value: 'System' },
                  { label: 'Dark', value: 'Dark' },
                  { label: 'Light', value: 'Light' },
                ]}
              />

              {/* Language */}
              <Select
                label="Language"
                value={settings.language}
                onChange={(value) => setSettings({ ...settings, language: value as Language })}
                options={[
                  { label: 'English', value: 'English' },
                  { label: 'Bahasa Indonesia', value: 'Bahasa Indonesia' },
                ]}
              />

              {/* Mod Groups */}
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-stone-400 font-sans">
                  Mod Groups
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {(['None', 'Folder', 'Pack'] as ModGroups[]).map((group) => (
                    <label
                      key={group}
                      className={`cursor-pointer border p-2 flex flex-col items-center justify-center gap-1 transition-colors ${settings.modGroups === group
                        ? 'border-orange-500/50 bg-orange-500/10'
                        : 'border-stone-800 bg-stone-900 hover:border-stone-700'
                        }`}
                    >
                      <input
                        type="radio"
                        name="modGroups"
                        value={group}
                        checked={settings.modGroups === group}
                        onChange={(e) => setSettings({ ...settings, modGroups: e.target.value as ModGroups })}
                        className="hidden"
                      />
                      <span className={`text-xs font-medium font-sans ${settings.modGroups === group ? 'text-orange-300' : 'text-stone-300'
                        }`}>
                        {group}
                      </span>
                    </label>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="p-6 border-t border-stone-800">
          <button
            onClick={handleSave}
            className="w-full bg-orange-600 hover:bg-orange-500 text-white font-bold text-sm uppercase py-3 border-2 border-orange-800 border-b-4 border-r-4 active:border-b-2 active:border-r-2 active:translate-y-0.5 transition-none shadow-none"
          >
            Save Settings
          </button>
        </div>
      </div>
    </div>
  );
};
