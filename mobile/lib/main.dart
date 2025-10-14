import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'dart:io' show Platform, Directory, File;
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'screens/onboarding_screen.dart';
import 'screens/home_screen.dart';
import 'screens/emergency_screen.dart';
import 'screens/safe_house_screen.dart';
import 'screens/contacts_screen.dart';
import 'state/app_state.dart';
import 'ffi/frb_generated.dart';

void main() async {
  // Ensure Flutter bindings are initialized
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize Rust FFI (native platforms only)
  try {
    // Load the FFI library
    // Try multiple locations to find it
    final libraryName = Platform.isMacOS || Platform.isIOS
        ? 'libunderground_railroad_ffi.dylib'
        : Platform.isWindows
            ? 'underground_railroad_ffi.dll'
            : 'libunderground_railroad_ffi.so';

    // Use absolute path from project root
    final exePath = Platform.resolvedExecutable;
    final exeDir = Directory(exePath).parent;

    // Try Frameworks directory first (bundled in app)
    final frameworksPath = '${exeDir.parent.path}/Frameworks/$libraryName';

    if (File(frameworksPath).existsSync()) {
      await RustLib.init(
        externalLibrary: ExternalLibrary.open(frameworksPath),
      );
      debugPrint('✅ FFI loaded from: $frameworksPath');
    } else {
      // Fallback to development path
      await RustLib.init();
      debugPrint('✅ FFI loaded from default path');
    }

    debugPrint('   Data will persist across sessions!');
  } catch (e) {
    debugPrint('⚠️ FFI not loaded: $e');
    debugPrint('   App will work but data won\'t persist across restarts');
    debugPrint('   Run: ./build_and_bundle.sh <platform> to fix this');
  }

  runApp(
    ChangeNotifierProvider(
      create: (context) => AppState(),
      child: const UndergroundRailroadApp(),
    ),
  );
}

class UndergroundRailroadApp extends StatelessWidget {
  const UndergroundRailroadApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Underground Railroad',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.deepPurple,
          brightness: Brightness.light,
        ),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.deepPurple,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: Consumer<AppState>(
        builder: (context, appState, child) {
          if (!appState.isInitialized) {
            return const OnboardingScreen();
          }
          return const HomeScreen();
        },
      ),
    );
  }
}
