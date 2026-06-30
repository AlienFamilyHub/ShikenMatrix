package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.Manifest
import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Drawable
import android.location.LocationManager
import android.media.session.MediaController
import android.media.session.MediaSessionManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.BatteryManager
import android.os.Build
import org.json.JSONArray
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.security.MessageDigest
import kotlin.math.max
import kotlin.math.roundToInt

class DeviceSnapshotCollector(private val context: Context) {
  private val rootManager = RootManager(context)
  private val assets = mutableListOf<DeviceSnapshotAsset>()

  fun collectSnapshot(): DeviceSnapshot {
    assets.clear()
    val json = collectJson()
    return DeviceSnapshot(json = json, assets = assets.toList())
  }

  fun collectJson(): JSONObject =
    JSONObject()
      .put("foregroundApp", foregroundApp())
      .put("media", media())
      .put("battery", battery())
      .put("network", network())
      .put("coarseLocation", coarseLocation())
      .put("timestampMs", System.currentTimeMillis())

  private fun foregroundApp(): JSONObject {
    val foreground = JSONObject()
    val usageStatsManager = context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
    val endTime = System.currentTimeMillis()
    val events = usageStatsManager.queryEvents(endTime - 60_000, endTime)
    var packageName: String? = null
    val event = UsageEvents.Event()

    while (events.hasNextEvent()) {
      events.getNextEvent(event)
      if (event.eventType == UsageEvents.Event.MOVE_TO_FOREGROUND) {
        packageName = event.packageName
      }
    }

    if (packageName == null) {
      packageName = rootManager
        .runRootCommand("dumpsys window | grep -E 'mCurrentFocus|topResumedActivity'")
        ?.substringAfter("{", "")
        ?.substringBefore("}", "")
        ?.split(" ")
        ?.firstOrNull { it.contains("/") }
        ?.substringBefore("/")
    }

    foreground.put("packageName", packageName)
    foreground.put("label", packageName?.let(::applicationLabel))
    foreground.put("appIcon", packageName?.let { applicationIcon(it, 96, "app-icon") } ?: JSONObject.NULL)
    foreground.put("usageAccessLikelyGranted", packageName != null)
    return foreground
  }

  private fun media(): JSONObject {
    val media = JSONObject()
    val sessionManager = context.getSystemService(Context.MEDIA_SESSION_SERVICE) as MediaSessionManager
    val controller = runCatching {
      sessionManager.getActiveSessions(
        android.content.ComponentName(context, ShikenMatrixNotificationListenerService::class.java),
      ).firstOrNull()
    }.getOrNull()

    val metadata = controller?.metadata
    val playbackState = controller?.playbackState
    media.put("packageName", controller?.packageName)
    media.put("title", metadata?.getString(android.media.MediaMetadata.METADATA_KEY_TITLE))
    media.put("artist", metadata?.getString(android.media.MediaMetadata.METADATA_KEY_ARTIST))
    media.put("album", metadata?.getString(android.media.MediaMetadata.METADATA_KEY_ALBUM))
    media.put("duration", metadata?.getLong(android.media.MediaMetadata.METADATA_KEY_DURATION)?.toDouble() ?: 0.0)
    media.put("position", playbackState?.position?.toDouble() ?: 0.0)
    media.put("state", playbackState?.state ?: 0)
    media.put("appLabel", controller?.packageName?.let(::applicationLabel))
    media.put("appIcon", controller?.packageName?.let { applicationIcon(it, 96, "media-app-icon") } ?: JSONObject.NULL)
    media.put("artwork", metadata?.let(::mediaArtwork) ?: JSONObject.NULL)
    media.put("notificationAccessRequired", controller == null)
    return media
  }

