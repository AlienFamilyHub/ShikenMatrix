import SwiftUI
import AppKit

// MARK: - Modern UI Constants
private enum UI {
    static let sidebarWidth: CGFloat = 280
    static let iconSize: CGFloat = 14
    static let cornerRadius: CGFloat = 6
    static let padding: CGFloat = 12
}

struct ContentView: View {
    // MARK: - State Properties
    @State private var config = ReporterConfig(enabled: false, wsUrl: "", token: "", enableMediaReporting: false)
    @State private var reporterHandle: UnsafeMutableRawPointer?
    
    // Status
    @State private var isRunning = false
    @State private var isConnected = false
    @State private var statusMessage = "就绪"
    @State private var lastError: String?
    
    // Permissions
    @State private var hasAccessibilityPermission = false
    @State private var hasMediaPermission = false
    
    // Data
    @State private var logs: [LogEntry] = []
    @State private var currentWindow: WindowData?
    @State private var currentMedia: MediaData?
    
    // UI Logic
    @State private var searchText = ""
    @State private var autoScroll = true
    @State private var isConfigExpanded = true // 配置区域折叠状态
    
    // Alerts & Timers
    @State private var showAlert = false
    @State private var alertMessage = ""
    @State private var showPermissionAlert = false
    @State private var showMediaAlert = false
    @State private var statusTimer: Timer?
    @State private var logCleanupTimer: Timer?
    
    // External
    private let appDelegate = NSApp.delegate as? AppDelegate
    private let githubUrl = URL(string: "https://github.com/TNXG/ShikenMatrix")!

    var body: some View {
        HSplitView {
            // MARK: - LEFT PANEL: Dashboard
            VStack(spacing: 0) {
                // 1. Toolbar Area (Top)
                toolbarSection
                
                Divider()
                
                ScrollView {
                    VStack(spacing: 16) {
                        // 2. Real-time Monitoring (The Core)
                        monitoringSection
                        
                        Divider().opacity(0.5)
                        
                        // 3. Configuration (Compact)
                        configSection
                    }
                    .padding(UI.padding)
                }
                
                Spacer()
                
                // 4. Footer
                footerSection
            }
            .frame(minWidth: UI.sidebarWidth, maxWidth: 320)
            .background(VisualEffectView(material: .sidebar, blendingMode: .behindWindow))
            
            // MARK: - RIGHT PANEL: Terminal
            VStack(spacing: 0) {
                logToolbarSection
                Divider()
                logListSection
            }
            .frame(minWidth: 500)
            .background(Color(NSColor.textBackgroundColor))
        }
        .frame(minWidth: 800, minHeight: 500)
        .onAppear(perform: setupApp)
        .onDisappear(perform: cleanup)
        .alert("提示", isPresented: $showAlert) { Button("OK", role: .cancel) { } } message: { Text(alertMessage) }
    }

