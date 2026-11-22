/**
 * ==============================================================================
 * Rusteus-Coop JavaScript Add-on: Session Companion & Diagnostics
 * ==============================================================================
 *
 * This add-on demonstrates how to create pure JavaScript extensions for Rusteus-Coop.
 *
 * HOW ADD-ONS WORK IN RUSTEUS-COOP:
 * 1. The engine scans `addons/` and `%APPDATA%\Nucleus Coop\addons\` for folders
 *    containing `plugin.toml` or `addon.toml`.
 * 2. When an add-on is active, its entrypoint script is loaded and executed inside
 *    the Boa ECMAScript engine.
 * 3. The code runs inside an isolated closure:
 *    `(function(exports, Plugins) { ... })(exports, globalThis.Plugins);`
 * 4. Anything attached to `exports` is automatically available to game handlers
 *    and other scripts via `Plugins.<js_namespace>`.
 *    For this add-on, it is accessible as `Plugins.sessionCompanion`.
 *
 * ==============================================================================
 */

// Version and metadata
exports.version = "1.0.0";
exports.author = "Rusteus-Coop Community";
exports.name = "Session Companion & Diagnostics";

// Internal in-memory state
var sessionLog = [];
var instanceRegistry = [];

/**
 * Calculates Greatest Common Divisor using Euclidean algorithm.
 */
function gcd(a, b) {
    a = Math.abs(Math.round(a));
    b = Math.abs(Math.round(b));
    while (b !== 0) {
        var t = b;
        b = a % b;
        a = t;
    }
    return a;
}

/**
 * Calculates aspect ratio metrics and classifications for a given viewport resolution.
 *
 * @param {number} width - Viewport width in pixels.
 * @param {number} height - Viewport height in pixels.
 * @returns {object} Aspect ratio metadata including ratio value, simplified fraction string, and classification.
 */
exports.calculateAspect = function(width, height) {
    var w = Math.max(1, Math.round(width || 1920));
    var h = Math.max(1, Math.round(height || 1080));
    var ratio = w / h;
    var divisor = gcd(w, h);
    var simplifiedW = Math.round(w / divisor);
    var simplifiedH = Math.round(h / divisor);

    // Normalize common aspect ratios
    var stringRatio = simplifiedW + ":" + simplifiedH;
    if (Math.abs(ratio - (16 / 9)) < 0.02) {
        stringRatio = "16:9";
    } else if (Math.abs(ratio - (21 / 9)) < 0.03) {
        stringRatio = "21:9 (Ultrawide)";
    } else if (Math.abs(ratio - (32 / 9)) < 0.05) {
        stringRatio = "32:9 (Super Ultrawide)";
    } else if (Math.abs(ratio - (4 / 3)) < 0.02) {
        stringRatio = "4:3";
    } else if (Math.abs(ratio - (8 / 9)) < 0.03) {
        stringRatio = "8:9 (Vertical 2x1 Split)";
    } else if (Math.abs(ratio - (32 / 18)) < 0.03) {
        stringRatio = "16:9 (Quad 2x2 Split)";
    }

    return {
        width: w,
        height: h,
        ratio: Math.round(ratio * 1000) / 1000,
        stringRatio: stringRatio,
        isStandard16x9: Math.abs(ratio - (16 / 9)) < 0.02,
        isUltrawide: ratio > 2.0,
        isVertical: ratio < 1.0,
        isSquareOrNarrow: ratio >= 1.0 && ratio < 1.4
    };
};

/**
 * Calculates optimal Field of View (FOV) correction for splitscreen viewports.
 *
 * In splitscreen gaming, non-standard aspect ratios (such as vertical 2x1 splits or stacked 1x2 splits)
 * severely compress horizontal peripheral vision if using the game's default 16:9 horizontal FOV.
 * This function calculates the mathematically corrected horizontal/vertical FOV to preserve
 * true vertical viewing angle without camera distortion.
 *
 * @param {number} baseHorizontalFov - Default 16:9 horizontal FOV (e.g. 75, 90).
 * @param {number} width - Viewport width in pixels.
 * @param {number} height - Viewport height in pixels.
 * @returns {object} Recommended FOV values and scaling factor.
 */
