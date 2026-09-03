//
//  mac_break_lock.swift
//  Native macOS Break Screen Lock for Pomotroid Shield
//
//  Features:
//  - Fully synchronized with Pomotroid active theme (background, dials, text, accent)
//  - Simple, minimalist circular progress dial matching the main Pomotroid app
//  - Full-screen hardware shielding on all connected monitors
//  - Real-time countdown synced with Pomotroid SQLite DB & log stream
//  - Dynamic wall-clock unlock time (e.g. "Auto-unlocks at 5:17 PM")
//  - Gentle mindfulness breathing prompts
//  - Safety ESC key handler
//  - Instant preview mode (--preview)
//

import Cocoa
import SwiftUI
import SQLite3

// ============================================================================
// Color Extension & Theme Engine
// ============================================================================

extension Color {
    init(hex: String) {
        var clean = hex.trimmingCharacters(in: .whitespacesAndNewlines).replacingOccurrences(of: "#", with: "")
        if clean.count == 6 { clean.append("FF") }
        var rgbValue: UInt64 = 0
        Scanner(string: clean).scanHexInt64(&rgbValue)
        let r = Double((rgbValue & 0xFF000000) >> 24) / 255.0
        let g = Double((rgbValue & 0x00FF0000) >> 16) / 255.0
        let b = Double((rgbValue & 0x0000FF00) >> 8) / 255.0
        let a = Double(rgbValue & 0x000000FF) / 255.0
        self.init(.sRGB, red: r, green: g, blue: b, opacity: a)
    }

    var nsColor: NSColor {
        return NSColor(self)
    }
}

struct ThemeColors {
    var background: Color = Color(hex: "#2f384b")
    var backgroundLight: Color = Color(hex: "#3d4457")
    var foreground: Color = Color(hex: "#f6f2eb")
    var foregroundDarker: Color = Color(hex: "#c0c9da")
    var shortRound: Color = Color(hex: "#05ec8c")
    var focusRound: Color = Color(hex: "#ff4e4d")
    var longRound: Color = Color(hex: "#0bbddb")
    var accent: Color = Color(hex: "#05ec8c")
}

func loadThemeColors(themeName: String) -> ThemeColors {
    var tc = ThemeColors()
    let slug = themeName.lowercased().replacingOccurrences(of: " ", with: "-")

    let paths = [
        NSString(string: "~/Library/Application Support/com.abirislam.pomotroid-shield/themes/\(slug).json").expandingTildeInPath,
        "/Users/abir/Downloads/pomotroid-with-mac-system-bridge/static/themes/\(slug).json",
        NSString(string: "~/Library/Application Support/com.splode.pomotroid/themes/\(slug).json").expandingTildeInPath
    ]

    for path in paths {
        if FileManager.default.fileExists(atPath: path),
           let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
           let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
           let colors = json["colors"] as? [String: String] {

            if let bg = colors["--color-background"] { tc.background = Color(hex: bg) }
            if let bgl = colors["--color-background-light"] { tc.backgroundLight = Color(hex: bgl) }
            if let fg = colors["--color-foreground"] { tc.foreground = Color(hex: fg) }
            if let fgd = colors["--color-foreground-darker"] { tc.foregroundDarker = Color(hex: fgd) }
            if let sr = colors["--color-short-round"] { tc.shortRound = Color(hex: sr) }
            if let fr = colors["--color-focus-round"] { tc.focusRound = Color(hex: fr) }
            if let lr = colors["--color-long-round"] { tc.longRound = Color(hex: lr) }
            if let acc = colors["--color-accent"] { tc.accent = Color(hex: acc) }
            break
        }
    }

    return tc
}