  private fun battery(): JSONObject {
    val battery = JSONObject()
    val batteryManager = context.getSystemService(Context.BATTERY_SERVICE) as BatteryManager
    battery.put("level", batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY))
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      battery.put("charging", batteryManager.isCharging)
    }
    return battery
  }

  private fun network(): JSONObject {
    val network = JSONObject()
    val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    val activeNetwork = connectivityManager.activeNetwork
    val capabilities = activeNetwork?.let(connectivityManager::getNetworkCapabilities)
    network.put("connected", capabilities != null)
    network.put("wifi", capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true)
    network.put("cellular", capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) == true)
    network.put("vpn", capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_VPN) == true)
    return network
  }

  private fun coarseLocation(): JSONObject {
    val location = JSONObject()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M &&
      context.checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) != PackageManager.PERMISSION_GRANTED
    ) {
      location.put("permissionGranted", false)
      return location
    }

    val locationManager = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
    val lastKnownLocation = listOf(LocationManager.NETWORK_PROVIDER, LocationManager.PASSIVE_PROVIDER)
      .firstNotNullOfOrNull { provider -> runCatching { locationManager.getLastKnownLocation(provider) }.getOrNull() }

    location.put("permissionGranted", true)
    location.put("latitude", lastKnownLocation?.latitude ?: 0.0)
    location.put("longitude", lastKnownLocation?.longitude ?: 0.0)
    location.put("accuracy", lastKnownLocation?.accuracy?.toDouble() ?: 0.0)
    return location
  }

  private fun applicationLabel(packageName: String): String? =
    runCatching {
      val packageManager = context.packageManager
      val appInfo = packageManager.getApplicationInfo(packageName, 0)
      packageManager.getApplicationLabel(appInfo).toString()
    }.getOrNull()

  private fun applicationIcon(packageName: String, maxDimension: Int, assetPrefix: String): JSONObject? =
    runCatching {
      val drawable = context.packageManager.getApplicationIcon(packageName)
      encodeDrawable(drawable, maxDimension, Bitmap.CompressFormat.PNG, 90, "$assetPrefix:$packageName")
    }.getOrNull()

  private fun mediaArtwork(metadata: android.media.MediaMetadata): JSONObject? {
    val bitmap = metadata.getBitmap(android.media.MediaMetadata.METADATA_KEY_ART)
      ?: metadata.getBitmap(android.media.MediaMetadata.METADATA_KEY_ALBUM_ART)
      ?: metadata.getBitmap(android.media.MediaMetadata.METADATA_KEY_DISPLAY_ICON)

    return bitmap?.let {
      encodeBitmap(
        bitmap = it,
        maxDimension = 320,
        compressFormat = Bitmap.CompressFormat.JPEG,
        quality = 72,
        stableKey = "media-artwork:${metadata.description?.mediaId ?: metadata.hashCode()}",
      )
    }
  }

  private fun encodeDrawable(
    drawable: Drawable,
    maxDimension: Int,
    compressFormat: Bitmap.CompressFormat,
    quality: Int,
    stableKey: String,
  ): JSONObject {
    val sourceBitmap = if (drawable is BitmapDrawable && drawable.bitmap != null) {
      drawable.bitmap
    } else {
      val width = drawable.intrinsicWidth.takeIf { it > 0 } ?: maxDimension
      val height = drawable.intrinsicHeight.takeIf { it > 0 } ?: maxDimension
      Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888).also { bitmap ->
        val canvas = Canvas(bitmap)
        drawable.setBounds(0, 0, canvas.width, canvas.height)
        drawable.draw(canvas)
      }
    }

    return encodeBitmap(sourceBitmap, maxDimension, compressFormat, quality, stableKey)
  }

  private fun encodeBitmap(
    bitmap: Bitmap,
    maxDimension: Int,
    compressFormat: Bitmap.CompressFormat,
    quality: Int,
    stableKey: String,
  ): JSONObject {
    val longestSide = max(bitmap.width, bitmap.height).coerceAtLeast(1)
    val scale = (maxDimension.toFloat() / longestSide).coerceAtMost(1f)
    val encodedBitmap = if (scale < 1f) {
      Bitmap.createScaledBitmap(
        bitmap,
        (bitmap.width * scale).roundToInt().coerceAtLeast(1),
        (bitmap.height * scale).roundToInt().coerceAtLeast(1),
        true,
      )
    } else {
      bitmap
    }

    val outputStream = ByteArrayOutputStream()
    encodedBitmap.compress(compressFormat, quality, outputStream)
    val bytes = outputStream.toByteArray()
    val encodedWidth = encodedBitmap.width
    val encodedHeight = encodedBitmap.height
    if (encodedBitmap !== bitmap) {
      encodedBitmap.recycle()
    }

    val mimeType = when (compressFormat) {
      Bitmap.CompressFormat.PNG -> "image/png"
      Bitmap.CompressFormat.WEBP,
      Bitmap.CompressFormat.WEBP_LOSSLESS,
      Bitmap.CompressFormat.WEBP_LOSSY,
      -> "image/webp"
      else -> "image/jpeg"
    }
    val assetId = "${stableKey}:${bytes.sha256Hex().take(16)}"
    assets.add(DeviceSnapshotAsset(id = assetId, mimeType = mimeType, bytes = bytes))

    return JSONObject()
      .put("mimeType", mimeType)
      .put("width", encodedWidth)
      .put("height", encodedHeight)
      .put("contentItemIdentifier", assetId)
  }

  private fun ByteArray.sha256Hex(): String =
    MessageDigest.getInstance("SHA-256")
      .digest(this)
      .joinToString("") { byte -> "%02x".format(byte) }
}