exports.calculateFov = function(baseHorizontalFov, width, height) {
    var baseH = baseHorizontalFov || 90;
    var w = Math.max(1, width || 1920);
    var h = Math.max(1, height || 1080);
    var currentAspect = w / h;
    var standardAspect = 16.0 / 9.0;

    // Convert base horizontal FOV in 16:9 to vertical FOV
    var baseHRad = (baseH * Math.PI) / 180.0;
    var verticalFovRad = 2.0 * Math.atan(Math.tan(baseHRad / 2.0) / standardAspect);
    var verticalFovDeg = (verticalFovRad * 180.0) / Math.PI;

    // Calculate adjusted horizontal FOV for the current aspect ratio
    var adjustedHRad = 2.0 * Math.atan(Math.tan(verticalFovRad / 2.0) * currentAspect);
    var adjustedHDeg = (adjustedHRad * 180.0) / Math.PI;

    // Clamp to practical gaming boundaries (45 to 135 degrees)
    var recommendedFov = Math.max(45, Math.min(135, Math.round(adjustedHDeg)));

    return {
        baseHorizontalFov: baseH,
        currentAspect: Math.round(currentAspect * 100) / 100,
        verticalFovDegrees: Math.round(verticalFovDeg * 10) / 10,
        adjustedHorizontalFov: Math.round(adjustedHDeg * 10) / 10,
        recommendedFov: recommendedFov,
        fovScaleFactor: Math.round((adjustedHDeg / baseH) * 100) / 100
    };
};

/**
 * Logs a session telemetry event with timestamp and category tag.
 *
 * @param {string} tag - Event classification (e.g. "GEOMETRY", "STAGING", "LIFECYCLE").
 * @param {string} message - Informational message.
 */
exports.logSessionEvent = function(tag, message) {
    var entry = {
        timestamp: new Date().toISOString ? new Date().toISOString() : "T" + Date.now(),
        tag: tag || "INFO",
        message: message || ""
    };
    sessionLog.push(entry);
    return entry;
};

/**
 * Returns all recorded telemetry entries.
 */
exports.getTelemetry = function() {
    return sessionLog.slice(0);
};

/**
 * Optimizes instance geometry and FOV for a game handler Context object.
 * Game handlers can call this convenience helper directly inside `Game.Play`:
 *
 * Example in handler.js:
 * ```javascript
 * Game.Play = function() {
 *     if (typeof Plugins !== "undefined" && Plugins.sessionCompanion) {
 *         var opt = Plugins.sessionCompanion.optimizeGeometry(Context, 90);
 *         // Use opt.fov.recommendedFov or opt.aspect.stringRatio
 *     }
 * };
 * ```
 *
 * @param {object} context - Nucleus Co-op `Context` object.
 * @param {number} baseFov - Game's standard 16:9 horizontal FOV (default 90).
 * @returns {object} Calculated optimization results.
 */
exports.optimizeGeometry = function(context, baseFov) {
    var w = (context && typeof context.Width === "number") ? context.Width : 1920;
    var h = (context && typeof context.Height === "number") ? context.Height : 1080;
    var pId = (context && typeof context.PlayerID === "number") ? context.PlayerID : 0;
    var nick = (context && context.Nickname) ? context.Nickname : ("Player " + (pId + 1));

    var aspect = exports.calculateAspect(w, h);
    var fov = exports.calculateFov(baseFov || 90, w, h);

    var info = {
        playerIndex: pId,
        nickname: nick,
        width: w,
        height: h,
        aspect: aspect,
        fov: fov,
        instanceFolder: (context && context.Folder) ? context.Folder : ""
    };

    instanceRegistry.push(info);

    exports.logSessionEvent("GEOMETRY", "Optimized Player " + pId + " (" + nick + ") viewport: " +
        w + "x" + h + " (" + aspect.stringRatio + ") -> Recommended FOV: " + fov.recommendedFov);

    return info;
};

/**
 * Generates an interactive, modern HTML Diagnostic Dashboard report.
 *
 * @param {string} gameName - Display name of the active game.
 * @param {Array} instances - List of instance geometry and configuration objects.
 * @returns {string} Complete HTML document string.
 */
