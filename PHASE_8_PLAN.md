# Phase 8-11: qBittorrent-Level Professional Client Upgrade

## 🎯 Project Goal
Transform SeedCore into a professional-grade BitTorrent client with qBittorrent-level features, monitoring, and UI.

**User Requirements:**
- ✅ UI redesign first (sidebar + table + bottom panel)
- ✅ Keep card view as optional toggle
- ✅ Full qBittorrent parity for monitoring
- ✅ Include DHT + PEX in this phase

---

## 📐 Architecture Overview

### UI Structure (New)
```
┌─────────────────────────────────────────────────────────┐
│  Header (Logo, Stats, Add, Settings, View Toggle)      │
├──────────┬──────────────────────────────────────────────┤
│          │                                              │
│ Sidebar  │  Main Content (Table or Cards)              │
│          │                                              │
│ - All    │  ┌────────────────────────────────────────┐ │
│ - Active │  │ Name  Size  Progress  Speed  ETA  ... │ │
│ - Down   │  ├────────────────────────────────────────┤ │
│ - Seed   │  │ torrent1.iso    ...                    │ │
│ - Paused │  │ movie.mkv       ...                    │ │
│          │  │ ...                                    │ │
│ Categories│  └────────────────────────────────────────┘ │
│ - Movies │                                              │
│ - Games  │                                              │
│ - ...    │                                              │
│ Tags     │                                              │
├──────────┴──────────────────────────────────────────────┤
│  Bottom Details Panel (Tabs)                            │
│  [General] [Trackers] [Peers] [Pieces] [Files]          │
│  ┌────────────────────────────────────────────────────┐ │
│  │ Selected torrent details here...                   │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Data Flow
```
Frontend (React)
    ↓
Tauri Commands (IPC)
    ↓
AppState (Arc<RwLock<...>>)
    ↓
TorrentEngine → PeerManager → Peer connections
            → DiskManager → File I/O
            → PieceManager → Piece selection
            → Tracker → Announcements
            → DHT → Peer discovery
            → Database → Persistence
