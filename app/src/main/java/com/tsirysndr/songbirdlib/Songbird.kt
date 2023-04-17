package com.tsirysndr.songbirdlib

class Songbird {
    companion object {
        init {
            System.loadLibrary("songbird_android");
        }
        external fun example()
        external fun start()

        external fun start_blocking(socket_path: String)
    }
}