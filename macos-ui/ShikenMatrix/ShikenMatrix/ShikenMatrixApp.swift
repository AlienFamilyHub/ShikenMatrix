//
//  ShikenMatrixApp.swift
//  ShikenMatrix
//
//  Created by tianxiang on 2026/1/11.
//

import SwiftUI
import AppKit

@main
struct ShikenMatrixApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        // Hide default window, manage via StatusBar
        WindowGroup {
            ContentView()
                .frame(minWidth: 400, minHeight: 300)
                .onAppear {
                    appDelegate.hideWindow()
                }
        }
        .commandsRemoved()
        .windowStyle(.hiddenTitleBar)
    }
}

/// Application delegate to manage startup and status bar
class AppDelegate: NSObject, NSApplicationDelegate {
    var statusBarManager: StatusBarManager?
    var window: NSWindow?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Create status bar manager
        statusBarManager = StatusBarManager()

        // Get the main window
        if let window = NSApp.windows.first {
            self.window = window
            statusBarManager?.setWindow(window)

            // Hide window on startup - start in tray mode
            window.setIsVisible(false)
        }

        // Set activation policy to accessory to avoid dock icon
        // (commented out to keep in dock for now)
        // NSApp.setActivationPolicy(.accessory)
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            showWindow()
        }
        return true
    }

    func showWindow() {
        window?.setIsVisible(true)
        window?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func hideWindow() {
        window?.setIsVisible(false)
    }

    func updateStatusBarStatus(isRunning: Bool, isConnected: Bool) {
        statusBarManager?.updateStatus(isRunning: isRunning, isConnected: isConnected)
    }
}