exports.generateHtmlReport = function(gameName, instances) {
    var gName = gameName || "Multi-Instance Co-op Session";
    var list = instances || instanceRegistry;
    var count = list.length;
    var timestamp = new Date().toISOString ? new Date().toISOString() : "Session";

    var html = '<!DOCTYPE html>\n<html lang="en">\n<head>\n' +
        '<meta charset="UTF-8">\n' +
        '<title>Rusteus-Coop Session Diagnostics - ' + gName + '</title>\n' +
        '<style>\n' +
        '  :root { --bg-dark: #0f1117; --card-bg: #181b26; --card-border: #262a3b; --accent: #6366f1; --accent-glow: #818cf8; --text-main: #f1f5f9; --text-muted: #94a3b8; --success: #22c55e; --warning: #f59e0b; }\n' +
        '  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; background: var(--bg-dark); color: var(--text-main); margin: 0; padding: 28px; line-height: 1.5; }\n' +
        '  .container { max-width: 1080px; margin: 0 auto; }\n' +
        '  .header { display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--card-border); padding-bottom: 20px; margin-bottom: 24px; }\n' +
        '  .brand { display: flex; align-items: center; gap: 14px; }\n' +
        '  .badge { display: inline-block; padding: 4px 10px; border-radius: 9999px; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; }\n' +
        '  .badge-accent { background: rgba(99, 102, 241, 0.18); color: var(--accent-glow); border: 1px solid rgba(99, 102, 241, 0.35); }\n' +
        '  .badge-success { background: rgba(34, 197, 94, 0.18); color: #4ade80; border: 1px solid rgba(34, 197, 94, 0.35); }\n' +
        '  .badge-warning { background: rgba(245, 158, 11, 0.18); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.35); }\n' +
        '  .title { font-size: 24px; font-weight: 700; margin: 0; }\n' +
        '  .meta { color: var(--text-muted); font-size: 13px; margin-top: 4px; }\n' +
        '  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 18px; margin-bottom: 28px; }\n' +
        '  .card { background: var(--card-bg); border: 1px solid var(--card-border); border-radius: 12px; padding: 18px; box-shadow: 0 4px 14px rgba(0,0,0,0.3); }\n' +
        '  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }\n' +
        '  .card-title { font-size: 15px; font-weight: 600; margin: 0; color: var(--text-main); }\n' +
        '  .stat-val { font-size: 28px; font-weight: 700; color: var(--accent-glow); margin: 6px 0; }\n' +
        '  .stat-label { font-size: 12px; color: var(--text-muted); }\n' +
        '  table { width: 100%; border-collapse: collapse; margin-top: 12px; font-size: 13px; }\n' +
        '  th { text-align: left; padding: 10px 12px; background: rgba(255,255,255,0.03); color: var(--text-muted); font-weight: 600; border-bottom: 1px solid var(--card-border); }\n' +
        '  td { padding: 12px; border-bottom: 1px solid rgba(255,255,255,0.04); }\n' +
        '  .log-box { background: rgba(0,0,0,0.35); border: 1px solid var(--card-border); border-radius: 8px; padding: 14px; font-family: ui-monospace, SFMono-Regular, Consolas, monospace; font-size: 12px; max-height: 220px; overflow-y: auto; color: #cbd5e1; }\n' +
        '  .log-line { margin-bottom: 4px; }\n' +
        '  .log-tag { color: var(--accent-glow); font-weight: bold; }\n' +
        '</style>\n</head>\n<body>\n' +
        '<div class="container">\n' +
        '  <div class="header">\n' +
        '    <div class="brand">\n' +
        '      <div>\n' +
        '        <h1 class="title">Rusteus-Coop Session Diagnostics</h1>\n' +
        '        <div class="meta">Game: <strong>' + gName + '</strong> &bull; Generated: ' + timestamp + ' &bull; Extension: Session Companion v1.0.0</div>\n' +
        '      </div>\n' +
        '    </div>\n' +
        '    <div>\n' +
        '      <span class="badge badge-accent">' + count + ' Active Instance' + (count === 1 ? '' : 's') + '</span>\n' +
        '    </div>\n' +
        '  </div>\n';

    // Summary cards
    html += '  <div class="grid">\n' +
        '    <div class="card">\n' +
        '      <div class="card-header"><span class="card-title">Instance Staging</span><span class="badge badge-success">Isolated</span></div>\n' +
        '      <div class="stat-val">' + count + '</div>\n' +
        '      <div class="stat-label">Isolated player environments & save sandboxes</div>\n' +
        '    </div>\n' +
        '    <div class="card">\n' +
        '      <div class="card-header"><span class="card-title">FOV Optimization</span><span class="badge badge-accent">Auto-Compensated</span></div>\n' +
        '      <div class="stat-val">Aspect Adaptive</div>\n' +
        '      <div class="stat-label">Corrected vertical-to-horizontal peripheral view</div>\n' +
        '    </div>\n' +
        '    <div class="card">\n' +
        '      <div class="card-header"><span class="card-title">Mutex Safety</span><span class="badge badge-success">Dynamic Kernel</span></div>\n' +
        '      <div class="stat-val">NtQuery Mutant</div>\n' +
        '      <div class="stat-label">Automatic multi-instance handle termination</div>\n' +
        '    </div>\n' +
        '  </div>\n';

    // Instance geometry table
    html += '  <div class="card" style="margin-bottom: 28px;">\n' +
        '    <div class="card-header"><span class="card-title">Player Viewport Geometry & FOV Matrix</span></div>\n' +
        '    <table>\n' +
        '      <thead><tr><th>Slot</th><th>Player</th><th>Resolution</th><th>Aspect Ratio</th><th>Recommended FOV</th><th>Status</th></tr></thead>\n' +
        '      <tbody>\n';

    if (list.length === 0) {
        html += '        <tr><td colspan="6" style="text-align: center; color: var(--text-muted);">No active instances recorded yet.</td></tr>\n';
    } else {
        for (var i = 0; i < list.length; i++) {
            var item = list[i];
            var asp = item.aspect || exports.calculateAspect(item.width, item.height);
            var fv = item.fov || exports.calculateFov(90, item.width, item.height);
            var statusBadge = asp.isVertical ?
                '<span class="badge badge-warning">Vertical Split</span>' :
                (asp.isUltrawide ? '<span class="badge badge-accent">Ultrawide</span>' : '<span class="badge badge-success">Standard</span>');

            html += '        <tr>\n' +
                '          <td><strong>Player ' + item.playerIndex + '</strong></td>\n' +
                '          <td>' + item.nickname + '</td>\n' +
                '          <td>' + item.width + ' &times; ' + item.height + '</td>\n' +
                '          <td>' + asp.stringRatio + ' (' + asp.ratio + ')</td>\n' +
                '          <td><strong>' + fv.recommendedFov + '&deg;</strong> <span style="color: var(--text-muted); font-size: 11px;">(Base: ' + fv.baseHorizontalFov + '&deg;)</span></td>\n' +
                '          <td>' + statusBadge + '</td>\n' +
                '        </tr>\n';
        }
    }

    html += '      </tbody>\n' +
        '    </table>\n' +
        '  </div>\n';

    // Telemetry log section
    html += '  <div class="card">\n' +
        '    <div class="card-header"><span class="card-title">Session Telemetry Log</span></div>\n' +
        '    <div class="log-box">\n';

    if (sessionLog.length === 0) {
        html += '      <div class="log-line">No events recorded.</div>\n';
    } else {
        for (var j = 0; j < sessionLog.length; j++) {
            var ev = sessionLog[j];
            html += '      <div class="log-line"><span style="color: var(--text-muted);">[' + ev.timestamp + ']</span> <span class="log-tag">[' + ev.tag + ']</span> ' + ev.message + '</div>\n';
        }
    }

    html += '    </div>\n' +
        '  </div>\n' +
        '</div>\n' +
        '</body>\n</html>';

    return html;
};

