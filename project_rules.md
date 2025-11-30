# Project Rules & Guidelines - Treasure Chest Mod Manager

## Architecture
- **Framework**: Tauri (Core) + React (Frontend).
- **Build Tool**: Vite.
- **Language**: Rust (Backend) + TypeScript (Frontend).
- **Styling**: TailwindCSS.
- **State Management**: 
  - Frontend: React Hooks / Context / Zustand.
  - Backend: Rust structs / State managed in Tauri commands.
- **IPC**: Use Tauri's `invoke` for frontend-to-backend communication and `emit`/`listen` for events.

## Coding Standards
- **Functional Components**: Use React Functional Components with Hooks.
- **Types**: Define shared types in `src/types` and ensure Rust structs derive `Serialize`/`Deserialize` to match.
- **Error Handling**: 
  - UI should display user-friendly error toasts.
  - Rust backend should return `Result<T, String>` (or custom error type) to be handled by the frontend.
- **File Structure**:
  - `src/`: Frontend React code.
  - `src-tauri/`: Rust backend code.

## Specific Feature Implementation Rules

### Mod Installation
1. **URL Parsing**: Identify if the URL is a Nexus Mods URL.
2. **API Check**: Fetch mod details from Nexus API.
3. **Download**: Stream download to a temporary folder.
4. **Extraction**: Extract using `7zip-bin` or `adm-zip` (prefer 7zip for .rar support if possible, or warn user).
5. **Validation**: Check for `manifest.json`.
6. **Installation**: Move to `Mods` folder.

### Dependency Management
- When installing a mod, parse its requirements.
- If a requirement is missing, add it to the download queue automatically.
- Detect circular dependencies.

### Mod Updates
- Compare local `Version` in `manifest.json` with remote version from Nexus API.
- "Update All" should queue all outdated mods.

### Backup
- Before updating, zip the current mod folder to `Backups/{ModName}-{Version}.zip`.

## UI/UX
- Maintain the "Pixel" aesthetic.
- Use the provided color palette (Zinc/Stone/Orange).
- Ensure the UI is responsive to window resizing.