func fetchActiveTheme(db: OpaquePointer?) -> ThemeColors {
    guard let db = db else {
        return loadThemeColors(themeName: "Pomotroid")
    }

    var themeMode = "auto"
    var themeDark = "Pomotroid"
    var themeLight = "Pomotroid Light"

    var stmt: OpaquePointer? = nil
    if sqlite3_prepare_v2(db, "SELECT key, value FROM settings WHERE key LIKE 'theme_%'", -1, &stmt, nil) == SQLITE_OK {
        while sqlite3_step(stmt) == SQLITE_ROW {
            guard let kStr = sqlite3_column_text(stmt, 0),
                  let vStr = sqlite3_column_text(stmt, 1) else { continue }
            let key = String(cString: kStr)
            let val = String(cString: vStr)
            if key == "theme_mode" { themeMode = val }
            else if key == "theme_dark" { themeDark = val }
            else if key == "theme_light" { themeLight = val }
        }
        sqlite3_finalize(stmt)
    }

    let isDarkOS: Bool
    if #available(macOS 10.14, *) {
        isDarkOS = NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
    } else {
        isDarkOS = true
    }

    let activeName: String
    if themeMode == "dark" {
        activeName = themeDark
    } else if themeMode == "light" {
        activeName = themeLight
    } else {
        activeName = isDarkOS ? themeDark : themeLight
    }

    return loadThemeColors(themeName: activeName)
}

// ============================================================================
// Timer State & Process Discovery
// ============================================================================

struct PomotroidTimerState {
    var isRunning: Bool
    var roundType: String
    var totalSecs: Int
    var elapsedSecs: Int
    var remainingSecs: Int
    var currentRound: Int = 1
    var totalRounds: Int = 4
    var todayRounds: Int = 0
    var todayFocusMins: Int = 0
}

func parseLogTimestamp(from line: String) -> Date? {
    guard let r = line.range(of: "\\[(\\d{4}-\\d{2}-\\d{2})\\]\\[(\\d{2}:\\d{2}:\\d{2})\\]", options: .regularExpression) else {
        return nil
    }
    let sub = String(line[r])
    let clean = sub.replacingOccurrences(of: "[", with: "").replacingOccurrences(of: "]", with: " ").trimmingCharacters(in: .whitespaces)
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
    formatter.timeZone = TimeZone(secondsFromGMT: 0)
    return formatter.date(from: clean)
}

func isPomotroidRunning() -> Bool {
    let shieldApps = NSRunningApplication.runningApplications(withBundleIdentifier: "com.abirislam.pomotroid-shield")
    if !shieldApps.isEmpty { return true }
    let originalApps = NSRunningApplication.runningApplications(withBundleIdentifier: "com.splode.pomotroid")
    if !originalApps.isEmpty { return true }

    for app in NSWorkspace.shared.runningApplications {
        if let name = app.localizedName?.lowercased(), name.contains("pomotroid") { return true }
        if let exe = app.executableURL?.lastPathComponent.lowercased(), exe.contains("pomotroid") { return true }
    }
    return false
}