```

---

## 🚀 Implementation Phases

### **Phase 8A: UI Redesign - Layout & Navigation** (PRIORITY 1)

#### Components to Create:
1. **`Sidebar.tsx`** (NEW)
   - Filter sections (All, Active, Downloading, Seeding, Paused)
   - Categories section (Movies, Games, etc.)
   - Tags section
   - Counter badges
   - Collapsible sections
   - Active filter highlighting

2. **`TorrentTable.tsx`** (NEW)
   - Sortable columns (name, size, progress, speed, ETA, ratio, etc.)
   - Resizable columns
   - Row selection (single/multi)
   - Context menu (right-click)
   - Virtual scrolling for performance
   - Column visibility toggle

3. **`ViewToggle.tsx`** (NEW)
   - Switch between table/card view
   - Save preference to settings
   - Icons for each view mode

4. **Update `App.tsx`**
   - New 3-panel layout (sidebar + main + bottom)
   - View mode state management
   - Layout persistence

5. **Update `Header.tsx`**
   - Add view toggle button
   - Keep existing features

#### Features:
- ✅ Responsive sidebar (collapsible on small screens)
- ✅ Sortable table columns
- ✅ Context menu with common actions
- ✅ Keyboard navigation (arrow keys)
- ✅ Multi-select with Ctrl/Shift
- ✅ Remember column widths/order
- ✅ View mode toggle (table/cards)

---

### **Phase 8B: UI Redesign - Bottom Details Panel** (PRIORITY 1)

#### Components to Create:

1. **`BottomPanel.tsx`** (NEW)
   - Tab container
   - Resizable height (drag to resize)
   - Minimize/maximize toggle
   - Shows when torrent selected

2. **`tabs/GeneralTab.tsx`** (NEW)
   - Transfer stats (download/upload speeds, total downloaded/uploaded)
   - Progress (percentage, downloaded, remaining)
   - Info (hash, size, path, created date, comment)
   - Time stats (added, completed, last activity)

3. **`tabs/TrackersTab.tsx`** (NEW)
   - Tracker list table
   - Columns: URL, Status, Peers, Seeds, Leechers, Downloaded, Message
   - Status indicators (working, updating, error, disabled)
   - Last announce time, next announce time
   - Manual announce button
   - Add/remove/edit tracker buttons

4. **`tabs/PeersTab.tsx`** (NEW)
   - Live peer list table
   - Columns: IP, Port, Client, Flags, Progress, Down Speed, Up Speed, Downloaded, Uploaded
   - Peer flags icons (D=downloading, U=uploading, O=optimistic unchoke, S=snubbed)
   - Country flags (optional)
   - Add peer button
   - Ban peer context menu

5. **`tabs/PiecesTab.tsx`** (NEW)
   - Visual pieces map (grid of colored squares)
   - Color legend (have=green, downloading=yellow, missing=gray)
   - Availability heatmap overlay
   - Piece size info
   - Stats (have, downloading, missing)
   - Download order visualization

6. **`tabs/FilesTab.tsx`** (NEW)
   - File tree view
   - Columns: Name, Size, Progress, Priority
   - Priority controls (skip, low, normal, high)
   - Folder collapse/expand
   - "Open file" / "Open folder" buttons
   - Rename file

#### Features:
- ✅ Real-time updates (peers, speeds, progress)
- ✅ Resizable panel height
- ✅ Tab switching with keyboard (Ctrl+1-5)
- ✅ Panel minimize/maximize
- ✅ Empty state when no torrent selected

---

### **Phase 8C: Backend - Peer & Tracker Data Exposure** (PRIORITY 1)

#### Backend Changes:

1. **Extend `peer/mod.rs`**
   ```rust
   pub struct PeerInfo {
       pub ip: String,
       pub port: u16,
       pub client: String,  // Parse from peer_id
       pub is_choked: bool,
       pub is_interested: bool,
       pub am_choking: bool,
       pub am_interested: bool,
       pub progress: f64,  // 0.0-1.0
       pub download_speed: u64,  // bytes/sec
       pub upload_speed: u64,
       pub downloaded: u64,
       pub uploaded: u64,
       pub is_seeder: bool,
   }
   ```

2. **Extend `tracker/mod.rs`**
   ```rust
   pub struct TrackerInfo {
       pub url: String,
       pub status: TrackerStatus,  // Working, Updating, Error, Disabled
       pub status_message: String,
       pub peers: u32,
       pub seeds: u32,
       pub leechers: u32,
       pub downloaded: u32,
       pub last_announce: Option<i64>,  // timestamp
       pub next_announce: Option<i64>,
       pub last_scrape: Option<i64>,
   }
   
   pub enum TrackerStatus {
       Working,
       Updating,
       Error,
       Disabled,
   }
   ```

3. **New Commands in `commands.rs`**
   ```rust
   #[tauri::command]
   pub fn get_peer_list(torrent_id: String) -> Result<Vec<PeerInfo>, String>;
   
   #[tauri::command]
   pub fn get_tracker_list(torrent_id: String) -> Result<Vec<TrackerInfo>, String>;
   
   #[tauri::command]
   pub fn get_pieces_info(torrent_id: String) -> Result<PiecesInfo, String>;
   
   #[tauri::command]
   pub fn get_file_list(torrent_id: String) -> Result<Vec<FileInfo>, String>;
   
   #[tauri::command]
   pub fn set_file_priority(torrent_id: String, file_index: usize, priority: FilePriority) -> Result<(), String>;
   ```

4. **Extend `PeerManager`** (src-tauri/src/peer/manager.rs)
   - Add `get_peer_list()` method
   - Track per-peer stats
   - Parse client name from peer_id

5. **Extend `TorrentEngine`** (src-tauri/src/engine/mod.rs)
   - Store tracker list with status
   - Expose tracker info
   - Track announce/scrape history

#### TypeScript Types to Add:
```typescript
export interface PeerInfo {
  ip: string;
  port: number;
  client: string;
  flags: string;
  progress: number;
  download_speed: number;
  upload_speed: number;
  downloaded: number;
  uploaded: number;
}

