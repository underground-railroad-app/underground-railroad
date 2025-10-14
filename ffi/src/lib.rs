mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

/// FFI bridge for Flutter mobile app
///
/// This crate provides C-compatible FFI functions that Flutter can call.
/// It wraps the Underground Railroad core library.

pub mod api;

// Android JNI initialization
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn JNI_OnLoad(_vm: jni::JavaVM, _reserved: *mut std::os::raw::c_void) -> jni::sys::jint {
    // Initialize Android logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("UndergroundRailroad"),
    );

    log::info!("Underground Railroad FFI loaded on Android");
    log::info!("JNI_OnLoad called - ready for Veilid when supported");

    // TODO: Initialize veilid-core Android globals
    // This requires proper veilid-core Android setup
    // See VEILID_MOBILE_SETUP.md for details

    // Return JNI version
    jni::sys::JNI_VERSION_1_6
}