func fetchPomotroidStateAndTheme() -> (PomotroidTimerState, ThemeColors) {
    var targetRounds = 4
    var todayRounds = 0
    var todayFocusMins = 0
    var dbRoundType = "short-break"
    var dbDuration = 300
    var sStart: Int64 = Int64(Date().timeIntervalSince1970)
    var sEnd: Int64? = nil
    var themeColors = ThemeColors()

    var dbPath = NSString(string: "~/Library/Application Support/com.abirislam.pomotroid-shield/pomotroid_shield.db").expandingTildeInPath
    if !FileManager.default.fileExists(atPath: dbPath) {
        dbPath = NSString(string: "~/Library/Application Support/com.splode.pomotroid/pomotroid.db").expandingTildeInPath
    }

    if FileManager.default.fileExists(atPath: dbPath) {
        var db: OpaquePointer? = nil
        if sqlite3_open_v2(dbPath, &db, SQLITE_OPEN_READONLY, nil) == SQLITE_OK {
            themeColors = fetchActiveTheme(db: db)

            var stmt: OpaquePointer? = nil
            if sqlite3_prepare_v2(db, "SELECT value FROM settings WHERE key = 'long_break_interval'", -1, &stmt, nil) == SQLITE_OK {
                if sqlite3_step(stmt) == SQLITE_ROW, let cStr = sqlite3_column_text(stmt, 0) {
                    if let n = Int(String(cString: cStr)) { targetRounds = n }
                }
                sqlite3_finalize(stmt)
            }

            let startOfToday = Int64(Calendar.current.startOfDay(for: Date()).timeIntervalSince1970)
            if sqlite3_prepare_v2(db, "SELECT COUNT(*), COALESCE(SUM(duration_secs), 0) FROM sessions WHERE started_at >= ? AND round_type = 'work' AND completed = 1", -1, &stmt, nil) == SQLITE_OK {
                sqlite3_bind_int64(stmt, 1, startOfToday)
                if sqlite3_step(stmt) == SQLITE_ROW {
                    todayRounds = Int(sqlite3_column_int(stmt, 0))
                    todayFocusMins = Int(sqlite3_column_int(stmt, 1)) / 60
                }
                sqlite3_finalize(stmt)
            }

            if sqlite3_prepare_v2(db, "SELECT started_at, ended_at, round_type, duration_secs FROM sessions ORDER BY id DESC LIMIT 1", -1, &stmt, nil) == SQLITE_OK {
                if sqlite3_step(stmt) == SQLITE_ROW {
                    sStart = sqlite3_column_int64(stmt, 0)
                    if sqlite3_column_type(stmt, 1) != SQLITE_NULL {
                        sEnd = sqlite3_column_int64(stmt, 1)
                    }
                    if let cType = sqlite3_column_text(stmt, 2) {
                        dbRoundType = String(cString: cType)
                    }
                    let dur = Int(sqlite3_column_int(stmt, 3))
                    if dur > 0 { dbDuration = dur }
                }
                sqlite3_finalize(stmt)
            }

            sqlite3_close(db)
        }
    } else {
        themeColors = loadThemeColors(themeName: "Pomotroid")
    }

    if !isPomotroidRunning() {
        return (PomotroidTimerState(isRunning: false, roundType: "idle", totalSecs: 300, elapsedSecs: 0, remainingSecs: 0), themeColors)
    }

    let totalSecs = dbDuration
    var elapsedSecs = 0
    var isRunning = true
    var roundType = dbRoundType
    var curRound = 1

    var logPath = NSString(string: "~/Library/Logs/com.abirislam.pomotroid-shield/Pomotroid Shield.log").expandingTildeInPath
    if !FileManager.default.fileExists(atPath: logPath) {
        let alt = NSString(string: "~/Library/Logs/com.abirislam.pomotroid-shield/Pomotroid.log").expandingTildeInPath
        if FileManager.default.fileExists(atPath: alt) {
            logPath = alt
        } else {
            logPath = NSString(string: "~/Library/Logs/com.splode.pomotroid/Pomotroid.log").expandingTildeInPath
        }
    }

    if let logContent = try? String(contentsOfFile: logPath, encoding: .utf8) {
        let lines = logContent.components(separatedBy: .newlines)
        var boundaryIdx = 0
        for (i, line) in lines.enumerated() {
            if line.contains("[timer] reset") || line.contains("round complete type=long-break") {
                boundaryIdx = i
            }
        }
        var completedInCycle = 0
        for line in lines[boundaryIdx...] {
            if line.contains("[timer] reset") || line.contains("round complete type=long-break") {
                completedInCycle = 0
            } else if line.contains("round complete type=work") {
                completedInCycle += 1
            }
        }

        if roundType == "long-break" {
            curRound = targetRounds
        } else if roundType == "short-break" {
            curRound = max(1, completedInCycle)
        } else {
            curRound = (completedInCycle % targetRounds) + 1
        }

        let recent = lines.suffix(300).reversed()
        for line in recent {
            if line.contains("[timer]") {
                if line.contains("[timer] idle") || line.contains("[timer] reset") || line.contains("round complete") {
                    roundType = "idle"
                    isRunning = false
                    elapsedSecs = 0
                    break
                }
                if let r = line.range(of: "\\[timer\\] paused elapsed=(\\d+)s", options: .regularExpression) {
                    let sub = String(line[r])
                    let digits = sub.filter { "0123456789".contains($0) }
                    isRunning = false
                    elapsedSecs = min(totalSecs, Int(digits) ?? 0)
                    break
                }
                if let r = line.range(of: "\\[timer\\] resumed elapsed=(\\d+)s", options: .regularExpression) {
                    let sub = String(line[r])
                    let digits = sub.filter { "0123456789".contains($0) }
                    let base = Int(digits) ?? 0
                    let delta = parseLogTimestamp(from: line).map { max(0, Int(Date().timeIntervalSince($0))) } ?? 0
                    isRunning = true
                    elapsedSecs = min(totalSecs, base + delta)
                    break
                }
                if line.contains("[timer] started") || line.contains("[timer] auto-starting") {
                    let delta = parseLogTimestamp(from: line).map { max(0, Int(Date().timeIntervalSince($0))) } ?? 0
                    isRunning = true
                    elapsedSecs = min(totalSecs, delta)
                    break
                }
            }
        }
    } else if sEnd == nil && sStart > 0 {
        elapsedSecs = max(0, min(totalSecs, Int(Int64(Date().timeIntervalSince1970) - sStart)))
    }

    let remainingSecs = max(0, totalSecs - elapsedSecs)
    let state = PomotroidTimerState(
        isRunning: isRunning,
        roundType: roundType,
        totalSecs: totalSecs,
        elapsedSecs: elapsedSecs,
        remainingSecs: remainingSecs,
        currentRound: curRound,
        totalRounds: targetRounds,
        todayRounds: todayRounds,
        todayFocusMins: todayFocusMins
    )

    return (state, themeColors)
}

