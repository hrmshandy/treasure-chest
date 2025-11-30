import React from 'react';
import { Bell, Settings, X, DownloadCloud } from 'lucide-react';
import { NexusLogo } from '../ui/NexusLogo';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface HeaderProps {
    onOpenSettings: () => void;
    onOpenDownloads: () => void;
}

export const Header: React.FC<HeaderProps> = ({ onOpenSettings, onOpenDownloads }) => {
    const appWindow = getCurrentWindow();

    const onClose = () => {
        appWindow.close();
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
