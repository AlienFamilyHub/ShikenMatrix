package moe.tnxg.shikenmatrix.mobile.ui

import android.Manifest
import android.app.AppOpsManager
import android.app.ActivityManager
import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager
import android.location.LocationManager
import android.os.Build
import android.os.PowerManager
import android.provider.Settings
import moe.tnxg.shikenmatrix.mobile.nativebridge.ShikenMatrixNotificationListenerService

data class PermissionStatus(
    val coarseLocation: Boolean,
    val usageAccess: Boolean,
    val notificationListener: Boolean,
    val locationEnabled: Boolean,
    val batteryOptimizationExempt: Boolean,
    val backgroundRestricted: Boolean?,
) {
    val allGranted: Boolean
        get() = coarseLocation && usageAccess && notificationListener && locationEnabled
}

fun Context.probePermissionStatus(): PermissionStatus {
    val coarseLocation =
        checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) == PackageManager.PERMISSION_GRANTED

    val usageAccess = run {
        val ops = getSystemService(Context.APP_OPS_SERVICE) as AppOpsManager
        val mode = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            ops.unsafeCheckOpNoThrow(
                AppOpsManager.OPSTR_GET_USAGE_STATS,
                android.os.Process.myUid(),
                packageName,
            )
        } else {
            @Suppress("DEPRECATION")
            ops.checkOpNoThrow(
                AppOpsManager.OPSTR_GET_USAGE_STATS,
                android.os.Process.myUid(),
                packageName,
            )
        }
        mode == AppOpsManager.MODE_ALLOWED
    }

    val notificationListener = run {
        val flat = Settings.Secure.getString(contentResolver, "enabled_notification_listeners").orEmpty()
        val target =
            ComponentName(packageName, ShikenMatrixNotificationListenerService::class.java.name).flattenToString()
        flat.split(':').any { it.equals(target, ignoreCase = true) }
    }

    val locationEnabled = run {
        val lm = getSystemService(Context.LOCATION_SERVICE) as LocationManager
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            lm.isLocationEnabled
        } else {
            @Suppress("DEPRECATION")
            lm.isProviderEnabled(LocationManager.GPS_PROVIDER) ||
                    lm.isProviderEnabled(LocationManager.NETWORK_PROVIDER)
        }
    }

    return PermissionStatus(
        coarseLocation = coarseLocation,
        usageAccess = usageAccess,
        notificationListener = notificationListener,
        locationEnabled = locationEnabled,
        batteryOptimizationExempt = (getSystemService(Context.POWER_SERVICE) as PowerManager)
            .isIgnoringBatteryOptimizations(packageName),
        backgroundRestricted = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            (getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager).isBackgroundRestricted
        } else null,
    )
}