// ============================================================================
// Observable View Model
// ============================================================================

class BreakLockViewModel: ObservableObject {
    @Published var state: PomotroidTimerState
    @Published var theme: ThemeColors
    @Published var localTimeStr: String = ""
    @Published var breathGuide: String = "Inhale slowly..."
    @Published var breathScale: CGFloat = 1.0
    let isPreview: Bool

    private var gcdTimer: DispatchSourceTimer?
    private var breathTimer: Timer?
    private var breathIndex: Int = 0
    private var previewElapsed: Double = 0.0

    let breathCycles: [(text: String, scale: CGFloat)] = [
        ("Inhale gently...", 1.05),
        ("Hold...", 1.05),
        ("Exhale slowly...", 0.95),
        ("Rest and release...", 1.0)
    ]

    init(initialState: PomotroidTimerState, theme: ThemeColors, isPreview: Bool = false) {
        self.state = initialState
        self.theme = theme
        self.isPreview = isPreview
        updateClock()
        startTimers()
    }

    func updateClock() {
        let formatter = DateFormatter()
        formatter.dateFormat = "h:mm a"
        localTimeStr = formatter.string(from: Date())
    }

    func startTimers() {
        let source = DispatchSource.makeTimerSource(queue: .main)
        source.schedule(deadline: .now(), repeating: .milliseconds(250), leeway: .milliseconds(50))
        source.setEventHandler { [weak self] in
            guard let self = self else { return }

            if self.isPreview {
                self.previewElapsed += 0.25
                let total = 300
                let rem = max(0, total - Int(self.previewElapsed))
                self.state = PomotroidTimerState(
                    isRunning: true,
                    roundType: "short-break",
                    totalSecs: total,
                    elapsedSecs: Int(self.previewElapsed),
                    remainingSecs: rem,
                    currentRound: 2,
                    totalRounds: 4,
                    todayRounds: 3,
                    todayFocusMins: 75
                )
                self.updateClock()
                if rem <= 0 {
                    NSApp.terminate(nil)
                }
                return
            }

            let (newState, newTheme) = fetchPomotroidStateAndTheme()
            self.state = newState
            self.theme = newTheme
            self.updateClock()

            if !newState.isRunning || newState.roundType != "short-break" || newState.remainingSecs <= 0 {
                NSApp.terminate(nil)
            }
        }
        source.resume()
        self.gcdTimer = source

        let bTimer = Timer(timeInterval: 3.5, repeats: true) { [weak self] _ in
            guard let self = self else { return }
            self.breathIndex = (self.breathIndex + 1) % self.breathCycles.count
            let cycle = self.breathCycles[self.breathIndex]
            withAnimation(.easeInOut(duration: 3.2)) {
                self.breathGuide = cycle.text
                self.breathScale = cycle.scale
            }
        }
        RunLoop.main.add(bTimer, forMode: .common)
        self.breathTimer = bTimer
    }

    deinit {
        gcdTimer?.cancel()
        breathTimer?.invalidate()
    }
}

// ============================================================================
// Simple, Clean Pomotroid-Themed View
// ============================================================================

struct BreakLockView: View {
    @ObservedObject var viewModel: BreakLockViewModel

    var progress: Double {
        guard viewModel.state.totalSecs > 0 else { return 0.0 }
        return Double(viewModel.state.elapsedSecs) / Double(viewModel.state.totalSecs)
    }

    var timeFormatted: String {
        let mins = viewModel.state.remainingSecs / 60
        let secs = viewModel.state.remainingSecs % 60
        return String(format: "%02d:%02d", mins, secs)
    }