export interface TrackerInfo {
  url: string;
  status: 'Working' | 'Updating' | 'Error' | 'Disabled';
  message: string;
  peers: number;
  seeds: number;
  leechers: number;
  last_announce: number | null;
  next_announce: number | null;
}

export interface PiecesInfo {
  total_pieces: number;
  pieces_have: number;
  pieces_downloading: number;
  bitfield: number[];  // 0=missing, 1=have, 2=downloading
  availability: number[];  // How many peers have each piece
}

export interface FileInfo {
  path: string;
  size: number;
  downloaded: number;
  priority: 'Skip' | 'Low' | 'Normal' | 'High';
}
```

---

### **Phase 9A: Backend - Queue Management System** (PRIORITY 2)

#### New Module: `src-tauri/src/queue/mod.rs`

```rust
pub struct QueueManager {
    /// Torrent priority queue
    queue: VecDeque<QueueEntry>,
    /// Active download slots
    max_active_downloads: usize,
    /// Active upload slots
    max_active_uploads: usize,
}

pub struct QueueEntry {
    pub torrent_id: String,
    pub priority: Priority,
    pub force_start: bool,  // Bypass queue limits
}

pub enum Priority {
    High,
    Normal,
    Low,
}
```

#### Features:
- Auto-start next torrent when one completes
- Force start (bypass limits)
- Manual queue reordering
- Priority-based scheduling
- Seeding ratio limits (auto-stop when ratio reached)

#### New Commands:
```rust
#[tauri::command]
pub fn set_torrent_priority(torrent_id: String, priority: Priority) -> Result<(), String>;

#[tauri::command]
pub fn move_queue_up(torrent_id: String) -> Result<(), String>;

#[tauri::command]
pub fn move_queue_down(torrent_id: String) -> Result<(), String>;

#[tauri::command]
pub fn force_start(torrent_id: String) -> Result<(), String>;
```

---

### **Phase 9B: Backend - Per-Torrent Limits & Categories** (PRIORITY 2)

#### Extend `TorrentEngine`:
```rust
pub struct TorrentLimits {
    pub download_limit: Option<u64>,  // bytes/sec, None = use global
    pub upload_limit: Option<u64>,
    pub max_connections: Option<usize>,
    pub max_upload_slots: Option<usize>,
}

pub struct TorrentMetadata {
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub save_path: PathBuf,
    pub auto_managed: bool,
    pub sequential_download: bool,
    pub super_seeding: bool,
}
```

#### Database Schema Extension:
- Add `categories` table
- Add `tags` table
- Add `torrent_metadata` table
- Add `torrent_limits` table

#### New Commands:
```rust
#[tauri::command]
pub fn set_torrent_limits(torrent_id: String, limits: TorrentLimits) -> Result<(), String>;

#[tauri::command]
pub fn set_category(torrent_id: String, category: String) -> Result<(), String>;

#[tauri::command]
pub fn add_tag(torrent_id: String, tag: String) -> Result<(), String>;

#[tauri::command]
pub fn get_categories() -> Result<Vec<CategoryInfo>, String>;
```

---

### **Phase 10A: Backend - DHT Implementation** (PRIORITY 3)

#### New Module: `src-tauri/src/dht/`

1. **`mod.rs`** - DHT manager
2. **`node.rs`** - Kademlia node
3. **`routing.rs`** - Routing table (k-buckets)
4. **`rpc.rs`** - DHT RPC messages (ping, find_node, get_peers, announce_peer)

#### DHT Protocol:
- Kademlia-based routing (160-bit node IDs)
- UDP protocol on port 6881
- Bootstrap from router.bittorrent.com
- K-bucket routing table (k=8)
- Periodic refresh (every 15 minutes)
- Store peer announcements

#### Integration:
- Add DHT as peer source in TorrentEngine
- Bootstrap on startup
- Announce torrents to DHT
- Fetch peers from DHT for trackerless torrents

#### Commands:
```rust
#[tauri::command]
pub fn get_dht_stats() -> Result<DhtStats, String>;
```

---

### **Phase 10B: Backend - PEX (Peer Exchange)** (PRIORITY 3)

#### Extension Messages:
```rust
// BEP 11 - Peer Exchange
pub enum ExtensionMessage {
    Handshake(ExtensionHandshake),
    PexMessage(PexMessage),
}

