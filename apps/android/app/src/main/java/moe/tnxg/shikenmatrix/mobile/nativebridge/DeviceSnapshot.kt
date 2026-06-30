package moe.tnxg.shikenmatrix.mobile.nativebridge

import org.json.JSONObject

data class DeviceSnapshot(
  val json: JSONObject,
  val assets: List<DeviceSnapshotAsset>,
) {
  fun stableStateKey(): String =
    JSONObject(json.toString())
      .removeVolatileFields()
      .toString()

  private fun JSONObject.removeVolatileFields(): JSONObject {
    remove("timestampMs")
    optJSONObject("media")?.remove("position")
    return this
  }
}

data class DeviceSnapshotAsset(
  val id: String,
  val mimeType: String,
  val bytes: ByteArray,
) {
  override fun equals(other: Any?): Boolean =
    other is DeviceSnapshotAsset && id == other.id

  override fun hashCode(): Int = id.hashCode()
}