    // MARK: - Left Panel: Toolbar
    private var toolbarSection: some View {
        HStack {
            // Status Indicator
            HStack(spacing: 6) {
                Circle()
                    .fill(statusIndicatorColor)
                    .frame(width: 8, height: 8)
                    .shadow(color: statusIndicatorColor.opacity(0.3), radius: 2)
                
                Text(statusMessage)
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            // Main Toggle Switch (Modern Style)
            Toggle("", isOn: Binding(
                get: { isRunning },
                set: { _ in toggleReporter() }
            ))
            .toggleStyle(.switch)
            .controlSize(.small)
            .disabled(config.wsUrl.isEmpty || config.token.isEmpty)
            .help(isRunning ? "停止服务" : "启动服务")
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
    }

    // MARK: - Left Panel: Monitoring (Core)
    private var monitoringSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label("LIVE MONITOR", systemImage: "waveform.path.ecg")
                .font(.system(size: 10, weight: .bold))
                .foregroundColor(.secondary)
            
            if !isRunning {
                emptyStateView(text: "服务未启动", icon: "pause.circle")
            } else if currentWindow == nil && currentMedia == nil {
                emptyStateView(text: "等待数据...", icon: "clock.arrow.circlepath")
            } else {
                // Window Info Card
                if let window = currentWindow {
                    MonitorCard {
                        HStack(alignment: .top, spacing: 10) {
                            // Icon - Use directly from WindowData (fetched via AppKit)
                            if let nsImage = window.icon {
                                Image(nsImage: nsImage)
                                    .resizable()
                                    .frame(width: 32, height: 32)
                            } else {
                                Image(systemName: "macwindow")
                                    .font(.system(size: 20))
                                    .frame(width: 32, height: 32)
                                    .foregroundStyle(.blue.gradient)
                            }
                            
                            VStack(alignment: .leading, spacing: 2) {
                                Text(window.title)
                                    .font(.system(size: 12, weight: .semibold))
                                    .lineLimit(2)
                                    .fixedSize(horizontal: false, vertical: true)
                                
                                HStack(spacing: 6) {
                                    Text(window.processName)
                                    Text("PID: \(window.pid)")
                                        .font(.system(size: 9, design: .monospaced))
                                        .opacity(0.7)
                                }
                                .font(.caption)
                                .foregroundColor(.secondary)
                            }
                        }
                    }
                }
                
                // Media Info Card
                if let media = currentMedia {
                    MonitorCard {
                        HStack(alignment: .center, spacing: 10) {
                            // Artwork - 使用缓存的图片，避免反复解码
                            if let data = media.artworkData, let nsImage = ImageCache.shared.image(for: "artwork-\(media.title)\(media.artist)", from: data) {
                                Image(nsImage: nsImage)
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                                    .frame(width: 32, height: 32)
                                    .clipShape(RoundedRectangle(cornerRadius: 4))
                            } else {
                                Image(systemName: "music.quarternote.3")
                                    .frame(width: 32, height: 32)
                                    .background(Color.secondary.opacity(0.1))
                                    .cornerRadius(4)
                            }
                            
                            VStack(alignment: .leading, spacing: 1) {
                                Text(media.title)
                                    .font(.system(size: 11, weight: .medium))
                                    .lineLimit(1)
                                Text(media.artist)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                    .lineLimit(1)
                            }
                        }
                    }
                }
            }
            
            // Error Display (Compact)
            if let error = lastError {
                HStack(alignment: .top, spacing: 6) {
                    Image(systemName: "xmark.octagon.fill")
                        .foregroundColor(.red)
                    Text(error)
                        .lineLimit(4)
                }
                .font(.caption)
                .padding(8)
                .background(Color.red.opacity(0.08))
                .cornerRadius(UI.cornerRadius)
            }
        }
    }

    // MARK: - Left Panel: Config (Compact)
    private var configSection: some View {
        DisclosureGroup(isExpanded: $isConfigExpanded) {
            VStack(spacing: 12) {
                // Connection Fields
                CompactTextField(title: "Address", icon: "network", text: $config.wsUrl, placeholder: "ws://server:port")
                    .disabled(isRunning)
                
                CompactTextField(title: "Token", icon: "key", text: $config.token, placeholder: "Auth Token", isSecure: true)
                    .disabled(isRunning)
                
                // Feature Toggle
                HStack {
                    Image(systemName: "film")
                        .font(.system(size: UI.iconSize))
                        .foregroundColor(.secondary)
                        .frame(width: 16)
                    Text("媒体监控")
                        .font(.system(size: 12))
                        .foregroundColor(.secondary)
                    
                    Spacer()
                    
                    if config.enableMediaReporting {
                        StatusDot(color: hasMediaPermission ? .green : .orange)
                    }
                    
                    Toggle("", isOn: $config.enableMediaReporting)
                        .toggleStyle(.switch)
                        .controlSize(.mini)
                        .disabled(isRunning)
                        .onChange(of: config.enableMediaReporting) { val in
                            if val { checkMediaPermission() }
                        }
                }
                .padding(4)
            }
            .padding(.top, 8)
        } label: {
            Label("SETTINGS", systemImage: "gearshape")
                .font(.system(size: 10, weight: .bold))
                .foregroundColor(.secondary)
        }
    }