pub struct PexMessage {
    pub added: Vec<SocketAddr>,      // New peers
    pub added_flags: Vec<u8>,        // Flags for added peers
    pub dropped: Vec<SocketAddr>,    // Dropped peers
}
```

#### Features:
- Exchange peer lists with connected peers
- Respect PEX disable flag
- Limit exchange frequency (once per minute)
- Filter out bad peers
- Add discovered peers to peer pool

---

### **Phase 11: Polish & Testing** (PRIORITY 4)

#### UI Polish:
- ✅ Smooth animations (Framer Motion)
- ✅ Loading skeletons
- ✅ Empty states for all tabs
- ✅ Tooltips everywhere
- ✅ Keyboard shortcuts documentation
- ✅ Light theme implementation
- ✅ Accessibility (ARIA labels, focus management)

#### Backend Polish:
- ✅ Connection pooling
- ✅ Memory usage optimization
- ✅ Bandwidth limiting enforcement
- ✅ Error recovery
- ✅ Logging levels

#### Testing:
- ✅ Unit tests for new modules
- ✅ Integration tests for DHT
- ✅ E2E test with real torrents
- ✅ Performance testing (1000+ torrents)

---

## 📊 Success Metrics

### Phase 8 Complete When:
- ✅ Sidebar with filters, categories, tags
- ✅ Table view with sortable/resizable columns
- ✅ Bottom panel with 5 working tabs
- ✅ View toggle (table/cards) working
- ✅ Live peer list showing real data
- ✅ Tracker status displaying correctly
- ✅ Pieces map visualization complete
- ✅ Zero TypeScript errors
- ✅ All backend tests passing

### Phase 9 Complete When:
- ✅ Queue management working
- ✅ Priority system functional
- ✅ Per-torrent bandwidth limits enforced
- ✅ Categories and tags fully functional
- ✅ Database schema updated
- ✅ Settings persist correctly

### Phase 10 Complete When:
- ✅ DHT bootstrapping successful
- ✅ Peer discovery via DHT working
- ✅ PEX exchanging peers
- ✅ Magnet links work (with DHT)
- ✅ Trackerless torrents work

### Final Success (Phase 11):
- ✅ Professional UI matching qBittorrent
- ✅ All monitoring features working
- ✅ Queue system robust
- ✅ DHT + PEX functional
- ✅ Performance excellent (1000+ torrents)
- ✅ Zero crashes in 24h stress test

---

## 🎨 Design System

### Colors (Enhanced)
```css
/* Dark Theme */
--bg-primary: #0a0a0a       /* Main background */
--bg-secondary: #141414     /* Sidebar */
--bg-tertiary: #1e1e1e      /* Panel backgrounds */
--bg-elevated: #242424      /* Table rows hover */
--border: #2a2a2a           /* Borders */

--text-primary: #ffffff
--text-secondary: #a0a0a0
--text-tertiary: #707070

--accent-primary: #3b82f6   /* Primary blue */
--accent-hover: #2563eb
--success: #10b981          /* Green */
--warning: #f59e0b          /* Orange */
--error: #ef4444            /* Red */
--info: #06b6d4             /* Cyan */

/* Peer status colors */
--peer-downloading: #3b82f6
--peer-uploading: #10b981
--peer-snubbed: #ef4444
--peer-optimistic: #f59e0b

