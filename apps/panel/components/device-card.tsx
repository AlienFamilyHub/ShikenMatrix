'use client'

import { type DeviceInfo, present, anyPresent } from '@/lib/status-data'

function batteryIcon(level: number, charging: boolean): string {
  if (charging) return 'battery_charging_full'
  if (level >= 95) return 'battery_full'
  if (level >= 80) return 'battery_6_bar'
  if (level >= 60) return 'battery_5_bar'
  if (level >= 45) return 'battery_4_bar'
  if (level >= 30) return 'battery_3_bar'
  if (level >= 15) return 'battery_2_bar'
  return 'battery_1_bar'
}

export function DeviceCard({ device }: { device?: DeviceInfo | null }) {
  if (
    !device ||
    !anyPresent(
      device.battery_level,
      device.battery_charging,
      device.network_wifi,
      device.network_cellular,
      device.network_vpn,
      device.latitude,
      device.longitude,
    )
  ) {
    return null
  }

  const hasLocation = present(device.latitude) && present(device.longitude)

  return (
    <md-outlined-card class="info-card device-card">
      <header className="card-header">
        <md-icon>smartphone</md-icon>
        <h3 className="md-typescale-title-medium">设备信息</h3>
      </header>

      <div className="device-grid">
        {present(device.battery_level) && (
          <div className="device-stat">
            <md-icon>
              {batteryIcon(
                device.battery_level as number,
                device.battery_charging === true,
              )}
            </md-icon>
            <div className="device-stat-text">
              <span className="md-typescale-title-medium">
                {device.battery_level}%
              </span>
              <span className="md-typescale-label-small device-stat-label">
                {device.battery_charging === true ? '充电中' : '电量'}
              </span>
            </div>
          </div>
        )}

        {present(device.network_wifi) && (
          <div className="device-stat">
            <md-icon>{device.network_wifi ? 'wifi' : 'wifi_off'}</md-icon>
            <div className="device-stat-text">
              <span className="md-typescale-title-medium">
                {device.network_wifi ? '已连接' : '未连接'}
              </span>
              <span className="md-typescale-label-small device-stat-label">
                Wi-Fi
              </span>
            </div>
          </div>
        )}

        {present(device.network_cellular) && (
          <div className="device-stat">
            <md-icon>
              {device.network_cellular ? 'signal_cellular_alt' : 'mobiledata_off'}
            </md-icon>
            <div className="device-stat-text">
              <span className="md-typescale-title-medium">
                {device.network_cellular ? '已连接' : '未连接'}
              </span>
              <span className="md-typescale-label-small device-stat-label">
                蜂窝网络
              </span>
            </div>
          </div>
        )}

        {present(device.network_vpn) && (
          <div className="device-stat">
            <md-icon>{device.network_vpn ? 'vpn_lock' : 'vpn_key_off'}</md-icon>
            <div className="device-stat-text">
              <span className="md-typescale-title-medium">
                {device.network_vpn ? '已开启' : '关闭'}
              </span>
              <span className="md-typescale-label-small device-stat-label">
                VPN
              </span>
            </div>
          </div>
        )}

        {hasLocation && (
          <div className="device-stat device-stat-wide">
            <md-icon>location_on</md-icon>
            <div className="device-stat-text">
              <span className="md-typescale-title-medium">
                {(device.latitude as number).toFixed(4)},{' '}
                {(device.longitude as number).toFixed(4)}
              </span>
              <span className="md-typescale-label-small device-stat-label">
                粗略位置
              </span>
            </div>
          </div>
        )}
      </div>
    </md-outlined-card>
  )
}
