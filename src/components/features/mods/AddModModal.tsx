import React, { useState } from 'react';
import { X, Link, UploadCloud, File } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';

interface AddModModalProps {
    isOpen: boolean;
    onClose: () => void;
    onInstall: (url: string) => void;
}

export const AddModModal: React.FC<AddModModalProps> = ({ isOpen, onClose, onInstall }) => {
    const [activeTab, setActiveTab] = useState<'url' | 'file'>('file');
    const [url, setUrl] = useState('');
    const [selectedFile, setSelectedFile] = useState<string | null>(null);

    if (!isOpen) return null;

    const handleSubmit = () => {
        if (activeTab === 'url' && url) {
            onInstall(url);
            onClose();
        } else if (activeTab === 'file' && selectedFile) {
            onInstall(selectedFile); // Pass file path as "url"
            onClose();
        }
    };

    const handleFileSelect = async () => {
        try {
            const file = await open({
                multiple: false,
                filters: [{
                    name: 'Archives',
                    extensions: ['zip', 'rar', '7z']
                }]
            });

            if (file) {
                // @ts-ignore - plugin-opener returns string or string[] based on multiple
                setSelectedFile(file as string);
            }
        } catch (error) {
            console.error('Failed to open file dialog:', error);
        }
    };

    return (
        <div className="fixed inset-0 z-50" role="dialog" aria-modal="true">
            <div className="fixed inset-0 backdrop-blur-sm transition-opacity bg-black/60 bg-stone-950/95" onClick={onClose}></div>
            <div className="fixed inset-0 flex items-center justify-center p-4">
                <div className="border rounded-xl shadow-2xl w-full max-w-lg overflow-hidden transform transition-all bg-stone-950 border-stone-800">
                    <div className="px-6 py-5 border-b flex justify-between items-center border-stone-800">
                        <h3 className="text-base font-semibold tracking-tight text-stone-100 font-mono">
                            Add New Mod
                        </h3>
                        <button onClick={onClose} className="text-stone-500 hover:text-stone-200">
                            <X className="w-5 h-5" />
                        </button>
                    </div>

                    <div className="p-6">
                        {/* Tabs Header */}
                        {/* <div className="flex space-x-1 p-1 rounded-lg mb-6 border bg-stone-900 border-stone-800">
                            <button
                                onClick={() => setActiveTab('url')}
                                className={`flex-1 py-1.5 text-xs font-medium rounded-md shadow-sm transition-all font-mono ${activeTab === 'url' ? 'bg-stone-800 text-stone-100 border-stone-600' : 'text-stone-500 border-transparent hover:text-stone-300'}`}
                            >
                                URL
                            </button>
                            <button
                                onClick={() => setActiveTab('file')}
                                className={`flex-1 py-1.5 text-xs font-medium rounded-md shadow-sm transition-all font-mono ${activeTab === 'file' ? 'bg-stone-800 text-stone-100 border-stone-600' : 'text-stone-500 border-transparent hover:text-stone-300'}`}
                            >
                                File Upload
                            </button>
                        </div> */}

                        {/* URL Tab Content */}
                        {activeTab === 'url' && (
                            <div className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-xs font-medium text-stone-400 font-mono">
                                        Nexus Mods URL or Direct Link
                                    </label>
                                    <div className="relative">
                                        <Link className="absolute left-3 top-2.5 w-4 h-4 text-stone-500" />
                                        <input
                                            type="text"
                                            placeholder="https://www.nexusmods.com/stardewvalley/mods/..."
                                            value={url}
                                            onChange={(e) => setUrl(e.target.value)}
                                            className="w-full border rounded-lg pl-10 pr-4 py-2 text-sm focus:outline-none focus:border-orange-500/50 focus:ring-1 focus:ring-orange-500/50 transition-all bg-stone-900 border-stone-800 text-stone-200"
                                        />
                                    </div>
                                    <p className="text-[10px] text-stone-600 font-mono">
                                        Supports direct downloads from Nexus Mods if logged in.
                                    </p>
                                </div>
                            </div>
                        )}

                        {/* File Tab Content */}
                        {activeTab === 'file' && (
                            <div
                                onClick={handleFileSelect}
                                className="drop-zone w-full h-40 rounded-lg flex flex-col items-center justify-center text-center cursor-pointer border border-transparent hover:bg-stone-900/50 transition-colors"
                            >
                                {selectedFile ? (
                                    <>
                                        <div className="w-12 h-12 flex items-center justify-center mb-3 border-2 border-stone-600 bg-stone-800 text-green-400">
                                            <File className="w-6 h-6" />
                                        </div>
                                        <p className="text-sm font-medium text-stone-300 font-mono break-all px-4">
                                            {selectedFile}
                                        </p>
                                        <p className="text-xs text-stone-500 mt-1 font-mono">
                                            Click to change file
                                        </p>
                                    </>
                                ) : (
                                    <>
                                        <div className="w-12 h-12 flex items-center justify-center mb-3 border-2 border-stone-600 bg-stone-800 text-orange-400">
                                            <UploadCloud className="w-6 h-6" />
                                        </div>
                                        <p className="text-sm font-medium text-stone-300 font-mono">
                                            Click to select file
                                        </p>
                                        <p className="text-xs text-stone-500 mt-1 font-mono">
                                            .zip, .rar, or .7z files
                                        </p>
                                    </>
                                )}
                            </div>
                        )}
                    </div>

                    <div className="px-6 py-4 bg-stone-925 border-t flex justify-end gap-3 border-stone-800">
                        <button onClick={onClose} className="px-4 py-2 text-xs font-medium transition-colors text-stone-400 hover:text-stone-200 font-mono">
                            Cancel
                        </button>
                        <button
                            onClick={handleSubmit}
                            disabled={activeTab === 'url' ? !url : !selectedFile}
                            className="px-4 py-2 hover:bg-orange-500 rounded-md text-xs font-medium shadow-lg transition-all bg-orange-600 text-white shadow-orange-900/20 font-mono disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            Add Mod
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};
