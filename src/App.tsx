import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Header } from './components/layout/Header';
import { Toolbar } from './components/layout/Toolbar';
import { Footer } from './components/layout/Footer';
import { ModList } from './components/features/mods/ModList';
import { AddModModal } from './components/features/mods/AddModModal';
import { SettingsModal } from './components/features/settings/SettingsModal';
import { Mod } from './types/mod';
import { Settings, defaultSettings } from './types/settings';
import { MOCK_MODS } from './data/mock';
import { DownloadManager } from './components/features/downloads/DownloadManager';
import { ToastContainer } from './components/ui/Toast';
import { useToast } from './hooks/useToast';

function App() {
  const [mods, setMods] = useState<Mod[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [isAddModModalOpen, setIsAddModModalOpen] = useState(false);
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);
  const [isDownloadManagerOpen, setIsDownloadManagerOpen] = useState(false);
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [showAutoDetectAlert, setShowAutoDetectAlert] = useState(false);
  const [newlyInstalledModId, setNewlyInstalledModId] = useState<string | null>(null);

  const { toasts, dismissToast, showToast } = useToast();


  useEffect(() => {
    initializeSettings();
  }, []);

  useEffect(() => {
    if (settings.gamePath) {
      loadMods();
    }
  }, [settings.gamePath]);

  async function initializeSettings() {
    try {
      // Try to load existing settings
      const loadedSettings = await invoke<Settings>('load_settings');
      setSettings(loadedSettings);

      // If no game path is configured, try auto-detection
      if (!loadedSettings.gamePath) {
        const [gamePath, smapiPath] = await invoke<[string | null, string | null]>('auto_detect_paths');

        if (gamePath) {
          // Auto-detection succeeded
          const newSettings = {
            ...loadedSettings,
            gamePath,
            smapiPath: smapiPath || '',
          };
          setSettings(newSettings);
          await invoke('save_settings', { settings: newSettings });
        } else {
          // Auto-detection failed
          setShowAutoDetectAlert(true);
          setIsSettingsModalOpen(true);
        }
      }
    } catch (error) {
      console.error('Failed to initialize settings:', error);
      // Try auto-detection anyway
      try {
        const [gamePath, smapiPath] = await invoke<[string | null, string | null]>('auto_detect_paths');
        if (gamePath) {
          const newSettings = {
            ...defaultSettings,
            gamePath,
            smapiPath: smapiPath || '',
          };
          setSettings(newSettings);
          await invoke('save_settings', { settings: newSettings });
        } else {
          setShowAutoDetectAlert(true);
          setIsSettingsModalOpen(true);
        }
      } catch (autoDetectError) {
        console.error('Auto-detection failed:', autoDetectError);
        setShowAutoDetectAlert(true);
        setIsSettingsModalOpen(true);
      }
    }
  }

  async function loadMods() {
    if (!settings.gamePath) return;

    try {
      const result = await invoke<Mod[]>('scan_mods', { gamePath: settings.gamePath });
      // Map Rust Mod to Frontend Mod (if needed, but they should match mostly)
      // We need to add 'status' and 'installDate' which might not be in the Rust struct yet
      const mappedMods = result.map(m => ({
        ...m,
        status: 'working' as const, // Default for now
        installDate: 'Unknown', // Default for now
        isEnabled: m.is_enabled, // Map snake_case to camelCase
      }));
      setMods(mappedMods);
    } catch (error) {
      console.warn('Failed to scan mods (likely running in browser), using mock data:', error);
      setMods(MOCK_MODS);
    }
  }

  const handleToggleMod = async (id: string, enabled: boolean) => {
    // Optimistic update
    setMods(mods.map(m => m.id === id ? { ...m, isEnabled: enabled } : m));
    // TODO: Call backend to actually toggle
  };

  const handleUpdateMod = (id: string) => {
    console.log('Update mod:', id);
  };

  const handleDeleteMod = (id: string) => {
    setMods(mods.filter(m => m.id !== id));
  };

  const handleInstallMod = async (url: string) => {
    console.log('Installing mod from:', url);
    try {
      await invoke('install_mod', { url, gamePath: settings.gamePath });
      loadMods(); // Refresh list
    } catch (error) {
      console.error('Failed to install mod:', error);
    }
  };

  const handleSaveSettings = async (newSettings: Settings) => {
    try {
      await invoke('save_settings', { settings: newSettings });
      setSettings(newSettings);
      setShowAutoDetectAlert(false);
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert('Failed to save settings. Please try again.');
    }
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-stone-950 text-stone-400 font-sans selection:bg-indigo-500/30 selection:text-indigo-200">
      {/* Auto-detect alert */}
      {showAutoDetectAlert && (
        <div className="fixed top-4 right-4 z-50 max-w-md p-4 bg-orange-900/90 border-2 border-orange-500 text-orange-100 shadow-lg">
          <p className="text-sm font-medium">
            Could not auto-detect Stardew Valley installation. Please configure the game path manually in settings.
          </p>
          <button
            onClick={() => setShowAutoDetectAlert(false)}
            className="mt-2 text-xs underline hover:no-underline"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Test NXM button for debugging */}
      {import.meta.env.DEV && (
        <div className="fixed top-4 left-4 z-50">
          <button
            onClick={async () => {
              // Test URL from Stardew Valley Expanded mod
              const testUrl = "nxm://stardewvalley/mods/3753/files/149330?key=test&expires=9999999999";
              console.log("Testing NXM URL:", testUrl);
              try {
                await invoke('test_nxm_url', { url: testUrl });
                console.log("✅ Test NXM URL processed successfully");
              } catch (error) {
                console.error("❌ Test NXM URL failed:", error);
                showToast('error', 'Test Failed', { message: String(error) });
              }
            }}
            className="px-3 py-2 bg-purple-600 text-white text-xs font-bold border-2 border-purple-400 shadow-lg hover:bg-purple-700"
          >
            🧪 Test NXM
          </button>
        </div>
      )}

      <DownloadManager
        isOpen={isDownloadManagerOpen}
        onClose={() => setIsDownloadManagerOpen(false)}
        onModInstalled={(uniqueId) => {
          setNewlyInstalledModId(uniqueId);
          loadMods();
          // Clear highlight after 5 seconds
          setTimeout(() => setNewlyInstalledModId(null), 5000);
        }}
        onToast={(type, title, message) => showToast(type, title, message ? { message } : undefined)}
      />

      <ToastContainer toasts={toasts} onDismiss={dismissToast} />

      <Header
        onOpenSettings={() => setIsSettingsModalOpen(true)}
        onOpenDownloads={() => setIsDownloadManagerOpen(true)}
      />

      <main className="flex-1 flex flex-col min-h-0 bg-stone-950">
        <Toolbar
          onAddMod={() => setIsAddModModalOpen(true)}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
        />

        <ModList
          mods={mods.filter(m => m.name.toLowerCase().includes(searchQuery.toLowerCase()))}
          onToggleMod={handleToggleMod}
          onUpdateMod={handleUpdateMod}
          onDeleteMod={handleDeleteMod}
          highlightedModId={newlyInstalledModId}
        />
      </main>

      <AddModModal
        isOpen={isAddModModalOpen}
        onClose={() => setIsAddModModalOpen(false)}
        onInstall={handleInstallMod}
      />

      <SettingsModal
        isOpen={isSettingsModalOpen}
        onClose={() => setIsSettingsModalOpen(false)}
        settings={settings}
        onSave={handleSaveSettings}
      />

      <Footer />
    </div>
  );
}

export default App;