    // MARK: - Left Panel: Footer
    private var footerSection: some View {
        HStack {
            Link(destination: githubUrl) {
                HStack(spacing: 4) {
                    Image(systemName: "terminal")
                    Text("ShikenMatrix")
                        .fontWeight(.medium)
                }
                .font(.system(size: 10))
                .foregroundColor(.secondary)
            }
            .buttonStyle(.plain)
            
            Spacer()
            
            // 权限指示器
            if !hasAccessibilityPermission {
                Button(action: requestAccessibilityPermission) {
                    Text("! AX Permission")
                        .font(.system(size: 9, weight: .bold))
                        .foregroundColor(.white)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 2)
                        .background(Capsule().fill(Color.red))
                }
                .buttonStyle(.plain)
            }
        }
        .padding(12)
        .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
        .border(width: 1, edges: [.top], color: Color(NSColor.separatorColor))
    }

    // MARK: - Right Panel: Log Toolbar
    private var logToolbarSection: some View {
        HStack(spacing: 0) {
            // Search
            HStack(spacing: 6) {
                Image(systemName: "magnifyingglass")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
                TextField("Filter logs...", text: $searchText)
                    .textFieldStyle(.plain)
                    .font(.system(size: 12))
            }
            .padding(.vertical, 6)
            .padding(.horizontal, 8)
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(UI.cornerRadius)
            .overlay(
                RoundedRectangle(cornerRadius: UI.cornerRadius)
                    .stroke(Color.secondary.opacity(0.2), lineWidth: 0.5)
            )
            .frame(maxWidth: 200)
            
            Spacer()
            
            // Actions
            HStack(spacing: 16) {
                Toggle(isOn: $autoScroll) {
                    Image(systemName: "arrow.down.to.line.compact")
                        .font(.system(size: 12))
                        .foregroundColor(autoScroll ? .accentColor : .secondary)
                }
                .toggleStyle(.button)
                .buttonStyle(.plain)
                .help("Auto Scroll")
                
                Button(action: { logs.removeAll() }) {
                    Image(systemName: "trash")
                        .font(.system(size: 12))
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
                .help("Clear Logs")
            }
        }
        .padding(8)
        .background(Color(NSColor.controlBackgroundColor).opacity(0.3))
    }

    // MARK: - Right Panel: Log List (Tabular)
    private var logListSection: some View {
        ScrollViewReader { proxy in
            List {
                let filtered = logs.filter { searchText.isEmpty || $0.message.localizedCaseInsensitiveContains(searchText) }
                ForEach(filtered) { log in
                    LogEntryRow(log: log)
                        .listRowInsets(EdgeInsets(top: 0, leading: 4, bottom: 0, trailing: 4))
                        .listRowSeparator(.hidden)
                }
            }
            .listStyle(.plain)
            .onChange(of: logs.count) {
                if autoScroll, let last = logs.last {
                    proxy.scrollTo(last.id, anchor: .bottom)
                }
            }
        }
    }
    
    // MARK: - Logic Handlers (Unchanged)
    // 为了节省篇幅，核心逻辑代码与上一个版本保持一致，此处进行连接
    private func setupApp() {
        // 防止重复初始化
        guard statusTimer == nil else { return }
        
        loadConfig(); checkAccessibilityPermission(); startStatusUpdates(); startLogCleanup()
        if RustBridge.isRunning() { setupCallbacks() }
        if config.enableMediaReporting { DispatchQueue.main.asyncAfter(deadline: .now() + 1) { checkMediaPermission() } }
    }
    
    private func setupCallbacks() {
        RustBridge.setLogCallback { l, m in
            DispatchQueue.main.async { [self] in self.addLog(m, level: l == .info ? .info : (l == .warning ? .warning : .error)) }
        }
        RustBridge.setWindowCallback { w in
            DispatchQueue.main.async { [self] in self.currentWindow = w }
        }
        RustBridge.setMediaCallback { m in
            DispatchQueue.main.async { [self] in self.currentMedia = m }
        }
    }

    private func toggleReporter() {
        if isRunning {
            if let handle = reporterHandle { _ = RustBridge.stopReporter(handle) }
            reporterHandle = nil; isRunning = false; isConnected = false; statusMessage = "已停止"; config.enabled = false
            currentWindow = nil; currentMedia = nil; lastError = nil; _ = RustBridge.saveConfig(config)
            updateStatusBar()
        } else {
            guard !config.wsUrl.isEmpty, !config.token.isEmpty else { alertMessage = "配置无效"; showAlert = true; return }
            config.enabled = true; _ = RustBridge.saveConfig(config)
            if let handle = RustBridge.startReporter(config: config) {
                reporterHandle = handle; isRunning = true; statusMessage = "启动中..."; lastError = nil
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { self.setupCallbacks() }
            } else {
                alertMessage = "启动失败"; showAlert = true; config.enabled = false
            }
            updateStatusBar()
        }
    }
    
    private func updateStatusBar() { appDelegate?.updateStatusBarStatus(isRunning: isRunning, isConnected: isConnected) }
    
    private func startStatusUpdates() {
        statusTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { _ in
            guard isRunning, let h = reporterHandle else { return }
            let s = RustBridge.getStatus(h)
            if s.isConnected != isConnected { isConnected = s.isConnected; statusMessage = isConnected ? "运行中" : "连接中断" }
            if let err = s.lastError, err != lastError { lastError = err; addLog("Err: \(err)", level: .error) }
            updateStatusBar()
        }
    }
    private func stopStatusUpdates() { statusTimer?.invalidate(); logCleanupTimer?.invalidate() }

    private func cleanup() {
        print("🧹 ContentView: Starting cleanup...")
        // Stop timers
        stopStatusUpdates()
        // Clear FFI callbacks to prevent memory leaks
        RustBridge.clearCallbacks()
        // Clear image cache
        ImageCache.shared.clearCache()
        // Clear handle
        if let handle = reporterHandle {
            _ = RustBridge.stopReporter(handle)
            reporterHandle = nil
        }
        // Clear logs to free memory
        logs.removeAll()
        currentWindow = nil
        currentMedia = nil
        print("✅ ContentView: Cleanup completed")
    }
    
    private func loadConfig() { if let c = RustBridge.loadConfig() { config = c } }
    private func startLogCleanup() {
        logCleanupTimer = Timer.scheduledTimer(withTimeInterval: 15, repeats: true) { _ in
            // 更激进的清理：只保留最近 200 条日志（之前是 500 条）
            let maxLogs = 200
            if logs.count > maxLogs {
                logs.removeFirst(logs.count - maxLogs)
            }
            // 同时清理超过 2 分钟的日志
            let date = Date().addingTimeInterval(-120)
            logs.removeAll { $0.timestamp < date }
        }
    }
    private func checkAccessibilityPermission() { hasAccessibilityPermission = RustBridge.checkAccessibilityPermission() }
    private func requestAccessibilityPermission() { _ = RustBridge.requestAccessibilityPermission() }
    private func checkMediaPermission() { hasMediaPermission = RustBridge.checkMediaPermission() }
    
    private func addLog(_ msg: String, level: LogLevel) {
        let id = UInt64(Date().timeIntervalSince1970 * 1000) + UInt64(logs.count)
        logs.append(LogEntry(id: id, timestamp: Date(), message: msg, level: level))
        // 更激进的清理：超过 200 条时移除前 100 条（频率比原来高，防止日志堆积）
        if logs.count > 200 { logs.removeFirst(100) }
    }
    
    private var statusIndicatorColor: Color {
        !isRunning ? .secondary : (isConnected ? .green : .orange)
    }
}

