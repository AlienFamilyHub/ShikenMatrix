//
//  ContentView.swift
//  ShikenMatrix
//
//  Created by tianxiang on 2026/1/11.
//

import SwiftUI
import AppKit

struct ContentView: View {
    @State private var config = ReporterConfig(enabled: false, wsUrl: "", token: "")
    @State private var reporterHandle: UnsafeMutableRawPointer?
    @State private var isRunning = false
    @State private var isConnected = false
    @State private var statusMessage = "Stopped"
    @State private var lastError: String?
    @State private var showAlert = false
    @State private var alertMessage = ""

    // Log viewer state
    @State private var logs: [LogEntry] = []
    @State private var autoScroll = true

    // Timer for status updates
    @State private var statusTimer: Timer?

    // App delegate reference
    private let appDelegate = NSApp.delegate as? AppDelegate

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(spacing: 16) {
                    connectionSection
                    statusSection
                    logViewerSection
                }
                .padding()
            }

            Divider()
            footerView
        }
        .frame(minWidth: 500, minHeight: 400)
        .onAppear {
            loadConfig()
            startStatusUpdates()
            addLog("Application started", level: .info)
        }
        .onDisappear {
            stopStatusUpdates()
        }
        .alert("Error", isPresented: $showAlert) {
            Button("OK", role: .cancel) { }
        } message: {
            Text(alertMessage)
        }
    }

    // MARK: - View Components

    private var connectionSection: some View {
        GroupBox(label: Text("Connection").fontWeight(.semibold)) {
            VStack(alignment: .leading, spacing: 12) {
                Toggle("Enable Reporter", isOn: $config.enabled)

                HStack {
                    Text("WebSocket URL:")
                    TextField("ws://", text: $config.wsUrl)
                        .textFieldStyle(.roundedBorder)
                        .disabled(isRunning)
                }

                HStack {
                    Text("Token:")
                    SecureField("Enter token", text: $config.token)
                        .textFieldStyle(.roundedBorder)
                        .disabled(isRunning)
                }
            }
            .padding(8)
        }
    }

    private var statusSection: some View {
        GroupBox(label: Text("Status").fontWeight(.semibold)) {
            VStack(alignment: .leading, spacing: 12) {
                statusControlRow

                if isRunning {
                    Divider()
                    connectionDetailsView
                }
            }
            .padding(8)
        }
    }

    private var statusControlRow: some View {
        HStack {
            Circle()
                .fill(statusIndicatorColor)
                .frame(width: 12, height: 12)

            Text(statusMessage)
                .foregroundColor(.secondary)

            Spacer()

            Button(action: toggleReporter) {
                Text(isRunning ? "Stop" : "Start")
                    .frame(minWidth: 80)
            }
            .buttonStyle(.borderedProminent)
            .disabled(config.wsUrl.isEmpty || config.token.isEmpty)
        }
    }

    private var connectionDetailsView: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "network")
                    .foregroundColor(.secondary)
                Text("Server:")
                Text(config.wsUrl)
                    .foregroundColor(.secondary)
            }
            .font(.caption)

            if let error = lastError {
                HStack {
                    Image(systemName: "exclamationmark.triangle")
                        .foregroundColor(.orange)
                    Text(error)
                        .foregroundColor(.orange)
                }
                .font(.caption)
            }
        }
    }

    private var logViewerSection: some View {
        GroupBox(label: HStack {
            Text("Logs").fontWeight(.semibold)
            Spacer()
            Toggle("Auto-scroll", isOn: $autoScroll)
                .toggleStyle(.switch)
                .controlSize(.small)
        }) {
            VStack(alignment: .leading, spacing: 0) {
                logScrollView
                clearLogsButton
            }
            .padding(8)
        }
    }

    private var logScrollView: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 4) {
                    ForEach(logs) { log in
                        logEntryRow(log)
                    }
                }
                .padding(.vertical, 8)
            }
            .frame(height: 200)
            .onChange(of: logs) { _, newLogs in
                if autoScroll, let lastLog = newLogs.last {
                    withAnimation {
                        proxy.scrollTo(lastLog.id, anchor: .bottom)
                    }
                }
            }
        }
    }

    private func logEntryRow(_ log: LogEntry) -> some View {
        HStack(alignment: .top, spacing: 8) {
            Text(log.timestamp, style: .time)
                .font(.system(.caption, design: .monospaced))
                .foregroundColor(.secondary)
                .frame(width: 60, alignment: .leading)

            Image(systemName: log.icon)
                .foregroundColor(log.level.color)
                .frame(width: 16)

            Text(log.message)
                .font(.system(.caption, design: .monospaced))
                .textSelection(.enabled)

            Spacer()
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 2)
        .id(log.id)
    }

    private var clearLogsButton: some View {
        HStack {
            Spacer()
            Button("Clear Logs") {
                logs.removeAll()
            }
            .buttonStyle(.borderless)
            .controlSize(.small)
        }
    }

    private var footerView: some View {
        HStack {
            Text("Click menu bar icon to hide window")
                .font(.caption)
                .foregroundColor(.secondary)
            Spacer()
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
        .background(Color(nsColor: .controlBackgroundColor))
    }

    private var statusIndicatorColor: Color {
        if isRunning {
            return isConnected ? .green : .orange
        }
        return .red
    }

    // MARK: - Configuration

    private func loadConfig() {
        if let cfg = RustBridge.loadConfig() {
            config = cfg
            statusMessage = "Config loaded"
            addLog("Configuration loaded", level: .info)
        } else {
            statusMessage = "No config found"
            addLog("No existing configuration found", level: .warning)
        }
    }

    private func saveConfig() -> Bool {
        if RustBridge.saveConfig(config) {
            statusMessage = "Config saved"
            addLog("Configuration saved", level: .info)
            return true
        } else {
            alertMessage = "Failed to save configuration"
            showAlert = true
            addLog("Failed to save configuration", level: .error)
            return false
        }
    }

    // MARK: - Reporter Control

    private func toggleReporter() {
        if isRunning {
            stopReporter()
        } else {
            startReporter()
        }
    }

    private func startReporter() {
        guard saveConfig() else { return }

        guard let handle = RustBridge.startReporter(config: config) else {
            alertMessage = "Failed to start reporter. Please check your configuration."
            showAlert = true
            addLog("Failed to start reporter", level: .error)
            return
        }

        reporterHandle = handle
        isRunning = true
        statusMessage = "Starting..."
        addLog("Reporter started", level: .info)

        // Update status bar
        updateStatusBar()
    }

    private func stopReporter() {
        if let handle = reporterHandle {
            _ = RustBridge.stopReporter(handle)
            reporterHandle = nil
        }
        isRunning = false
        isConnected = false
        statusMessage = "Stopped"
        lastError = nil
        addLog("Reporter stopped", level: .info)

        // Update status bar
        updateStatusBar()
    }

    // MARK: - Status Updates

    private func startStatusUpdates() {
        statusTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { _ in
            updateStatus()
        }
    }

    private func stopStatusUpdates() {
        statusTimer?.invalidate()
        statusTimer = nil
    }

    private func updateStatus() {
        guard isRunning, let handle = reporterHandle else { return }

        let status = RustBridge.getStatus(handle)
        let wasConnected = isConnected
        isConnected = status.isConnected

        // Update message
        if status.isConnected {
            statusMessage = "Connected"
        } else {
            statusMessage = "Connecting..."
        }

        // Handle errors
        if let error = status.lastError, error != lastError {
            lastError = error
            addLog("Error: \(error)", level: .error)
        }

        // Log connection state changes
        if status.isConnected != wasConnected {
            addLog(status.isConnected ? "Connected to server" : "Disconnected from server", level: .info)
        }

        // Update status bar
        updateStatusBar()
    }

    private func updateStatusBar() {
        appDelegate?.updateStatusBarStatus(isRunning: isRunning, isConnected: isConnected)
    }

    // MARK: - Logging

    private func addLog(_ message: String, level: LogLevel) {
        let log = LogEntry(
            id: UUID(),
            timestamp: Date(),
            message: message,
            level: level
        )
        logs.append(log)

        // Keep only last 500 logs
        if logs.count > 500 {
            logs.removeFirst(logs.count - 500)
        }
    }
}

// MARK: - Log Models

struct LogEntry: Identifiable, Equatable {
    let id: UUID
    let timestamp: Date
    let message: String
    let level: LogLevel

    var icon: String {
        switch level {
        case .info: return "info.circle"
        case .warning: return "exclamationmark.triangle"
        case .error: return "xmark.circle"
        }
    }
}

enum LogLevel: Equatable {
    case info
    case warning
    case error

    var color: Color {
        switch self {
        case .info: return .blue
        case .warning: return .orange
        case .error: return .red
        }
    }
}

#Preview {
    ContentView()
}
