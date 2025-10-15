import 'dart:io';
import 'package:path/path.dart';
import 'package:path_provider/path_provider.dart';

/// Platform-specific directory utilities
class PlatformDirectories {
  /// Get the application data directory
  /// - macOS/Linux: ~/.underground-railroad
  /// - iOS/Android: Uses standard app documents directory
  static Future<Directory> getAppDataDirectory() async {
    if (Platform.isMacOS || Platform.isLinux) {
      // Use home directory with hidden folder
      final home = Platform.environment['HOME'];
      if (home == null) {
        throw StateError('HOME environment variable not set');
      }
      final dir = Directory(join(home, '.underground-railroad'));
      if (!await dir.exists()) {
        await dir.create(recursive: true);
      }
      return dir;
    } else {
      // iOS/Android: use standard documents directory
      return await getApplicationDocumentsDirectory();
    }
  }
}
