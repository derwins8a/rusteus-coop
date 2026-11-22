# Session Companion & Diagnostics Add-on

> **ID**: `org.splitscreenme.session-companion`  
> **Type**: JavaScript Add-on (`js-addon`)  
> **API Namespace**: `Plugins.sessionCompanion`  
> **Version**: 1.0.0  

## Overview
**Session Companion & Diagnostics** is an official example add-on demonstrating the extension ecosystem in **Rusteus-Coop** (Nucleus Co-op Rust Rewrite).

It solves a universal splitscreen challenge: **aspect ratio and FOV distortion**. When a screen is split vertically (e.g. 2 players side-by-side, 960x1080), standard 16:9 horizontal FOV pinches horizontal peripheral vision and causes motion sickness. This add-on provides mathematical aspect ratio normalization and FOV compensation, in addition to generating live HTML/JSON diagnostic telemetry dashboards.

---

## Capabilities & Exported JavaScript API

Game handlers and community scripts can access this add-on via `Plugins.sessionCompanion`:

```javascript
// Check if the extension is installed and active
if (typeof Plugins !== "undefined" && Plugins.sessionCompanion) {
    // 1. Calculate aspect ratio metrics
    var aspect = Plugins.sessionCompanion.calculateAspect(Context.Width, Context.Height);
    // aspect: { width, height, ratio, stringRatio: "8:9 (Vertical 2x1 Split)", isVertical: true }

    // 2. Calculate mathematically corrected Field of View (FOV)
    var fov = Plugins.sessionCompanion.calculateFov(90, Context.Width, Context.Height);
    // fov: { recommendedFov: 59, verticalFovDegrees: 58.7, fovScaleFactor: 0.66 }

    // 3. Convenience helper for Game.Play
    var opt = Plugins.sessionCompanion.optimizeGeometry(Context, 90);

    // 4. Generate an interactive HTML Diagnostic Report
    var html = Plugins.sessionCompanion.generateHtmlReport(Game.GameName);
}
```

---

## File Structure

```
org.splitscreenme.session-companion/
├── plugin.toml    # Extension manifest with metadata, capabilities, hooks & permissions
├── index.js       # Main runtime entrypoint exporting methods to Plugins.sessionCompanion
└── README.md      # Documentation and integration guide
```

---

## Packaging into a `.nucaddon` Archive

To distribute this add-on to other players or developers, create a standard ZIP archive of the contents and rename the extension to `.nucaddon`:

```powershell
Compress-Archive -Path "org.splitscreenme.session-companion\*" -DestinationPath "org.splitscreenme.session-companion.nucaddon"
```

Users can install the resulting `.nucaddon` archive directly in the Rusteus-Coop UI via **Extensions & Add-ons -> Install Extension**.