// ==============================================================================
// Multi-Instance Lifecycle Hooks
// ==============================================================================
// These callbacks correspond to the hook kinds declared in `plugin.toml`:
// "pre_prepare", "post_stage", "pre_launch", "post_window", "on_stop".

exports.onPrePrepare = function(playerCount) {
    instanceRegistry = [];
    exports.logSessionEvent("LIFECYCLE", "PrePrepare hook invoked for " + playerCount + " players.");
};

exports.onPostStage = function(instanceDir, playerIndex) {
    exports.logSessionEvent("STAGING", "PostStage hook invoked for Player " + playerIndex + " at " + instanceDir);
};

exports.onPreLaunch = function(instanceDir, playerIndex) {
    exports.logSessionEvent("LIFECYCLE", "PreLaunch hook invoked for Player " + playerIndex);
};

exports.onPostWindow = function(instanceDir, playerIndex) {
    exports.logSessionEvent("WINDOW", "PostWindow hook invoked for Player " + playerIndex + ": Window mapped and positioned.");
};

exports.onStop = function() {
    exports.logSessionEvent("LIFECYCLE", "OnStop hook invoked: Terminating session and releasing instance resources.");
};

// Initialization message logged into Boa runtime
exports.logSessionEvent("INITIALIZE", "Session Companion & Diagnostics extension v1.0.0 loaded successfully.");
