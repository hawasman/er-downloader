# ER Downloader

A Tauri-based application for downloading and managing Elden Ring Convergence mod updates.

## Getting Started

#### Required Software

1. **Install these tools first:**

   - [Node.js](https://nodejs.org/en) (LTS version recommended)
   - [Rust](https://rustup.rs/) (Just follow the installer's default options)
   - [pnpm](https://pnpm.io/installation) (Run `npm install -g pnpm` after installing Node.js)
   - [Visual Studio Code](https://code.visualstudio.com/) (Recommended editor)

2. **VS Code Extensions:**
   - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
   - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### Configuration Steps

1. **Environment Setup**

   - Navigate to the `src-tauri` folder
   - Copy `.env.example` to a new file named `.env`
   - Open `.env` and update these values:
     ```env
     DROPBOX_TOKEN=your_dropbox_token_here
     PATCH_NOTES_URL=http://localhost/patch_notes.md
     UPDATES_URL=http://localhost/updates.json
     ```

2. **Dropbox Setup**

   - Go to [Dropbox App Console](https://www.dropbox.com/developers/apps/)
   - Create a new app (Choose "Scoped access" and "App folder")
   - Generate an access token
   - Copy the token to your `.env` file

3. **Dropbox Folder Structure**

   ```
   App Folder
   │   ConvergenceER.zip  (Contains initial mod files)
   └─── Updates/
       │─── v2.2.1.zip
       │─── v2.2.2.zip
       │─── v2.2.3.zip
       └─── ...
   Note: The initial mod file (`ConvergenceER.zip`) must contain a `version.txt` file with a single version number (e.g., "v2.2.1"). Update files in the Updates folder don't need this file.
   ```

### Application Architecture

The application is built with [Tauri](https://v2.tauri.app/start/), using:

- Frontend: React.js (in `src/` directory)
- Backend: Rust (in `src-tauri/` directory)

Key Rust Components:

- `downloader.rs`: Handles file downloads and extraction
- `helpers.rs`: Contains utility functions and data structures
- `lib.rs`: Manages app initialization and React-to-Rust communication

### Running the App

1. **First Time Setup:**

   ```bash
   # Install dependencies
   pnpm install

   # Start the development version
   pnpm tauri dev
   ```

2. **Testing Setup (Optional):**
   - Install [Caddy](https://caddyserver.com/) for local file serving
   - Create a folder for test files:
     ```
     fake-server/
     │── updates.json
     └── patch_notes.md
     ```
   - Start Caddy server:
     ```bash
     cd fake-server
     caddy file-server
     ```

### Folder Structure After Installation

When the mod is installed for the first time, it creates this structure based on the content of the zip:

```
Selected Installation Directory/
│─── locale/
│─── mod/
│─── modengine2/
│─── config_eldenring.toml
│─── modengine2_launcher.exe
└─── Start_Convergence.bat

Note: Update files must mirror the main mod's folder structure. For example, if an update modifies a file in `mod/action/script`, the update zip must contain the same path structure (`mod/action/script/modified_file`).
```

## Troubleshooting

- If the app crashes during development when looking for `updates.json` or `patch_notes.md`, this is normal and only happens in debug mode
- Make sure your Dropbox token has the correct permissions
- Check that the main mod zip file contain a `version.txt` file at the root level
- Verify that your `.env` file contains all required values
- Check the [Tauri documentation](https://v2.tauri.app/start/) if you wish to understand how it works
