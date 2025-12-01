import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
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
import { ConfirmDialog } from './components/ui/ConfirmDialog';
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
  const [deleteConfirmMod, setDeleteConfirmMod] = useState<Mod | null>(null);
  const [selectedModIds, setSelectedModIds] = useState<Set<string>>(new Set());
  const [isBulkDeleteConfirmOpen, setIsBulkDeleteConfirmOpen] = useState(false);

  // Enhanced UI State
  const [filterStatus, setFilterStatus] = useState<'all' | 'enabled' | 'disabled' | 'updates' | 'config'>('all');
  const [sortConfig, setSortConfig] = useState<{ key: string; direction: 'asc' | 'desc' }>({ key: 'name', direction: 'asc' });
  const [pagination, setPagination] = useState({ currentPage: 1, itemsPerPage: 10 });

  const { toasts, dismissToast, showToast } = useToast();

  // ... (selection handlers) ...

  const filteredAndSortedMods = useMemo(() => {
    let result = [...mods];

    // 1. Filter
    if (filterStatus !== 'all') {
      result = result.filter(mod => {
        switch (filterStatus) {
          case 'enabled': return mod.isEnabled;
          case 'disabled': return !mod.isEnabled;
          case 'updates': return mod.status === 'update-available';
          case 'config': return false; // Placeholder for config logic
          default: return true;
        }
      });
    }

    // 2. Search
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      result = result.filter(mod =>
        mod.name.toLowerCase().includes(query) ||
        mod.author.toLowerCase().includes(query)
      );
    }

    // 3. Sort
    result.sort((a, b) => {
      const aValue = a[sortConfig.key as keyof Mod] || '';
      const bValue = b[sortConfig.key as keyof Mod] || '';

      if (aValue === bValue) return 0;

      const comparison = aValue > bValue ? 1 : -1;
      return sortConfig.direction === 'asc' ? comparison : -comparison;
    });

    return result;
  }, [mods, filterStatus, searchQuery, sortConfig]);

  // 4. Paginate
  const totalPages = Math.ceil(filteredAndSortedMods.length / pagination.itemsPerPage);
  const paginatedMods = filteredAndSortedMods.slice(
    (pagination.currentPage - 1) * pagination.itemsPerPage,
    pagination.currentPage * pagination.itemsPerPage
  );

  // Reset page when filters change
  useEffect(() => {
    setPagination(prev => ({ ...prev, currentPage: 1 }));
  }, [filterStatus, searchQuery]);

  const handleSort = (key: string) => {
    setSortConfig(current => ({
      key,
      direction: current.key === key && current.direction === 'asc' ? 'desc' : 'asc'
    }));
  };


  const handleSelectMod = (id: string, selected: boolean) => {
    const newSelected = new Set(selectedModIds);
    if (selected) {
      newSelected.add(id);
    } else {
      newSelected.delete(id);
    }
    setSelectedModIds(newSelected);
  };

  const handleSelectAll = (selected: boolean) => {
    if (selected) {
      setSelectedModIds(new Set(mods.map(m => m.id)));
    } else {
      setSelectedModIds(new Set());
    }
  };


  useEffect(() => {
    initializeSettings();
  }, []);

  useEffect(() => {
    if (settings.gamePath) {
      loadMods();
    }
  }, [settings.gamePath]);

  // Auto-check for updates after mods load (with rate limiting)
  useEffect(() => {
    const autoCheckUpdates = async () => {
      // Only check if we have mods from Nexus
      const nexusMods = mods.filter(m => m.nexusId);
      if (nexusMods.length === 0) {
        console.log('⏭️ Skipping auto-check: No mods from Nexus');
        return;
      }

      // Rate limit: Only check once every 6 hours
      const lastCheckKey = 'last_update_check';
      const lastCheck = localStorage.getItem(lastCheckKey);
      const now = Date.now();
      const sixHours = 6 * 60 * 60 * 1000;

      if (lastCheck && (now - parseInt(lastCheck)) < sixHours) {
        console.log('⏭️ Skipping auto-check: Checked recently');
        return;
      }

      console.log('🔍 Auto-checking updates for', nexusMods.length, 'mods...');
      localStorage.setItem(lastCheckKey, now.toString());

      // Check updates for each Nexus mod (sequentially to avoid rate limiting)
      for (const mod of nexusMods) {
        try {
          const updateInfo = await invoke<{
            has_update: boolean;
            current_version: string;
            latest_version?: string;
            latest_file_id?: number;
          }>('check_mod_updates', {
            modPath: mod.path,
            currentVersion: mod.version,
            nexusModId: mod.nexusId,
          });

          if (updateInfo.has_update) {
            // Update mod status silently
            setMods(currentMods => currentMods.map(m => m.id === mod.id ? {
              ...m,
              status: 'update-available'
            } : m));
          }

          // Small delay between checks to be nice to the API
          await new Promise(resolve => setTimeout(resolve, 500));
        } catch (error) {
          console.warn(`Failed to check updates for ${mod.name}:`, error);
        }
      }

      const updatesAvailable = mods.filter(m => m.status === 'update-available').length;
      if (updatesAvailable > 0) {
        showToast('info', 'Updates Available', {
          message: `${updatesAvailable} mod${updatesAvailable > 1 ? 's have' : ' has'} updates available.`
        });
      }
    };

    // Delay auto-check slightly to let UI settle
    if (mods.length > 0) {
      const timer = setTimeout(autoCheckUpdates, 2000);
      return () => clearTimeout(timer);
    }
  }, [mods.length]); // Only trigger when mods count changes

  // Listen for mod installation events from backend
  useEffect(() => {
    const unlistenPromise = listen('mod-installed', (event: any) => {
      console.log('Mod installed event received:', event.payload);
      // Refresh mod list when a mod is installed
      if (settings.gamePath) {
        loadMods();
      }
    });

    return () => {
      unlistenPromise.then((unlisten: any) => unlisten());
    };
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
      const loadedMods = await invoke<Mod[]>('scan_mods', { gamePath: settings.gamePath });
      console.log('📦 Loaded mods from backend:', loadedMods);
      console.log('First mod example:', loadedMods[0]);

      // Set initial status based on enabled state
      const modsWithStatus = loadedMods.map(mod => ({
        ...mod,
        status: mod.isEnabled ? 'working' : 'disabled' as Mod['status'],
        installDate: new Date().toISOString(),
      }));

      console.log('📦 Mods with status:', modsWithStatus);
      setMods(modsWithStatus);
    } catch (error) {
      console.error('Failed to load mods:', error);
      // Fallback to mock data if running in browser or other error
      console.warn('Failed to scan mods (likely running in browser), using mock data:', error);
      setMods(MOCK_MODS);
    }
  }

  const handleToggleMod = async (id: string, enabled: boolean) => {
    const mod = mods.find(m => m.id === id);
    if (!mod) return;

    // Optimistic update
    setMods(mods.map(m => m.id === id ? {
      ...m,
      isEnabled: enabled,
      status: enabled ? 'working' : 'disabled'
    } : m));

    try {
      const newPath = await invoke<string>('toggle_mod_enabled', {
        modPath: mod.path,
        enabled
      });

      // Update path in state
      setMods(currentMods => currentMods.map(m => m.id === id ? {
        ...m,
        path: newPath,
        isEnabled: enabled,
        status: enabled ? 'working' : 'disabled'
      } : m));
    } catch (error) {
      console.error('Failed to toggle mod:', error);
      showToast('error', 'Failed to Toggle Mod', { message: String(error) });
      // Revert optimistic update
      setMods(mods.map(m => m.id === id ? mod : m));
    }
  };

  const handleUpdateMod = async (id: string) => {
    console.log('🔄 Update button clicked for mod:', id);
    const mod = mods.find(m => m.id === id);
    console.log('Found mod:', mod);

    if (!mod) {
      console.log('❌ Mod not found!');
      return;
    }

    // Check if mod has Nexus metadata
    console.log('Checking Nexus metadata:', { nexusId: mod.nexusId, nexusFileId: mod.nexusFileId });
    if (!mod.nexusId) {
      console.log('❌ No Nexus ID found');
      showToast('error', 'Cannot Check Updates', { message: 'This mod was not installed from Nexus Mods.' });
      return;
    }

    console.log('✅ Calling check_mod_updates with:', {
      modPath: mod.path,
      currentVersion: mod.version,
      nexusModId: mod.nexusId,
    });

    try {
      const updateInfo = await invoke<{
        has_update: boolean;
        current_version: string;
        latest_version?: string;
        latest_file_id?: number;
      }>('check_mod_updates', {
        modPath: mod.path,
        currentVersion: mod.version,
        nexusModId: mod.nexusId,
      });

      console.log('📊 Update info received:', updateInfo);

      if (updateInfo.has_update && updateInfo.latest_version) {
        console.log('✅ Update available!');
        // Update mod status
        setMods(mods.map(m => m.id === id ? {
          ...m,
          status: 'update-available'
        } : m));

        showToast('info', 'Update Available', {
          message: `${mod.name}: ${updateInfo.current_version} → ${updateInfo.latest_version}`
        });
      } else {
        console.log('ℹ️ No update available');
        showToast('success', 'Up to Date', {
          message: `${mod.name} is already at the latest version.`
        });
      }
    } catch (error) {
      console.error('❌ Failed to check updates:', error);
      showToast('error', 'Update Check Failed', { message: String(error) });
    }
  };

  const handleDeleteMod = (id: string) => {
    const mod = mods.find(m => m.id === id);
    if (!mod) return;

    // Show custom confirmation dialog
    setDeleteConfirmMod(mod);
  };

  const confirmDeleteMod = async () => {
    if (!deleteConfirmMod) return;

    const mod = deleteConfirmMod;
    setDeleteConfirmMod(null); // Close dialog immediately

    try {
      // Call backend to delete from disk
      await invoke('delete_mod', { modPath: mod.path });

      // Refresh mod list to reflect deletion
      await loadMods();

      // Show success notification AFTER list is refreshed
      showToast('success', 'Mod Deleted', { message: `${mod.name} has been deleted.` });
    } catch (error) {
      console.error('Failed to delete mod:', error);
      showToast('error', 'Failed to Delete Mod', { message: String(error) });
    }
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

  const handleOpenModsFolder = async () => {
    if (!settings.gamePath) {
      showToast('error', 'Game Path Not Set', { message: 'Please configure the game path in settings first.' });
      return;
    }

    try {
      await invoke('open_game_mods_folder', { gamePath: settings.gamePath });
    } catch (error) {
      console.error('Failed to open mods folder:', error);
      showToast('error', 'Failed to Open Folder', { message: String(error) });
    }
  };



  const handleBulkEnable = async () => {
    const selected = Array.from(selectedModIds);
    let successCount = 0;
    let failCount = 0;

    // Optimistic update for UI responsiveness
    setMods(currentMods => currentMods.map(m => selectedModIds.has(m.id) ? {
      ...m,
      isEnabled: true,
      status: 'working'
    } : m));

    for (const id of selected) {
      const mod = mods.find(m => m.id === id);
      if (!mod) continue;

      try {
        await invoke('toggle_mod_enabled', { modPath: mod.path, enabled: true });
        successCount++;
      } catch (error) {
        console.error(`Failed to enable mod ${mod.name}:`, error);
        failCount++;
      }
    }

    // Refresh mods to ensure correct state/paths
    await loadMods();
    setSelectedModIds(new Set());

    if (failCount > 0) {
      showToast('warning', 'Bulk Action Completed with Errors', {
        message: `Enabled ${successCount} mods. Failed to enable ${failCount} mods.`
      });
    } else {
      showToast('success', 'Bulk Action Complete', { message: `Enabled ${successCount} mods` });
    }
  };

  const handleBulkDisable = async () => {
    const selected = Array.from(selectedModIds);
    let successCount = 0;
    let failCount = 0;

    // Optimistic update
    setMods(currentMods => currentMods.map(m => selectedModIds.has(m.id) ? {
      ...m,
      isEnabled: false,
      status: 'disabled'
    } : m));

    for (const id of selected) {
      const mod = mods.find(m => m.id === id);
      if (!mod) continue;

      try {
        await invoke('toggle_mod_enabled', { modPath: mod.path, enabled: false });
        successCount++;
      } catch (error) {
        console.error(`Failed to disable mod ${mod.name}:`, error);
        failCount++;
      }
    }

    await loadMods();
    setSelectedModIds(new Set());

    if (failCount > 0) {
      showToast('warning', 'Bulk Action Completed with Errors', {
        message: `Disabled ${successCount} mods. Failed to disable ${failCount} mods.`
      });
    } else {
      showToast('success', 'Bulk Action Complete', { message: `Disabled ${successCount} mods` });
    }
  };

  const handleBulkDelete = () => {
    setIsBulkDeleteConfirmOpen(true);
  };

  const confirmBulkDelete = async () => {
    setIsBulkDeleteConfirmOpen(false);

    const selected = Array.from(selectedModIds);
    let successCount = 0;
    let failCount = 0;

    for (const id of selected) {
      const mod = mods.find(m => m.id === id);
      if (!mod) continue;

      try {
        await invoke('delete_mod', { modPath: mod.path });
        successCount++;
      } catch (error) {
        console.error(`Failed to delete mod ${mod.name}:`, error);
        failCount++;
      }
    }

    await loadMods();
    setSelectedModIds(new Set());

    if (failCount > 0) {
      showToast('warning', 'Bulk Delete Completed with Errors', {
        message: `Deleted ${successCount} mods. Failed to delete ${failCount} mods.`
      });
    } else {
      showToast('success', 'Bulk Delete Complete', { message: `Deleted ${successCount} mods` });
    }
  };

  const handleAutoDetect = async () => {
    try {
      const [gamePath, smapiPath] = await invoke<[string | null, string | null]>('auto_detect_paths');
      if (gamePath) {
        const newSettings = {
          ...settings,
          gamePath,
          smapiPath: smapiPath || '',
        };
        setSettings(newSettings);
        await invoke('save_settings', { settings: newSettings });
        showToast('success', 'Auto-detection Successful', { message: 'Game and SMAPI paths detected.' });
      } else {
        showToast('error', 'Auto-detection Failed', { message: 'Could not find Stardew Valley installation.' });
      }
    } catch (error) {
      console.error('Auto-detection error:', error);
      showToast('error', 'Auto-detection Error', { message: String(error) });
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
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          onAddMod={() => setIsAddModModalOpen(true)}
          onOpenMods={handleOpenModsFolder}
          onRefresh={loadMods}
          filterStatus={filterStatus}
          onFilterChange={setFilterStatus}
        />

        <ModList
          mods={paginatedMods}
          onToggleMod={handleToggleMod}
          onUpdateMod={handleUpdateMod}
          onDeleteMod={handleDeleteMod}
          highlightedModId={newlyInstalledModId}
          selectedModIds={selectedModIds}
          onSelectMod={handleSelectMod}
          onSelectAll={handleSelectAll}
          sortConfig={sortConfig}
          onSort={handleSort}
          pagination={{
            currentPage: pagination.currentPage,
            totalPages,
            totalItems: filteredAndSortedMods.length,
            itemsPerPage: pagination.itemsPerPage
          }}
          onPageChange={(page) => setPagination(prev => ({ ...prev, currentPage: page }))}
          onItemsPerPageChange={(items) => setPagination(prev => ({ ...prev, itemsPerPage: items, currentPage: 1 }))}
        />
      </main>

      {/* Bulk Actions Toolbar */}
      {selectedModIds.size > 0 && (
        <div className="absolute bottom-12 left-1/2 transform -translate-x-1/2 bg-stone-900 border border-stone-700 rounded-lg shadow-xl px-6 py-3 flex items-center gap-4 z-50 animate-in slide-in-from-bottom-4 fade-in duration-200">
          <span className="text-sm font-medium text-stone-400 font-mono">{selectedModIds.size} selected</span>
          <div className="h-4 w-px bg-stone-700" />
          <button onClick={handleBulkEnable} className="text-xs font-medium text-stone-300 hover:text-green-400 transition-colors font-mono uppercase tracking-wider">
            Enable
          </button>
          <button onClick={handleBulkDisable} className="text-xs font-medium text-stone-300 hover:text-stone-100 transition-colors font-mono uppercase tracking-wider">
            Disable
          </button>
          <div className="h-4 w-px bg-stone-700" />
          <button onClick={handleBulkDelete} className="text-xs font-medium text-stone-300 hover:text-red-400 transition-colors font-mono uppercase tracking-wider">
            Delete
          </button>
        </div>
      )}

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
        onAutoDetect={handleAutoDetect}
      />

      <ConfirmDialog
        isOpen={deleteConfirmMod !== null}
        title="Delete Mod"
        message={deleteConfirmMod ? `Are you sure you want to delete "${deleteConfirmMod.name}"? This action cannot be undone.` : ''}
        confirmLabel="Delete"
        variant="danger"
        onConfirm={confirmDeleteMod}
        onCancel={() => setDeleteConfirmMod(null)}
      />

      <ConfirmDialog
        isOpen={isBulkDeleteConfirmOpen}
        title="Delete Multiple Mods"
        message={`Are you sure you want to delete ${selectedModIds.size} mods? This action cannot be undone.`}
        confirmLabel={`Delete ${selectedModIds.size} Mods`}
        variant="danger"
        onConfirm={confirmBulkDelete}
        onCancel={() => setIsBulkDeleteConfirmOpen(false)}
      />

      <Footer />
    </div>
  );
}

export default App;