    var expectedUnlockTimeStr: String {
        let unlockDate = Date().addingTimeInterval(TimeInterval(viewModel.state.remainingSecs))
        let formatter = DateFormatter()
        formatter.dateFormat = "h:mm a"
        return formatter.string(from: unlockDate)
    }

    var body: some View {
        ZStack {
            // Native Theme Background
            viewModel.theme.background
                .ignoresSafeArea()

            VStack(spacing: 32) {
                // Header Bar
                HStack {
                    HStack(spacing: 8) {
                        Image(systemName: "lock.shield.fill")
                            .font(.system(size: 13, weight: .bold))
                            .foregroundColor(viewModel.theme.accent)
                        Text(viewModel.isPreview ? "POMOTROID SHIELD (PREVIEW)" : "POMOTROID SHIELD")
                            .font(.system(size: 12, weight: .bold, design: .rounded))
                            .tracking(2.0)
                            .foregroundColor(viewModel.theme.foreground)
                    }
                    .padding(.horizontal, 14)
                    .padding(.vertical, 7)
                    .background(
                        Capsule()
                            .fill(viewModel.theme.backgroundLight.opacity(0.6))
                    )

                    Spacer()

                    if viewModel.isPreview {
                        Text("PREVIEW ACTIVE • PRESS ESC TO EXIT")
                            .font(.system(size: 11, weight: .bold, design: .rounded))
                            .tracking(1.2)
                            .foregroundColor(viewModel.theme.accent)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                            .background(
                                Capsule()
                                    .fill(viewModel.theme.backgroundLight.opacity(0.8))
                            )
                    }

                    Spacer()

                    HStack(spacing: 6) {
                        Image(systemName: "clock")
                            .font(.system(size: 12))
                            .foregroundColor(viewModel.theme.foregroundDarker)
                        Text(viewModel.localTimeStr)
                            .font(.system(size: 13, weight: .medium, design: .monospaced))
                            .foregroundColor(viewModel.theme.foreground)
                    }
                }
                .padding(.horizontal, 48)
                .padding(.top, 36)

                Spacer()

                // Simple, Clean Pomotroid Progress Dial (matches TimerDial.svelte)
                VStack(spacing: 16) {
                    ZStack {
                        // Background track (stroke width 3, backgroundLight)
                        Circle()
                            .stroke(viewModel.theme.backgroundLight, lineWidth: 3)
                            .frame(width: 250, height: 250)

                        // Progress arc (stroke width 10, round linecap, shortRound color)
                        Circle()
                            .trim(from: 0.0, to: CGFloat(min(progress, 1.0)))
                            .stroke(
                                viewModel.theme.shortRound,
                                style: StrokeStyle(lineWidth: 10, lineCap: .round)
                            )
                            .rotationEffect(.degrees(-90))
                            .frame(width: 250, height: 250)
                            .animation(.linear(duration: 0.25), value: progress)

                        // Center content: digital time + round label
                        VStack(spacing: 4) {
                            Text(timeFormatted)
                                .font(.system(size: 56, weight: .light, design: .rounded))
                                .foregroundColor(viewModel.theme.foreground)
                                .monospacedDigit()

                            Text("SHORT BREAK")
                                .font(.system(size: 12, weight: .bold, design: .rounded))
                                .tracking(2.5)
                                .foregroundColor(viewModel.theme.shortRound)
                        }
                    }

                    // Gentle breathing mindfulness guide
                    Text(viewModel.breathGuide)
                        .font(.system(size: 14, weight: .medium, design: .rounded))
                        .foregroundColor(viewModel.theme.foregroundDarker)
                        .transition(.opacity)
                }

                Spacer()

                // Mindful Activity Cards
                HStack(spacing: 16) {
                    TipCard(
                        icon: "figure.walk",
                        title: "Stand Up & Stretch",
                        subtitle: "Release physical tension from back & neck",
                        theme: viewModel.theme
                    )
                    TipCard(
                        icon: "drop.fill",
                        title: "Hydrate",
                        subtitle: "Drink a tall glass of cool fresh water",
                        theme: viewModel.theme
                    )
                    TipCard(
                        icon: "eye",
                        title: "Rest Your Eyes",
                        subtitle: "Look at an object 20 feet away for 20 seconds",
                        theme: viewModel.theme
                    )
                }
                .padding(.horizontal, 48)

                Spacer()

                // Footer Bar
                HStack {
                    HStack(spacing: 6) {
                        Image(systemName: "flame.fill")
                            .foregroundColor(viewModel.theme.focusRound)
                        Text("Cycle: Round \(viewModel.state.currentRound) of \(viewModel.state.totalRounds)")
                            .font(.system(size: 13, weight: .medium, design: .rounded))
                            .foregroundColor(viewModel.theme.foregroundDarker)
                    }

                    Spacer()

                    Text("Press ESC to exit")
                        .font(.system(size: 11, weight: .medium, design: .rounded))
                        .foregroundColor(viewModel.theme.foregroundDarker.opacity(0.6))

                    Spacer()

                    HStack(spacing: 6) {
                        Image(systemName: "lock.open.fill")
                            .foregroundColor(viewModel.theme.shortRound)
                        Text("Auto-unlocks at \(expectedUnlockTimeStr)")
                            .font(.system(size: 13, weight: .semibold, design: .rounded))
                            .foregroundColor(viewModel.theme.foreground)
                    }
                }
                .padding(.horizontal, 48)
                .padding(.bottom, 32)
            }
        }
    }
}

