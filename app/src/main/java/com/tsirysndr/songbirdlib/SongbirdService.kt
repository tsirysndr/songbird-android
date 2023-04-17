package com.tsirysndr.songbirdlib

import android.app.Service
import android.content.Intent
import android.os.IBinder
import com.tsirysndr.songbirdlib.Songbird.Companion.start_blocking

class SongbirdService : Service() {
    private var backgroundThread: Thread? = null
    companion object {
        init {
            System.loadLibrary("songbird_android")
        }
    }
    override fun onStartCommand(intent: Intent, flags: Int, startId: Int): Int {
       backgroundThread = Thread {
           val appDir = applicationContext.getExternalFilesDir(null)?.absolutePath
           start_blocking("$appDir/songbird.sock")
       }
        backgroundThread!!.start()
        return START_STICKY
    }

    override fun onDestroy() {
        super.onDestroy()
        backgroundThread?.interrupt()
    }

    override fun onBind(intent: Intent): IBinder? {
        return null
    }
}