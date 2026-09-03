//
//  mac_break_lock.swift
//  Native macOS Break Screen Lock for Pomotroid
//
//  Features:
//  - Full-screen hardware shielding (CGShieldingWindowLevel) on ALL connected monitors
//  - GCD DispatchSourceTimer for reliable second-by-second countdown
//  - Real-time synchronization with Pomotroid SQLite DB and log stream
//  - Dynamic wall-clock unlock time (e.g. "Auto-unlocks at 5:17 PM")
//  - Guided mindfulness breathing animation (Inhale, Hold, Exhale, Rest)
//  - Rotating mindful break activity prompts
//  - Seamless escape valve (ESC key) for testing & safety
//  - First-class preview mode (--preview)
//

import Cocoa
import SwiftUI
import SQLite3

// ============================================================================
// Data Models
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

    // Also check by process / executable name (handles `npm run tauri dev` / `target/debug/pomotroid`)
    for app in NSWorkspace.shared.runningApplications {
        if let name = app.localizedName?.lowercased(), name.contains("pomotroid") {
            return true
        }
        if let exe = app.executableURL?.lastPathComponent.lowercased(), exe.contains("pomotroid") {
            return true
        }
    }
    return false
}

func fetchPomotroidState() -> PomotroidTimerState {
    if !isPomotroidRunning() {
        return PomotroidTimerState(isRunning: false, roundType: "idle", totalSecs: 300, elapsedSecs: 0, remainingSecs: 0)
    }

    var targetRounds = 4
    var todayRounds = 0
    var todayFocusMins = 0
    var dbRoundType = "short-break"
    var dbDuration = 300
    var sStart: Int64 = Int64(Date().timeIntervalSince1970)
    var sEnd: Int64? = nil

    var dbPath = NSString(string: "~/Library/Application Support/com.abirislam.pomotroid-shield/pomotroid_shield.db").expandingTildeInPath
    if !FileManager.default.fileExists(atPath: dbPath) {
        dbPath = NSString(string: "~/Library/Application Support/com.splode.pomotroid/pomotroid.db").expandingTildeInPath
    }

    if FileManager.default.fileExists(atPath: dbPath) {
        var db: OpaquePointer? = nil
        if sqlite3_open_v2(dbPath, &db, SQLITE_OPEN_READONLY, nil) == SQLITE_OK {
            var stmt: OpaquePointer? = nil
            if sqlite3_prepare_v2(db, "SELECT value FROM settings WHERE key = 'work_rounds'", -1, &stmt, nil) == SQLITE_OK {
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
    return PomotroidTimerState(
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
}

// ============================================================================
// Observable View Model
// ============================================================================

class BreakLockViewModel: ObservableObject {
    @Published var state: PomotroidTimerState
    @Published var localTimeStr: String = ""
    @Published var breathGuide: String = "Inhale slowly..."
    @Published var breathScale: CGFloat = 1.0
    let isPreview: Bool

    private var gcdTimer: DispatchSourceTimer?
    private var breathTimer: Timer?
    private var breathIndex: Int = 0
    private var previewElapsed: Double = 0.0

    let breathCycles: [(text: String, scale: CGFloat)] = [
        ("Inhale gently...", 1.15),
        ("Hold...", 1.15),
        ("Exhale slowly...", 0.92),
        ("Rest and release...", 1.0)
    ]

    init(initialState: PomotroidTimerState, isPreview: Bool = false) {
        self.state = initialState
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

            let newState = fetchPomotroidState()
            self.state = newState
            self.updateClock()

            // Dismiss the lock screen when break completes or changes
            if !newState.isRunning || newState.roundType != "short-break" || newState.remainingSecs <= 0 {
                NSApp.terminate(nil)
            }
        }
        source.resume()
        self.gcdTimer = source

        // Breathing cycle animation
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
// SwiftUI UI Views
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
            // Background: Deep obsidian glass
            Color(red: 7/255.0, green: 10/255.0, blue: 18/255.0)
                .ignoresSafeArea()

            // Ambient background glow
            RadialGradient(
                gradient: Gradient(colors: [
                    Color(red: 20/255.0, green: 184/255.0, blue: 166/255.0).opacity(0.12),
                    Color(red: 14/255.0, green: 165/255.0, blue: 233/255.0).opacity(0.06),
                    Color.clear
                ]),
                center: .center,
                startRadius: 80,
                endRadius: 550
            )
            .ignoresSafeArea()

            VStack(spacing: 36) {
                // Header Bar
                HStack {
                    HStack(spacing: 8) {
                        Image(systemName: "lock.shield.fill")
                            .font(.system(size: 14, weight: .bold))
                            .foregroundColor(Color(red: 45/255.0, green: 212/255.0, blue: 191/255.0))
                        Text(viewModel.isPreview ? "POMOTROID SHIELD (PREVIEW)" : "POMOTROID SHIELD")
                            .font(.system(size: 13, weight: .bold, design: .rounded))
                            .tracking(2.0)
                            .foregroundColor(.white.opacity(0.9))
                    }
                    .padding(.horizontal, 14)
                    .padding(.vertical, 7)
                    .background(
                        Capsule()
                            .fill(Color.white.opacity(0.06))
                            .overlay(Capsule().stroke(Color.white.opacity(0.12), lineWidth: 1))
                    )

                    Spacer()

                    if viewModel.isPreview {
                        Text("PREVIEW ACTIVE • PRESS ESC TO EXIT")
                            .font(.system(size: 11, weight: .bold, design: .rounded))
                            .tracking(1.2)
                            .foregroundColor(Color(red: 45/255.0, green: 212/255.0, blue: 191/255.0))
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                            .background(
                                Capsule()
                                    .fill(Color(red: 20/255.0, green: 184/255.0, blue: 166/255.0).opacity(0.15))
                                    .overlay(Capsule().stroke(Color(red: 20/255.0, green: 184/255.0, blue: 166/255.0).opacity(0.3), lineWidth: 1))
                            )
                    }

                    Spacer()

                    HStack(spacing: 6) {
                        Image(systemName: "clock")
                            .font(.system(size: 13))
                            .foregroundColor(.white.opacity(0.6))
                        Text(viewModel.localTimeStr)
                            .font(.system(size: 14, weight: .medium, design: .monospaced))
                            .foregroundColor(.white.opacity(0.85))
                    }
                }
                .padding(.horizontal, 48)
                .padding(.top, 36)

                Spacer()

                // Main Circular Countdown & Breathing Ring
                ZStack {
                    // Outer breathing glow pulse
                    Circle()
                        .fill(Color(red: 20/255.0, green: 184/255.0, blue: 166/255.0).opacity(0.08))
                        .frame(width: 320, height: 320)
                        .scaleEffect(viewModel.breathScale)

                    // Track ring
                    Circle()
                        .stroke(Color.white.opacity(0.08), lineWidth: 10)
                        .frame(width: 250, height: 250)

                    // Live progress ring
                    Circle()
                        .trim(from: 0.0, to: CGFloat(min(progress, 1.0)))
                        .stroke(
                            LinearGradient(
                                colors: [
                                    Color(red: 45/255.0, green: 212/255.0, blue: 191/255.0),
                                    Color(red: 56/255.0, green: 189/255.0, blue: 248/255.0)
                                ],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            ),
                            style: StrokeStyle(lineWidth: 10, lineCap: .round)
                        )
                        .rotationEffect(.degrees(-90))
                        .frame(width: 250, height: 250)
                        .animation(.linear(duration: 0.25), value: progress)

                    // Center Content
                    VStack(spacing: 6) {
                        Text("STEP AWAY & RECHARGE")
                            .font(.system(size: 10, weight: .bold, design: .rounded))
                            .tracking(2.5)
                            .foregroundColor(Color(red: 45/255.0, green: 212/255.0, blue: 191/255.0))

                        Text(timeFormatted)
                            .font(.system(size: 64, weight: .light, design: .rounded))
                            .foregroundColor(.white)
                            .monospacedDigit()

                        Text(viewModel.breathGuide)
                            .font(.system(size: 14, weight: .medium, design: .rounded))
                            .foregroundColor(.white.opacity(0.75))
                            .transition(.opacity)
                    }
                }

                // Mindful Break Tips Carousel
                HStack(spacing: 16) {
                    TipCard(icon: "figure.walk", title: "Stand Up & Stretch", subtitle: "Release physical tension from back & neck")
                    TipCard(icon: "drop.fill", title: "Hydrate", subtitle: "Drink a tall glass of cool fresh water")
                    TipCard(icon: "eye", title: "Rest Your Eyes", subtitle: "Look at an object 20 feet away for 20 seconds")
                }
                .padding(.horizontal, 48)

                Spacer()

                // Footer Bar
                HStack {
                    HStack(spacing: 6) {
                        Image(systemName: "flame.fill")
                            .foregroundColor(.orange)
                        Text("Cycle: Round \(viewModel.state.currentRound) of \(viewModel.state.totalRounds)")
                            .font(.system(size: 13, weight: .medium, design: .rounded))
                            .foregroundColor(.white.opacity(0.85))
                    }

                    Spacer()

                    Text("Press ESC to exit preview")
                        .font(.system(size: 11, weight: .medium, design: .rounded))
                        .foregroundColor(.white.opacity(0.4))

                    Spacer()

                    HStack(spacing: 6) {
                        Image(systemName: "lock.open.fill")
                            .foregroundColor(Color(red: 45/255.0, green: 212/255.0, blue: 191/255.0))
                        Text("Auto-unlocks at \(expectedUnlockTimeStr)")
                            .font(.system(size: 13, weight: .semibold, design: .rounded))
                            .foregroundColor(.white.opacity(0.85))
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

    var body: some View {
        HStack(spacing: 14) {
            Image(systemName: icon)
                .font(.system(size: 20))
                .foregroundColor(Color(red: 45/255.0, green: 212/255.0, blue: 191/255.0))
                .frame(width: 36, height: 36)
                .background(Circle().fill(Color.white.opacity(0.06)))

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.system(size: 13, weight: .semibold, design: .rounded))
                    .foregroundColor(.white.opacity(0.9))
                Text(subtitle)
                    .font(.system(size: 11, weight: .regular))
                    .foregroundColor(.white.opacity(0.55))
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color.white.opacity(0.04))
                .overlay(RoundedRectangle(cornerRadius: 12).stroke(Color.white.opacity(0.08), lineWidth: 1))
        )
    }
}

// ============================================================================
// NSApplication Delegate & Hardware Shield Windows
// ============================================================================

class AppDelegate: NSObject, NSApplicationDelegate {
    var windows: [NSWindow] = []
    var viewModel: BreakLockViewModel?

    func applicationDidFinishLaunching(_ notification: Notification) {
        print("DEBUG: applicationDidFinishLaunching, args=\(CommandLine.arguments)")
        let isPreview = CommandLine.arguments.contains("--preview") || CommandLine.arguments.contains("-p")

        let initial: PomotroidTimerState
        if isPreview {
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
            initial = fetchPomotroidState()
            if !initial.isRunning || initial.roundType != "short-break" || initial.remainingSecs <= 1 {
                exit(0)
            }
        }

        let vm = BreakLockViewModel(initialState: initial, isPreview: isPreview)
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

        // Create full-screen window for EVERY connected monitor
        for screen in NSScreen.screens {
            let window = NSWindow(
                contentRect: screen.frame,
                styleMask: [.borderless],
                backing: .buffered,
                defer: false,
                screen: screen
            )

            window.level = NSWindow.Level(rawValue: Int(CGShieldingWindowLevel()))
            window.backgroundColor = NSColor(calibratedRed: 7/255.0, green: 10/255.0, blue: 18/255.0, alpha: 1.0)
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
