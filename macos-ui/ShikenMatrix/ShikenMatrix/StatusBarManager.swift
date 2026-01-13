//
//  StatusBarManager.swift
//  ShikenMatrix
//
//  Menu bar / Status bar management
//

import AppKit
import SwiftUI

/// Manages the status bar item and menu
class StatusBarManager {
    private var statusItem: NSStatusItem?
    private var window: NSWindow?

    init() {
        setupStatusBar()
    }

    /// Set up the status bar item and menu
    private func setupStatusBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        if let button = statusItem?.button {
            // Use SF Symbol for better appearance
            if let image = NSImage(systemSymbolName: "chart.bar.doc.horizontal", accessibilityDescription: "ShikenMatrix") {
                image.isTemplate = true
                button.image = image
            } else {
                button.title = "📊"
            }
            button.toolTip = "ShikenMatrix - 窗口上报工具"
        }

        buildMenu()
    }

    /// Build the menu items
    private func buildMenu() {
        let menu = NSMenu()

        // Show Settings
        let showItem = NSMenuItem(
            title: "显示设置",
            action: #selector(showSettings),
            keyEquivalent: ""
        )
        showItem.target = self
        menu.addItem(showItem)

        // Status indicator
        let statusItem = NSMenuItem(
            title: "状态: 已停止",
            action: nil,
            keyEquivalent: ""
        )
        statusItem.tag = 100  // Tag for updating status
        menu.addItem(statusItem)

        menu.addItem(NSMenuItem.separator())

        // Quit
        let quitItem = NSMenuItem(
            title: "退出",
            action: #selector(quit),
            keyEquivalent: "q"
        )
        quitItem.target = self
        menu.addItem(quitItem)

        self.statusItem?.menu = menu
    }

    /// Set the window reference for show/hide control
    func setWindow(_ window: NSWindow) {
        self.window = window
    }

    /// Update the status text in the menu
    func updateStatus(isRunning: Bool, isConnected: Bool) {
        guard let menu = statusItem?.menu,
              let statusItem = menu.item(withTag: 100) else {
            return
        }

        if isRunning {
            if isConnected {
                statusItem.title = "状态: 已连接"
            } else {
                statusItem.title = "状态: 连接中..."
            }
        } else {
            statusItem.title = "状态: 已停止"
        }
    }

    /// Show the settings window
    @objc private func showSettings() {
        if let window = window {
            window.setIsVisible(true)
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
        }
    }

    /// Hide the settings window
    func hideSettings() {
        if let window = window {
            window.setIsVisible(false)
        }
    }

    /// Quit the application
    @objc private func quit() {
        NSApp.terminate(nil)
    }

    deinit {
        if let statusItem = statusItem {
            NSStatusBar.system.removeStatusItem(statusItem)
        }
    }
}
