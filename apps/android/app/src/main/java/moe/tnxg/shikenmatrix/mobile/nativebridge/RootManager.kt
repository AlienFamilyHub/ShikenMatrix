package moe.tnxg.shikenmatrix.mobile.nativebridge

import android.content.Context
import android.content.Intent
import org.json.JSONObject

class RootManager(private val context: Context) {
    private val managerCandidates = listOf(
        RootManagerCandidate("Magisk", "com.topjohnwu.magisk"),
        RootManagerCandidate("Magisk Alpha", "io.github.vvb2060.magisk"),
        RootManagerCandidate("Kitsune Mask / Magisk Delta", "io.github.huskydg.magisk"),
        RootManagerCandidate("KernelSU", "me.weishu.kernelsu"),
        RootManagerCandidate("KSU Next", "com.rifsxd.ksunext"),
        RootManagerCandidate("KSU Next", "io.github.rifsxd.ksunext"),
        RootManagerCandidate("SukiSU Ultra", "com.sukisu.ultra"),
        RootManagerCandidate("APatch", "me.bmax.apatch"),
    )

    fun requestRoot(): JSONObject {
        val processResult = runCatching {
            Runtime.getRuntime()
                .exec(arrayOf("su", "-c", "id"))
                .inputStream
                .bufferedReader()
                .readText()
        }

        val granted = processResult.getOrNull()?.contains("uid=0") == true
        return JSONObject()
            .put("granted", granted)
            .put("message", processResult.getOrElse { it.message ?: "root request failed" })
            .put("managerOpened", if (granted) false else openRootManager())
    }

    fun openRootManager(): JSONObject {
        val packageManager = context.packageManager
        val matchedCandidate = managerCandidates
            .firstNotNullOfOrNull { candidate ->
                packageManager.getLaunchIntentForPackage(candidate.packageName)
                    ?.let { launchIntent -> candidate to launchIntent }
            }
            ?: return JSONObject()
                .put("opened", false)
                .put("message", "未找到已安装且可启动的 Root 管理器")

        val (candidate, launchIntent) = matchedCandidate
        launchIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        context.startActivity(launchIntent)
        return JSONObject()
            .put("opened", true)
            .put("name", candidate.name)
            .put("packageName", candidate.packageName)
    }

    fun runRootCommand(command: String): String? =
        runCatching {
            Runtime.getRuntime()
                .exec(arrayOf("su", "-c", command))
                .inputStream
                .bufferedReader()
                .readText()
                .trim()
        }.getOrNull()

    private data class RootManagerCandidate(
        val name: String,
        val packageName: String,
    )
}
