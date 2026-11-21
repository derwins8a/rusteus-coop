// Sample SplitscreenMe Game Handler demonstrating Arbitrary DLL Injection,
// Export Execution, Named Pipe IPC, and Direct In-Process Memory Manipulation (Cheat Engine Parity).

Game.ExecutableName = "sample_game.exe";
Game.SteamID = "480";
Game.GUID = "sample-game-dll-injection";
Game.GameName = "Sample Game with Injected DLLs";
Game.MaxPlayers = 4;
Game.SupportsPositioning = true;
Game.SymlinkGame = true;

// =============================================================================
// 1. Declarative DLL Injections (Game.InjectedDlls)
// =============================================================================
Game.InjectedDlls = [
    // Injected at startup while process is suspended to patch engine initialization
    {
        Path: "mods/early_engine_patch.dll",
        Timing: "AtStartupSuspended",
        Required: true,
        TargetInstances: "All",
        ExportName: "InitPatch",
        ExportArgument: "disable_intro=1"
    },
    // Injected once the window is created and bounds are applied
    {
        Path: "mods/fov_fix.dll",
        Timing: "OnWindowCreated",
        DelayMs: 250,
        TargetInstances: "All",
        ExportName: "SetFieldOfView",
        ExportArgument: "95",
        EjectOnClose: true
    },
    // Injected into all instances except player 0 with Named Pipe IPC
    {
        Path: "mods/instance_telemetry.dll",
        Timing: "PostStartup",
        TargetInstances: "ExceptFirst",
        IpcChannel: "NamedPipe",
        PipeName: "\\\\.\\pipe\\nucleus_telemetry",
        Required: false
    }
];

// Or configure programmatically via helper:
Game.AddInjectedDll("mods/custom_input.dll", {
    Timing: "OnWindowCreated",
    DelayMs: 500,
    TargetInstances: [0, 1]
});

// =============================================================================
// 2. Dynamic Lifecycle Hook & Cheat Engine Memory Manipulation (Game.Play)
// =============================================================================
Game.Play = function() {
    // Environment variables automatically provided by Nucleus:
    // NUCLEUS_INSTANCE_INDEX = 0, 1, 2...
    // NUCLEUS_PLAYER_INDEX   = 0, 1, 2...
    // NUCLEUS_PLAYER_ID      = 1, 2, 3...
    // NUCLEUS_PIPE_NAME      = \\.\pipe\nucleus_instance_{index}
    // NUCLEUS_SHARED_MEM_NAME= Local\nucleus_instance_{index}

    // Example 1: Dynamic DLL Injection
    if (Context.Options && Context.Options["EnableMod"] === "Yes") {
        Context.InjectDll("mods/optional_feature.dll", {
            ExportName: "SetupFeature",
            ExportArgument: JSON.stringify({ playerIndex: Context.PlayerIndex })
        });
    }

    // Example 2: Cheat Engine Parity - AOB (Array of Bytes) Signature Scan
    // Locate the player health instruction: pattern "89 45 FC 8B 45" with wildcard mask
    var matches = Context.ScanMemory("89 45 ?? 8B 45", "xx?xx", null);
    if (matches && matches.length > 0) {
        var instructionAddr = matches[0];
        // Read current instruction bytes
        var bytes = Context.ReadMemoryBytes(instructionAddr, 5);

        // NOP out the health deduction instruction: [0x90, 0x90, 0x90, 0x90, 0x90]
        Context.WriteMemoryBytes(instructionAddr, [0x90, 0x90, 0x90, 0x90, 0x90]);
    }

    // Example 3: Multi-level Pointer Chain Resolution & Value Freezing
    // Base game module pointer + offsets [0x48, 0x1C, 0x30]
    var baseModuleAddr = 0x00400000;
    var playerHealthAddr = Context.ResolvePointer(baseModuleAddr, [0x48, 0x1C, 0x30]);

    if (playerHealthAddr !== 0) {
        // Read current health value
        var currentHealth = Context.ReadMemoryInt(playerHealthAddr);

        // Write infinite health: 9999
        Context.WriteMemoryInt(playerHealthAddr, 9999);

        // Continuously freeze health to 9999 (enforced automatically on every session heartbeat tick)
        var freezeId = Context.FreezeMemoryInt(playerHealthAddr, 9999);
    }

    // Example 4: Bidirectional Inter-Process Communication
    // Send a configuration message to the injected DLL via its Named Pipe
    Context.SendDllMessage(JSON.stringify({
        command: "SetPlayerNickname",
        nickname: Context.Nickname,
        instanceIndex: Context.PlayerIndex,
        viewport: {
            x: Context.PosX,
            y: Context.PosY,
            width: Context.Width,
            height: Context.Height
        }
    }));
};
