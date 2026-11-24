# Rusteus-Coop — Beginner-Friendly User Manual & Setup Guide

Welcome to **Rusteus-Coop**, the high-performance, next-generation Rust rewrite of NucleusCoop. Rusteus-Coop allows you to transform almost any PC game into a local split-screen multiplayer experience on a single computer using multiple monitors, controllers, keyboards, and mice.

---

## 1. System Requirements & Prerequisites

Before launching Rusteus-Coop, ensure your system meets the following requirements:

### Minimum Requirements
- **Operating System**: Windows 10 (64-bit) or Windows 11 (64-bit).
- **Filesystem**: An **NTFS-formatted hard drive or SSD** (FAT32 and exFAT do not support NTFS hardlinks, which Rusteus relies on for instant game staging).
- **Runtime**: [Microsoft Visual C++ 2015–2022 Redistributable (x64 and x86)](https://aka.ms/vs/17/release/vc_redist.x64.exe).
- **Input Hardware**: At least one gamepad (Xbox, PlayStation, Nintendo Switch, generic USB) or keyboard/mouse.

> [!IMPORTANT]
> **Antivirus Whitelisting**: Emulation tools (such as Goldberg Steam Emulator and ProtoInput raw input hooks) intercept system calls to separate controller inputs and trick games into believing multiple local instances are distinct LAN clients. Some antivirus utilities flag these as false positives. Always add the Rusteus-Coop folder to your antivirus exclusion list.

---

## 2. Quick Start: Launching Rusteus-Coop

1. Download the latest `rusteus-coop` release archive and extract it to a dedicated folder on your drive (e.g. `C:\Games\RusteusCoop\`).
2. Double-click **`nucleus-coop.exe`** to start the launcher.
3. On first startup, Rusteus performs an integrity check:
   - Validates that your drive supports NTFS hardlinks.
   - Checks that Windows Explorer shell folders (`Documents`) are intact.
   - Initialises the GPU-accelerated **Nefos UI** desktop interface.

---

## 3. Finding & Adding Games

Rusteus-Coop tracks games via **Handlers** (JavaScript configuration files created by the Splitscreen.Me community that describe how to isolate and run multiple instances of a game).

### Method A: Automatic Local Drive Scan
1. In the sidebar, select **Library**.
2. Click the **🔍 Scan Games** button in the top-right corner.
3. Rusteus will scan all connected internal and external drives for known game executables matching installed handlers.
4. Newly discovered games will appear as cover cards in your Library.

### Method B: Downloading Handlers from Splitscreen.Me Hub
If you have a game installed on your computer, but do not yet have its handler:
1. Select **Download Handlers (Hub)** from the sidebar navigation.
2. In the search box, type the name of the game (e.g. *Left 4 Dead 2*, *Borderlands 2*, *Risk of Rain 2*).
3. Click the game's card to open the **Handler Details Modal**.
4. Click **Install Handler**. Rusteus streams the archive, validates its SHA-256 signature, and extracts it to `./handlers/`.
5. Once downloaded, the card will display a green checkmark and automatically appear in your Library.

### Method C: Locating Missing Executables (`.exe Missing`)
If you downloaded a handler from the Hub but haven't specified where the game is installed on your PC:
1. The game's card in your Library will be dimmed with an **Uninstalled (.exe missing)** badge.
2. Click the green-outlined **📂 Locate .exe** button on the card.
3. In the native Windows file browser, navigate to your game's installation folder (e.g., `C:\Program Files (x86)\Steam\steamapps\common\Left 4 Dead 2\left4dead2.exe`) and select the `.exe`.
4. Rusteus verifies the executable and unlocks the game for configuration.

---

## 4. Configuring Splitscreen: Step-by-Step

Click on any installed game card in your Library and click **Configure Splitscreen**. This opens the **Setup Screen**:

```
┌────────────────────────────────────────────────────────────────────────┐
│  LEFT 4 DEAD 2  •  Total Playtime: 14h 22m  •  Last Played: 2 hrs ago   │
├────────────────────────────────────────────────────────────────────────┤
│  MONITOR LAYOUT:  [ 1x1 ] [ 2x1 Vertical ] [ 1x2 Horizontal ] [ 2x2 ]   │
├───────────────────────────────┬────────────────────────────────────────┤
│  PLAYER SLOTS (Drag & Drop)   │  CUSTOM HANDLER OPTIONS                │
│                               │                                        │
│  [ Player 1: Xbox Controller] │  Network Mode:     [ Offline / LAN ▾ ] │
│  [ Player 2: DualSense Pad  ] │  Audio Output:     [ Split Devices   ] │
│  [ Player 3: Keyboard/Mouse ] │  FOV Override:     [ 90°           ▾ ] │
│                               │                                        │
│  UNASSIGNED / VIRTUAL TRAY:   │  SAVE SLOTS (Ludusavi Isolation):      │
│  [ Virtual Gamepad 1 (Mock) ] │  [ Slot 1: Campaign Co-op ▾ ]          │
├───────────────────────────────┴────────────────────────────────────────┤
│                      [ 🚀 LAUNCH SPLITSCREEN ]                         │
└────────────────────────────────────────────────────────────────────────┘
```

### Step 1: Choose Your Monitor Layout
Click on the layout strip at the top to select your screen partition:
- **1x1 (Single)**: Fullscreen single-instance mode (ideal for testing mods or handlers).
- **2x1 (Side-by-Side)**: Splits the screen vertically down the middle (ideal for 2 players on ultra-wide or 16:9 monitors).
- **1x2 (Stacked)**: Splits the screen horizontally into top and bottom halves.
- **2x2 (Grid)**: Splits the screen into four equal quadrants for up to 4 players.
- **Multi-Monitor Span**: Spans players across distinct physical monitors (e.g., Player 1 on Monitor 1, Player 2 on Monitor 2).

### Step 2: Assign Controllers & Input Devices
- Connected gamepads appear in the input tray. Drag and drop each controller onto a player slot.
- **Keyboard & Mouse**: To assign keyboard and mouse to Player 1, check the "Assign Keyboard & Mouse" checkbox. If the game supports ProtoInput, multiple keyboards and mice can be assigned to different players.
- **Virtual / Mock Gamepads**: Don't have enough physical controllers plugged in? Click **Add Virtual Gamepad**. Rusteus will simulate a connected controller, allowing single developers or solo players to launch multi-player sessions for testing.
- **Ignored Controllers Tray**: If you have flight sticks, pedals, or virtual audio controllers plugged in that shouldn't participate in the game, move them to the **Ignored Controllers Tray**.

### Step 3: Configure Custom Handler Options
Many game handlers provide custom options. Rusteus displays these as intuitive controls:
- **Toggles**: Enable or disable features like *Skip Intro Videos*, *Mute Background Windows*, or *Custom FOV*.
- **Comboboxes**: For settings with more than 4 choices (such as *Target Monitor Refresh Rate* or *Rendering API*), click the combobox and type to filter with instant autocomplete.
- **Save Slots**: Select which isolated save profile to load. Rusteus uses integrated **Ludusavi savegame isolation** so Player 1 and Player 2 never overwrite each other's single-player campaign progress.

---

## 5. In-Game Controls & Features

Once you have configured your layout and controllers, click **🚀 Launch Splitscreen**.

### Launch Progression
1. **Instance Staging**: Rusteus creates isolated instance directories in milliseconds using NTFS hardlinks.
2. **Process Launching**: Game instances launch one by one with a stabilization delay to prevent Direct3D window conflicts.
3. **Singleton Mutex Killer**: Rusteus dynamically sweeps process handles, closing internal game singleton mutexes so multiple instances run concurrently.
4. **Window Tiling**: After a 1000ms stabilization period, window borders are stripped and instances are locked into their assigned screen coordinates.

### Essential In-Game Hotkeys

| Hotkey | Action | Description |
|---|---|---|
| **`VK_END`** or **`Ctrl + Q`** | **Toggle Input Lock** | Locks mouse cursors within each instance boundary and routes controller hooks. Press again to unlock cursor and return to desktop. |
| **`Ctrl + Shift + O`** or **Guide + Back** | **Open Couch OSD Overlay** | Summons the fullscreen, transparent In-Game Dashboard over the active game. |
| **`Ctrl + Alt + K`** | **Emergency Stop** | Immediately closes all active game instances and restores desktop state. |

---

## 6. Dynamic In-Game Couch OSD: Live Drop-In & Drop-Out

Rusteus introduces a groundbreaking feature: **Live In-Game OSD**. You no longer need to close the game when a friend arrives or has to leave!

### Summoning the Overlay
While playing, press **`Ctrl + Shift + O`** on your keyboard, or press the **Guide Button + Back** on your controller. A frosted, translucent overlay will appear smoothly over the game.

### Live Player Drop-In (Add a Player)
1. In the OSD menu, select **Add Player (Drop-In)**.
2. Plug in a new controller. The OSD automatically detects it.
3. Choose the new layout (e.g., transition from 2x1 Side-by-Side to 2x2 Grid).
4. Click **Confirm Drop-In**. Rusteus stages and launches the new instance, resizes the existing players' windows to the new layout without restarting them, and routes the new controller.

### Live Player Drop-Out (Remove a Player)
1. If Player 2 has to leave, open the OSD and select **Drop-Out Player 2**.
2. Player 2's instance terminates cleanly.
3. Surviving players automatically re-tile to fill the screen (e.g., Player 1 expands to fullscreen, or remaining players transition from 3-way split to 2-way split).

---

## 7. Desktop Shortcuts, Playnite & Steam Integration

Rusteus-Coop supports 1-click headless launching of pre-configured splitscreen sessions via desktop shortcuts, **Playnite**, and **Steam Big Picture**.

### Creating a Desktop Shortcut
1. Right-click any game card in your **Library** (or open the **Setup Screen** for a game) and click **Create Desktop Shortcut**.
2. Rusteus-Coop creates a native `.lnk` shortcut (or `.desktop` entry on Linux) on your Desktop using the game's icon.
3. Double-clicking the shortcut immediately launches that game's saved splitscreen profile without opening the main launcher window.

### Playnite & Steam Command-Line Integration
You can copy the exact launch command directly from the game's context menu (**Copy Launch Command (Playnite / Steam)**) or Setup Screen (**Copy Launch Cmd**):
```bash
nucleus-coop.exe --launch-profile "<game-guid>" [profile_number]
```
- **Aliases**: `--profile`, `--play-profile`
- When launched via Playnite or as a Non-Steam Game, Rusteus-Coop runs the session headlessly, enforces borderless viewports and hotkeys (`VK_END` / `Ctrl+Q`), and cleanly exits with code `0` once all game instances close so Playnite and Steam accurately track playtime.

---

## 8. Troubleshooting & FAQ

### Q1: The game crashes immediately on launch with "Access Denied" or fails to stage files.
- **Cause**: Rusteus or the game is located on a FAT32/exFAT drive, or Windows Developer Mode is required for non-admin symlinks.
- **Solution**: Move Rusteus-Coop and the game to an NTFS formatted partition. If you are not running as Administrator, enable **Developer Mode** in Windows Settings (*Settings > Update & Security > For Developers > Developer Mode*).

### Q2: Both controllers control the same player window.
- **Cause**: The game does not support native multi-controller separation, or ProtoInput/x360ce hooks were blocked by antivirus software.
- **Solution**: Open your antivirus utility (e.g. Windows Defender) and add an exclusion for the entire Rusteus-Coop directory. Ensure the game handler specifies `Game.Hook.XInputEnabled = true` or `Game.UseProtoInput = true`.

### Q3: Audio from all players plays out of the same speakers.
- **Cause**: Audio rerouting was not configured, or the secondary audio device was disconnected.
- **Solution**: In the Setup Screen, open Handler Options and set *Audio Routing* to *Per-Player Device*. Connect separate headphones or speakers for each player (e.g. HDMI TV audio for Player 1, USB headset for Player 2).

### Q4: My computer crashed mid-game, and now my Windows Documents folder points to a NucleusCoop folder!
- **Solution**: Rusteus includes an automatic startup registry recovery guard. Simply launch `nucleus-coop.exe` once; it will detect that the registry was left in a redirected state and restore your original Windows Documents path from the baseline backup in `./utils/backup/`.
