package com.tsirysndr.songbird

class Songbird {
    companion object {
        init {
            System.loadLibrary("songbird_android");
        }
        external fun example()
    }
}