// MARK: - Reusable Components

struct MonitorCard<Content: View>: View {
    let content: Content
    init(@ViewBuilder content: () -> Content) { self.content = content() }
    
    var body: some View {
        content
            .padding(10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(UI.cornerRadius)
            .overlay(
                RoundedRectangle(cornerRadius: UI.cornerRadius)
                    .stroke(Color.secondary.opacity(0.1), lineWidth: 1)
            )
    }
}

struct CompactTextField: View {
    let title: String
    let icon: String
    @Binding var text: String
    let placeholder: String
    var isSecure = false
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Label(title, systemImage: icon)
                .font(.system(size: 9, weight: .semibold))
                .foregroundColor(.secondary)
            
            Group {
                if isSecure { SecureField(placeholder, text: $text) }
                else { TextField(placeholder, text: $text) }
            }
            .textFieldStyle(.plain)
            .font(.system(size: 11)) // Monospace for tokens?
            .padding(6)
            .background(Color(NSColor.textBackgroundColor))
            .cornerRadius(UI.cornerRadius - 1)
            .overlay(
                RoundedRectangle(cornerRadius: UI.cornerRadius - 1)
                    .stroke(Color.secondary.opacity(0.2), lineWidth: 0.5)
            )
        }
    }
}

struct emptyStateView: View {
    let text: String
    let icon: String
    
    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: icon)
            Text(text)
        }
        .font(.system(size: 11))
        .foregroundColor(.secondary)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 20)
        .background(
            RoundedRectangle(cornerRadius: UI.cornerRadius)
                .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [4]))
                .foregroundColor(.secondary.opacity(0.1))
        )
    }
}

