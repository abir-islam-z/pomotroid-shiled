//
//  mac_break_lock.swift
//  Native macOS Hardware Break Screen Lock for Pomotroid Shield
//
//  Features:
//  - Fully synchronized with Pomotroid active theme (background, dials, text, accent)
//  - Multi-monitor hardware shielding at CGShieldingWindowLevel
//  - High-precision local countdown timer with dynamic unlock wall-clock calculation
//  - Breathing mindfulness prompts with fluid scale animations
//  - Safety ESC key handler (keycode 53)
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

func fetchActiveTheme() -> ThemeColors {
    var dbPath = NSString(string: "~/Library/Application Support/com.abirislam.pomotroid-shield/pomotroid_shield.db").expandingTildeInPath
    if !FileManager.default.fileExists(atPath: dbPath) {
        dbPath = NSString(string: "~/Library/Application Support/com.splode.pomotroid/pomotroid.db").expandingTildeInPath
    }

    if FileManager.default.fileExists(atPath: dbPath) {
        var db: OpaquePointer? = nil
        if sqlite3_open_v2(dbPath, &db, SQLITE_OPEN_READONLY, nil) == SQLITE_OK {
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
            sqlite3_close(db)

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
    }

    return loadThemeColors(themeName: "Pomotroid")
}

// ============================================================================
// Timer State
// ============================================================================

struct PomotroidTimerState {
    var isRunning: Bool
    var roundType: String
    var totalSecs: Int
    var elapsedSecs: Int
    var remainingSecs: Int
    var currentRound: Int = 1
    var totalRounds: Int = 4
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
        ("Inhale gently...", 1.08),
        ("Hold your breath...", 1.08),
        ("Exhale slowly...", 0.94),
        ("Rest and relax...", 1.0)
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

            self.previewElapsed += 0.25
            let total = self.state.totalSecs
            let rem = max(0, total - Int(self.previewElapsed))
            self.state.remainingSecs = rem
            self.state.elapsedSecs = min(total, Int(self.previewElapsed))
            self.updateClock()

            if self.isPreview {
                if self.previewElapsed >= 6.0 {
                    NSApp.terminate(nil)
                }
                return
            }

            if rem <= 0 {
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
            // Deep obsidian background
            Color(red: 7/255.0, green: 10/255.0, blue: 18/255.0)
                .ignoresSafeArea()

            // Soft ambient glow tinted with theme break colors
            RadialGradient(
                gradient: Gradient(colors: [
                    viewModel.theme.shortRound.opacity(0.12),
                    viewModel.theme.accent.opacity(0.05),
                    Color.clear
                ]),
                center: .center,
                startRadius: 80,
                endRadius: 600
            )
            .ignoresSafeArea()

            VStack(spacing: 36) {
                // Top Header
                HStack {
                    HStack(spacing: 8) {
                        Image(systemName: "shield.lefthalf.filled")
                            .font(.system(size: 16, weight: .bold))
                            .foregroundColor(viewModel.theme.shortRound)
                        Text("POMOTROID SHIELD BREAK")
                            .font(.system(size: 13, weight: .bold, design: .rounded))
                            .tracking(2.0)
                            .foregroundColor(viewModel.theme.foreground)
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .background(
                        Capsule()
                            .fill(viewModel.theme.shortRound.opacity(0.12))
                            .overlay(Capsule().stroke(viewModel.theme.shortRound.opacity(0.3), lineWidth: 1))
                    )

                    Spacer()

                    Text(viewModel.localTimeStr)
                        .font(.system(size: 15, weight: .medium, design: .monospaced))
                        .foregroundColor(viewModel.theme.foregroundDarker.opacity(0.7))
                }
                .padding(.horizontal, 52)
                .padding(.top, 40)

                Spacer()

                // Centerpiece: Progress Dial & Digital Countdown
                VStack(spacing: 28) {
                    ZStack {
                        // Background track
                        Circle()
                            .stroke(
                                viewModel.theme.foregroundDarker.opacity(0.12),
                                style: StrokeStyle(lineWidth: 10, lineCap: .round)
                            )
                            .frame(width: 270, height: 270)

                        // Animated progress arc
                        Circle()
                            .trim(from: 0.0, to: CGFloat(min(progress, 1.0)))
                            .stroke(
                                viewModel.theme.shortRound,
                                style: StrokeStyle(lineWidth: 10, lineCap: .round)
                            )
                            .rotationEffect(.degrees(-90))
                            .frame(width: 270, height: 270)
                            .animation(.linear(duration: 0.25), value: progress)

                        // Center content: digital time + round label
                        VStack(spacing: 6) {
                            Text(timeFormatted)
                                .font(.system(size: 62, weight: .light, design: .rounded))
                                .foregroundColor(viewModel.theme.foreground)
                                .monospacedDigit()

                            Text("SHORT BREAK")
                                .font(.system(size: 13, weight: .bold, design: .rounded))
                                .tracking(2.5)
                                .foregroundColor(viewModel.theme.shortRound)
                        }
                    }

                    // Gentle breathing mindfulness guide
                    Text(viewModel.breathGuide)
                        .font(.system(size: 16, weight: .medium, design: .rounded))
                        .foregroundColor(viewModel.theme.foregroundDarker)
                        .scaleEffect(viewModel.breathScale)
                        .transition(.opacity)
                }

                Spacer()

                // Mindful Activity Cards
                HStack(spacing: 20) {
                    TipCard(
                        icon: "figure.walk",
                        title: "Stand Up & Stretch",
                        subtitle: "Release tension from your back & neck",
                        theme: viewModel.theme
                    )
                    TipCard(
                        icon: "drop.fill",
                        title: "Hydrate",
                        subtitle: "Drink a tall glass of fresh cool water",
                        theme: viewModel.theme
                    )
                    TipCard(
                        icon: "eye",
                        title: "Rest Your Eyes",
                        subtitle: "Look at an object 20 feet away for 20s",
                        theme: viewModel.theme
                    )
                }
                .padding(.horizontal, 52)

                Spacer()

                // Footer Bar
                HStack {
                    HStack(spacing: 8) {
                        Image(systemName: "flame.fill")
                            .foregroundColor(viewModel.theme.focusRound)
                        Text("Cycle: Round \(viewModel.state.currentRound) of \(viewModel.state.totalRounds)")
                            .font(.system(size: 14, weight: .medium, design: .rounded))
                            .foregroundColor(viewModel.theme.foregroundDarker)
                    }

                    Spacer()

                    Text("Press ESC to exit")
                        .font(.system(size: 12, weight: .medium, design: .rounded))
                        .foregroundColor(viewModel.theme.foregroundDarker.opacity(0.6))

                    Spacer()

                    HStack(spacing: 8) {
                        Image(systemName: "lock.open.fill")
                            .foregroundColor(viewModel.theme.shortRound)
                        Text("Auto-unlocks at \(expectedUnlockTimeStr)")
                            .font(.system(size: 14, weight: .semibold, design: .rounded))
                            .foregroundColor(viewModel.theme.foreground)
                    }
                }
                .padding(.horizontal, 52)
                .padding(.bottom, 36)
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
                .font(.system(size: 20))
                .foregroundColor(theme.accent)
                .frame(width: 38, height: 38)
                .background(Circle().fill(theme.backgroundLight))

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.system(size: 13, weight: .semibold, design: .rounded))
                    .foregroundColor(theme.foreground)
                Text(subtitle)
                    .font(.system(size: 11, weight: .regular))
                    .foregroundColor(theme.foregroundDarker)
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(theme.backgroundLight.opacity(0.55))
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
        let args = CommandLine.arguments
        let isPreview = args.contains("--preview") || args.contains("-p")

        var customDuration: Int = 300
        var customRound: Int = 1
        var customRoundsTotal: Int = 4

        for (i, arg) in args.enumerated() {
            if arg == "--duration" && i + 1 < args.count {
                if let d = Int(args[i + 1]), d > 0 { customDuration = d }
            } else if arg == "--round" && i + 1 < args.count {
                if let r = Int(args[i + 1]), r > 0 { customRound = r }
            } else if arg == "--rounds-total" && i + 1 < args.count {
                if let t = Int(args[i + 1]), t > 0 { customRoundsTotal = t }
            }
        }

        let theme = fetchActiveTheme()
        let initial = PomotroidTimerState(
            isRunning: true,
            roundType: "short-break",
            totalSecs: customDuration,
            elapsedSecs: 0,
            remainingSecs: customDuration,
            currentRound: customRound,
            totalRounds: customRoundsTotal
        )

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