/* Piece colors */
--piece-have: #10b981
--piece-downloading: #f59e0b
--piece-missing: #404040
```

### Component Sizes
- Sidebar width: 220px (collapsible)
- Bottom panel height: 200-400px (resizable, default 280px)
- Table row height: 32px
- Header height: 64px

---

## 📁 New File Structure

```
SeedCore/
├── src-tauri/src/
│   ├── bencode.rs              ✅ Existing
│   ├── torrent/mod.rs          ✅ Existing
│   ├── tracker/
│   │   ├── mod.rs              ✅ Extend with TrackerInfo
│   │   └── http.rs             ✅ Extend with status tracking
│   ├── peer/
│   │   ├── mod.rs              ✅ Extend with PeerInfo
│   │   ├── handshake.rs        ✅ Existing
│   │   ├── message.rs          ✅ Extend with PEX messages
│   │   └── manager.rs          ✅ Extend with get_peer_list
│   ├── piece/
│   │   ├── mod.rs              ✅ Existing
│   │   ├── bitfield.rs         ✅ Existing
│   │   └── strategy.rs         ✅ Existing
│   ├── disk/mod.rs             ✅ Existing
│   ├── engine/mod.rs           ✅ Extend with monitoring
│   ├── database/mod.rs         ✅ Extend schema
│   ├── queue/                  🆕 NEW MODULE
│   │   └── mod.rs              🆕 Queue management
│   ├── dht/                    🆕 NEW MODULE
│   │   ├── mod.rs              🆕 DHT manager
│   │   ├── node.rs             🆕 Kademlia node
│   │   ├── routing.rs          🆕 Routing table
│   │   └── rpc.rs              🆕 DHT RPC
│   ├── commands.rs             ✅ Extend with new commands
│   ├── state.rs                ✅ Extend with queue state
│   ├── error.rs                ✅ Existing
│   └── utils.rs                ✅ Existing
│
├── src/
│   ├── components/
│   │   ├── Header.tsx              ✅ Update with view toggle
│   │   ├── Sidebar.tsx             🆕 NEW
│   │   ├── TorrentTable.tsx        🆕 NEW
│   │   ├── TorrentList.tsx         ✅ Keep for card view
│   │   ├── TorrentItem.tsx         ✅ Keep for card view
│   │   ├── ViewToggle.tsx          🆕 NEW
│   │   ├── BottomPanel.tsx         🆕 NEW
│   │   ├── tabs/
│   │   │   ├── GeneralTab.tsx      🆕 NEW
│   │   │   ├── TrackersTab.tsx     🆕 NEW
│   │   │   ├── PeersTab.tsx        🆕 NEW
│   │   │   ├── PiecesTab.tsx       🆕 NEW
│   │   │   └── FilesTab.tsx        🆕 NEW
│   │   ├── AddTorrentDialog.tsx    ✅ Existing
│   │   ├── SettingsDialog.tsx      ✅ Existing
│   │   ├── TorrentDetails.tsx      ✅ Keep but redesign
│   │   └── SpeedChart.tsx          ✅ Keep
│   ├── hooks/
│   │   └── useKeyboardShortcuts.ts ✅ Extend
│   ├── lib/
│   │   ├── api.ts                  ✅ Extend with new commands
│   │   └── utils.ts                ✅ Extend
│   ├── types/
│   │   └── index.ts                ✅ Extend with new types
│   ├── App.tsx                     ✅ Major redesign
│   └── index.css                   ✅ Extend styles
```

---

## 🚦 Getting Started

### Step 1: Start with Phase 8A
```bash
# Create new components
touch src/components/Sidebar.tsx
touch src/components/TorrentTable.tsx
touch src/components/ViewToggle.tsx

# Update App.tsx with new layout
```

### Step 2: Mock Data First
- Use placeholder data for peers, trackers
- Build UI fully functional with mock data
- Ensures UI is perfect before backend complexity

### Step 3: Connect Backend
- Implement backend data structures
- Wire up Tauri commands
- Replace mock data with real data

### Step 4: Test & Iterate
- Test with real torrents
- Performance profiling
- Bug fixes

---

**Let's build the future of BitTorrent clients!** 🚀