struct TipCard: View {
    let icon: String
    let title: String
    let subtitle: String
    let theme: ThemeColors

    var body: some View {
        HStack(spacing: 14) {
            Image(systemName: icon)
                .font(.system(size: 18))
                .foregroundColor(theme.accent)
                .frame(width: 34, height: 34)
                .background(Circle().fill(theme.backgroundLight))

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.system(size: 12, weight: .semibold, design: .rounded))
                    .foregroundColor(theme.foreground)
                Text(subtitle)
                    .font(.system(size: 11, weight: .regular))
                    .foregroundColor(theme.foregroundDarker)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 10)
                .fill(theme.backgroundLight.opacity(0.6))
        )
    }
}

// ============================================================================
// NSApplication Delegate & Window Setup
// ============================================================================

class AppDelegate: NSObject, NSApplicationDelegate {
    var windows: [NSWindow] = []
    var viewModel: BreakLockViewModel?

    func applicationDidFinishLaunching(_ notification: Notification) {
        let isPreview = CommandLine.arguments.contains("--preview") || CommandLine.arguments.contains("-p")

        let initial: PomotroidTimerState
        let theme: ThemeColors

        if isPreview {
            let (_, t) = fetchPomotroidStateAndTheme()
            theme = t
            initial = PomotroidTimerState(
                isRunning: true,
                roundType: "short-break",
                totalSecs: 300,
                elapsedSecs: 0,
                remainingSecs: 300,
                currentRound: 2,
                totalRounds: 4,
                todayRounds: 3,
                todayFocusMins: 75
            )
        } else {
            let (s, t) = fetchPomotroidStateAndTheme()
            theme = t
            initial = s
            if !initial.isRunning || initial.roundType != "short-break" || initial.remainingSecs <= 1 {
                exit(0)
            }
        }

        let vm = BreakLockViewModel(initialState: initial, theme: theme, isPreview: isPreview)
        self.viewModel = vm

        // Allow Escape key to dismiss cleanly
        NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            if event.keyCode == 53 { // ESC key
                NSApp.terminate(nil)
                return nil
            }
            return event
        }

        if !isPreview {
            NSApp.presentationOptions = [
                .hideDock,
                .hideMenuBar,
                .disableProcessSwitching,
                .disableForceQuit,
                .disableSessionTermination
            ]
        }

        for screen in NSScreen.screens {
            let window = NSWindow(
                contentRect: screen.frame,
                styleMask: [.borderless],
                backing: .buffered,
                defer: false,
                screen: screen
            )

            window.level = NSWindow.Level(rawValue: Int(CGShieldingWindowLevel()))
            window.backgroundColor = theme.background.nsColor
            window.isOpaque = true
            window.hasShadow = false
            window.ignoresMouseEvents = false
            window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]

            let hostingView = NSHostingView(rootView: BreakLockView(viewModel: vm))
            hostingView.frame = window.contentView!.bounds
            hostingView.autoresizingMask = [.width, .height]
            window.contentView?.addSubview(hostingView)

            window.makeKeyAndOrderFront(nil)
            windows.append(window)
        }

        NSApp.activate(ignoringOtherApps: true)
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let delegate = AppDelegate()
app.delegate = delegate
app.run()