struct StatusDot: View {
    let color: Color
    var body: some View {
        Circle().fill(color).frame(width: 6, height: 6)
    }
}

// MARK: - Optimized Log Row
struct LogEntryRow: View {
    let log: LogEntry
    static let formatter: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "HH:mm:ss.SSS"
        return f
    }()

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            // Fixed width timestamp to prevent wrapping
            Text(Self.formatter.string(from: log.timestamp))
                .font(.system(size: 10, design: .monospaced))
                .foregroundColor(.secondary)
                .frame(width: 80, alignment: .leading) 
                .fixedSize()
            
            // Icon
            Image(systemName: log.level.icon)
                .font(.system(size: 9))
                .foregroundColor(log.level.color)
                .frame(width: 12)
            
            // Message (Can wrap if needed, but distinct from time)
            Text(log.message)
                .font(.system(size: 11, design: .monospaced))
                .foregroundColor(log.level == .error ? .red : .primary)
                .lineLimit(nil)
                .fixedSize(horizontal: false, vertical: true)
                .textSelection(.enabled)
            
            Spacer(minLength: 0)
        }
        .padding(.vertical, 2)
    }
}

// MARK: - Supporting Models
enum LogLevel {
    case info, warning, error
    var color: Color {
        switch self { case .info: return .blue; case .warning: return .orange; case .error: return .red }
    }
    var icon: String {
        switch self { case .info: return "circle.fill"; case .warning: return "triangle.fill"; case .error: return "xmark.octagon.fill" }
    }
}

struct LogEntry: Identifiable, Equatable {
    let id: UInt64
    let timestamp: Date
    let message: String
    let level: LogLevel
}

struct VisualEffectView: NSViewRepresentable {
    let material: NSVisualEffectView.Material
    let blendingMode: NSVisualEffectView.BlendingMode
    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = material
        view.blendingMode = blendingMode
        view.state = .active
        return view
    }
    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {}
}

// Border Helper
extension View {
    func border(width: CGFloat, edges: [Edge], color: Color) -> some View {
        overlay(EdgeBorder(width: width, edges: edges).foregroundColor(color))
    }
}

struct EdgeBorder: Shape {
    var width: CGFloat
    var edges: [Edge]
    func path(in rect: CGRect) -> Path {
        var path = Path()
        for edge in edges {
            var x: CGFloat = 0, y: CGFloat = 0, w: CGFloat = 0, h: CGFloat = 0
            switch edge {
            case .top:    x=0; y=0; w=rect.width; h=width
            case .bottom: x=0; y=rect.height-width; w=rect.width; h=width
            case .leading:x=0; y=0; w=width; h=rect.height
            case .trailing:x=rect.width-width; y=0; w=width; h=rect.height
            }
            path.addRect(CGRect(x: x, y: y, width: w, height: h))
        }
        return path
    }
}