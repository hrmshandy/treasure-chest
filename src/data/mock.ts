import { Mod } from '../types/mod';

export const MOCK_MODS: Mod[] = [
    {
        id: '1',
        name: 'Content Patcher',
        author: 'Pathoschild',
        version: '1.30.0',
        uniqueId: 'Pathoschild.ContentPatcher',
        isEnabled: true,
        installDate: 'Oct 24, 2023',
        status: 'working',
        endorsements: 12000,
        path: 'Pathoschild.ContentPatcher',
    },
    {
        id: '2',
        name: 'CJB Cheats Menu',
        author: 'CJBok',
        version: '1.33.0',
        uniqueId: 'CJBok.CheatsMenu',
        isEnabled: true,
        installDate: 'Aug 10, 2023',
        status: 'update-available',
        endorsements: 8500,
        path: 'CJBok.CheatsMenu',
    },
    {
        id: '3',
        name: 'SpaceCore',
        author: 'spacechase0',
        version: '1.15.0',
        uniqueId: 'spacechase0.SpaceCore',
        isEnabled: false,
        installDate: 'Sep 01, 2023',
        status: 'error',
        endorsements: 4000,
        path: 'spacechase0.SpaceCore',
    }
];
