# Project Context

## Purpose
Treasure Chest is a desktop mod manager for Stardew Valley. It provides a graphical interface for users to:
- Browse and manage installed SMAPI mods
- Install mods from URLs or local files
- Enable/disable mods by renaming folders (prepending `.`)
- View mod metadata (name, author, version, dependencies)
- Track mod status (working, update available, errors)

## Tech Stack
- **Frontend**: React 19.1, TypeScript 5.8, Vite 7
- **Backend**: Rust (Tauri 2)
- **Styling**: Tailwind CSS v4 with stone/orange theme
- **UI Components**: Custom components with Lucide React icons
- **Build System**: Vite (frontend), Cargo (backend)
- **Runtime**: Tauri (cross-platform desktop app)

## Project Conventions

### Code Style
- **TypeScript/React**:
  - Strict mode enabled (`noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`)
  - Functional components with hooks (no class components)
  - Named exports for components (e.g., `export const Header`)
  - Props interfaces defined with `interface ComponentNameProps`
  - camelCase for variables, functions, and props
  - PascalCase for components and type definitions
- **Rust**:
  - snake_case for variables and functions
  - Standard Rust 2021 edition conventions
  - Tauri command functions use `#[tauri::command]` attribute
- **File Organization**:
  - `src/components/layout/` - Layout components (Header, Toolbar)
  - `src/components/features/` - Feature-specific components (mods/)
  - `src/components/ui/` - Reusable UI components
  - `src/types/` - TypeScript type definitions
  - `src-tauri/src/` - Rust backend code

### Architecture Patterns
- **Component Architecture**: Functional React components organized by feature and layout
- **State Management**: React hooks (`useState`, `useEffect`) - no external state library
- **Frontend-Backend Communication**: Tauri IPC using `invoke()` from `@tauri-apps/api/core`
- **Type Safety**: Shared types between Rust (Serde) and TypeScript interfaces
- **Styling**: Tailwind utility classes, no CSS modules or styled-components
- **Error Handling**: Try-catch with fallback to mock data in development

### Testing Strategy
No testing framework currently configured. Future considerations:
- Unit tests with Vitest for React components
- Integration tests for Tauri commands
- E2E tests for user workflows

### Git Workflow
Not currently defined. Standard Git practices apply.

## Domain Context

### Stardew Valley Modding
- **SMAPI**: Stardew Modding API - required for most mods
- **Mod Structure**: Each mod has a `manifest.json` file with metadata:
  - `Name`, `Author`, `Version`, `UniqueID`
  - `Description`, `Dependencies` (optional)
- **Mod Location**: `{GamePath}/Mods/` directory
- **Enabling/Disabling**: Convention is to prepend `.` to folder name to disable
- **Mod Sources**: Primarily Nexus Mods (nexusmods.com/stardewvalley)

### Key Domain Models
```typescript
interface Mod {
  id: string;           // UUID generated at runtime
  name: string;         // From manifest
  author: string;
  version: string;
  uniqueId: string;     // SMAPI unique identifier
  description?: string;
  dependencies?: string[];
  isEnabled: boolean;   // Based on folder name
  status: 'working' | 'update-available' | 'error' | 'disabled';
}
```

## Important Constraints
- **Desktop Only**: Tauri app, not a web application
- **File System Access**: Requires permissions to read/write game directory
- **No Git Repository**: Project is not currently in a git repository
- **Windows/Linux/macOS**: Cross-platform support via Tauri
- **SMAPI Compatibility**: Must respect SMAPI mod manifest format

## External Dependencies
- **Nexus Mods**: Potential future integration for mod browsing/downloading
- **Game Installation**: Requires user to specify Stardew Valley installation path
- **Network**: HTTP requests for downloading mods from URLs (using `reqwest` in Rust)
- **File Formats**: ZIP archive extraction for mod installation
