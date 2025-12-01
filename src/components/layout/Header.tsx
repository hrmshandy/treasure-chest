import React from 'react';
import { Bell, Settings, X, DownloadCloud, Play, RefreshCw } from 'lucide-react';
import { NexusLogo } from '../ui/NexusLogo';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../hooks/useToast';

interface HeaderProps {
    onOpenSettings: () => void;
    onOpenDownloads: () => void;
}

export const Header: React.FC<HeaderProps> = ({ onOpenSettings, onOpenDownloads }) => {
    const appWindow = getCurrentWindow();
    const { success, error } = useToast();
    const [isLoadingGame, setIsLoadingGame] = React.useState(false);

    const onClose = () => {
        appWindow.close();
    };

    const handleLaunchGame = async () => {
        setIsLoadingGame(true);

        try {
            await invoke('launch_game');
            success('Game launched successfully!');
        } catch (err) {
            console.error('Failed to launch game:', err);
            error('Failed to launch game', String(err));
        } finally {
            setTimeout(() => {
                setIsLoadingGame(false);
            }, 10000);
        }
    };

    return (
        <header className="flex-none h-10 border-b bg-stone-900/80 backdrop-blur-md flex items-center justify-between px-6 z-20 border-stone-800">
            <div className="flex items-center gap-4">
                <div className="w-8 h-8 flex items-center justify-center">
                    <img src="/Treasure_Chest.png" alt="Treasure Chest" />
                </div>
                <h1 className="font-bold tracking-widest text-xl text-stone-100 uppercase drop-shadow-[2px_2px_0_#000]">
                    Treasure Chest
                </h1>
            </div>

            <div className="flex items-center gap-1">
                <a
                    href="https://www.nexusmods.com/stardewvalley"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="group p-2 rounded-md transition-colors hover:bg-stone-800"
                    title="Nexus Mods"
                >
                    <NexusLogo />
                </a>
                <button
                    onClick={handleLaunchGame}
                    className="p-2 rounded-md text-stone-500 transition-colors hover:bg-stone-800 hover:text-stone-200 hover:cursor-pointer group relative"
                    title="Launch Stardew Valley"
                >
                    {isLoadingGame ? (
                        <RefreshCw className="w-6 h-6 animate-spin" />
                    ) : (
                        <>
                            <img src="/pufferchick.svg" alt="Launch Game" className="w-6 h-6" />
                            <Play className="w-4 h-4 absolute bottom-2 right-1 text-white" />
                        </>
                    )}
                </button>
                <div className="h-5 w-px bg-stone-800 mx-2"></div>
                <button className="p-2 rounded-md text-stone-500 transition-colors relative hover:bg-stone-800 hover:text-stone-200">
                    <Bell className="w-5 h-5" />
                    <span className="absolute top-2 right-2 w-2 h-2 bg-orange-500 rounded-full border-2 border-stone-950"></span>
                </button>
                <button
                    onClick={onOpenDownloads}
                    className="p-2 rounded-md text-stone-500 transition-colors hover:bg-stone-800 hover:text-stone-200"
                    title="Downloads"
                >
                    <DownloadCloud className="w-5 h-5" />
                </button>
                <button
                    onClick={onOpenSettings}
                    className="p-2 rounded-md text-stone-500 transition-colors hover:bg-stone-800 hover:text-stone-200"
                >
                    <Settings className="w-5 h-5" />
                </button>
                <button
                    onClick={onClose}
                    className="p-2 rounded-md text-stone-500 transition-colors hover:bg-stone-800 hover:text-stone-200 -mr-5"
                >
                    <X className="w-5 h-5" />
                </button>
            </div>
        </header>
    );
};
