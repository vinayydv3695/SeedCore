# SeedCore UI - Quick Start Guide

## Running the Application

### Development Mode
```bash
npm run tauri dev
```

This will:
1. Start the Vite dev server (frontend)
2. Compile and run the Rust backend
3. Open the SeedCore application window

### Production Build
```bash
npm run tauri build
```

This creates a production-ready executable in `src-tauri/target/release/`

## UI Features

### Header
- **Logo & Stats**: Shows active/total torrents count
- **Global Statistics**: 
  - Download/Upload speeds
  - Total downloaded/uploaded bytes
- **Add Torrent Button**: Opens the add torrent dialog

### Add Torrent Dialog
- **File Picker**: Select .torrent files from your filesystem
- **Magnet Link**: Paste magnet links (coming soon)

### Torrent List
- **Filter Tabs**: View All, Active, Downloading, Seeding, or Paused torrents
- **Real-time Updates**: Torrents refresh every 2 seconds
- **Empty State**: Helpful message when no torrents match the filter

### Torrent Item
Each torrent shows:
- **Name & Size**: Torrent name and total size
- **State**: Current state (Downloading, Seeding, Paused, etc.)
- **ETA**: Estimated time remaining (for active downloads)
- **Progress Bar**: Visual progress indicator with percentage
- **Statistics**:
  - Download speed
  - Upload speed
  - Connected peers
  - Seeds
- **Controls**:
  - Play/Pause button
  - Delete button

## Keyboard Shortcuts
- None yet (coming in future updates)

## Data Storage

### Database Location
- **Linux**: `~/.config/seedcore/data.db`
- **macOS**: `~/Library/Application Support/com.seedcore.app/data.db`
- **Windows**: `C:\Users\{user}\AppData\Roaming\seedcore\data.db`

### What's Stored
- Torrent metadata
- Download progress (bitfield)
- Downloaded/uploaded bytes
- Application settings

## Troubleshooting

### App won't start
- Check if port 6881 is available
- Check logs in the console
- Try deleting the database and restarting

### Torrents not loading
- Check network connectivity
- Verify tracker URLs are accessible
- Check firewall settings

### UI not updating
- Check browser console for errors
- Restart the application
- Clear browser cache (Ctrl+Shift+R)

## Known Limitations
- Magnet links not yet implemented
- No settings UI yet (uses defaults)
- No bandwidth limiting UI
- No file priority selection
- No DHT support yet

## Next Steps
1. Add real torrents to test functionality
2. Implement settings dialog
3. Add bandwidth limiting controls
4. Add file selection for multi-file torrents
5. Implement queue management
