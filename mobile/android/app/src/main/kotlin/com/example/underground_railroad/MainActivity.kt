package com.example.underground_railroad

import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity

class MainActivity : FlutterActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // JNI initialization happens automatically when our FFI library loads
        // See ffi/src/lib.rs JNI_OnLoad function
        android.util.Log.i("UndergroundRailroad", "MainActivity created - Veilid will initialize via FFI")
    }
}
