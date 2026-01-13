//
//  ShikenMatrixApp.swift
//  ShikenMatrix
//
//  Created by tianxiang on 2026/1/11.
//

import SwiftUI
import AppKit
import UserNotifications

@main
struct ShikenMatrixApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        // 使用 Settings 场景，避免多窗口问题
        Settings {
            EmptyView()
        }
    }
}

/// Application delegate to manage startup and status bar
class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    var statusBarManager: StatusBarManager?
    var window: NSWindow?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // 防止重复初始化
        guard window == nil else { return }
        
        // Set activation policy to regular to show dock icon (change back to .accessory to hide)
        NSApp.setActivationPolicy(.regular)
        
        // Request notification permission
        requestNotificationPermission()
        
        // Create status bar manager first
        statusBarManager = StatusBarManager()
        
        // Create and configure the main window manually
        createMainWindow()
        
        // Show notification that app is running in tray
        showStartupNotification()
    }
    
    private func requestNotificationPermission() {
        let center = UNUserNotificationCenter.current()
        center.requestAuthorization(options: [.alert, .sound]) { granted, error in
            if let error = error {
                print("通知权限请求失败: \(error.localizedDescription)")
            }
        }
    }
    
    private func showStartupNotification() {
        let center = UNUserNotificationCenter.current()
        
        let content = UNMutableNotificationContent()
        content.title = "ShikenMatrix"
        content.body = "应用已在系统托盘启动，点击托盘图标打开设置"
        
        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil // Deliver immediately
        )
        
        center.add(request) { error in
            if let error = error {
                print("通知发送失败: \(error.localizedDescription)")
            }
        }
    }
    
    private func createMainWindow() {
        // Create window with ContentView
        let contentView = ContentView()
        let hostingController = NSHostingController(rootView: contentView)
        
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 400),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        
        window.title = "ShikenMatrix"
        window.contentViewController = hostingController
        window.delegate = self
        window.center()
        
        // Configure window behavior
        window.level = .normal
        window.collectionBehavior = [.canJoinAllSpaces]
        
        // Disable window restoration to avoid className=null warnings
        window.isRestorable = false
        
        // Hide window on startup - start in tray mode
        window.setIsVisible(false)
        // Disable UI updates initially to save resources in tray mode
        RustBridge.setUpdatesEnabled(false)
        
        self.window = window
        statusBarManager?.setWindow(window)
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        // 重用现有窗口，不创建新窗口
        if !flag, let window = window {
            window.setIsVisible(true)
            RustBridge.setUpdatesEnabled(true)
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
        }
        return true
    }
    
    // Intercept window close to hide instead of quit
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        print("❎ Window close requested. Minimizing to tray.")
        hideWindow()
        return false  // Don't actually close the window
    }

    func showWindow() {
        if let window = window {
            print("📈 Window shown: Re-enabling UI updates.")
            window.setIsVisible(true)
            // Re-enable updates when window is shown
            RustBridge.setUpdatesEnabled(true)
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
        }
    }

    func hideWindow() {
        print("📉 Window hidden: Disabling UI updates and clearing cache...")
        // Disable updates when window is hidden to free memory
        RustBridge.setUpdatesEnabled(false)
        window?.setIsVisible(false)
    }

    func updateStatusBarStatus(isRunning: Bool, isConnected: Bool) {
        statusBarManager?.updateStatus(isRunning: isRunning, isConnected: isConnected)
    }

    // MARK: - Cleanup
    func applicationWillTerminate(_ notification: Notification) {
        print("🛑 AppDelegate: Application will terminate, cleaning up...")
        // Clear window delegate to prevent crashes
        window?.delegate = nil
        // Clear references
        window = nil
        statusBarManager = nil
        print("✅ AppDelegate: Cleanup completed")
    }

    deinit {
        print("♻️ AppDelegate deinit: Cleaning up...")
        window?.delegate = nil
        window = nil
        statusBarManager = nil
    }
}
