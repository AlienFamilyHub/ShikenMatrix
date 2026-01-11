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
            // Use a simple icon or text for the status bar
            button.title = "SM"
            button.toolTip = "ShikenMatrix - Window Reporter"
        }

        buildMenu()
    }

    /// Build the menu items
    private func buildMenu() {
        let menu = NSMenu()

        // Show Settings
        let showItem = NSMenuItem(
            title: "Show Settings",
            action: #selector(showSettings),
            keyEquivalent: ""
        )
        showItem.target = self
        menu.addItem(showItem)

        // Status indicator
        let statusItem = NSMenuItem(
            title: "Status: Stopped",
            action: nil,
            keyEquivalent: ""
        )
        statusItem.tag = 100  // Tag for updating status
        menu.addItem(statusItem)

        menu.addItem(NSMenuItem.separator())

        // Quit
        let quitItem = NSMenuItem(
            title: "Quit",
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
                statusItem.title = "Status: Connected"
            } else {
                statusItem.title = "Status: Connecting..."
            }
        } else {
            statusItem.title = "Status: Stopped"
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
