/// Build script for Rust FFI library
/// This ensures the FFI library is built before Flutter runs

import 'dart:io';

Future<void> main(List<String> args) async {
  print('🦀 Building Rust FFI library...');
  
  // Get project root (mobile/tool -> mobile -> root)
  final toolDir = Directory.current;
  final mobileDir = toolDir.parent;
  final projectRoot = mobileDir.parent;
  
  // Build the FFI library
  final result = await Process.run(
    'cargo',
    ['build', '--release', '-p', 'underground-railroad-ffi'],
    workingDirectory: projectRoot.path,
  );
  
  if (result.exitCode != 0) {
    print('❌ Failed to build FFI library:');
    print(result.stderr);
    exit(1);
  }
  
  print('✅ FFI library built successfully!');
  
  // Copy the library for easy access during development
  final dylibSource = File('${projectRoot.path}/target/release/libunderground_railroad_ffi.dylib');
  final dylibDest = File('${mobileDir.path}/macos/libunderground_railroad_ffi.dylib');
  
  if (dylibSource.existsSync()) {
    await dylibSource.copy(dylibDest.path);
    print('📋 Copied FFI library to mobile/macos/');
  }
}
