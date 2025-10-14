import 'package:flutter/foundation.dart';
import 'package:veilid/veilid.dart';

/// Service for initializing and managing Veilid connection
class VeilidService {
  static final VeilidService _instance = VeilidService._internal();
  factory VeilidService() => _instance;
  VeilidService._internal();

  bool _initialized = false;
  bool _connected = false;
  Stream<VeilidUpdate>? _updateStream;

  bool get isInitialized => _initialized;
  bool get isConnected => _connected;

  /// Get platform configuration (logging only - for initializeVeilidCore)
  Future<Map<String, dynamic>> _getPlatformConfig(String appName) async {
    return {
      'logging': {
        'terminal': {
          'enabled': kDebugMode,
          'level': kDebugMode ? 'Debug' : 'Info',
          'ignore_log_targets': []
        },
        'otlp': {
          'enabled': false,
          'level': 'Trace',
          'grpc_endpoint': '127.0.0.1:4317',
          'service_name': appName,
          'ignore_log_targets': []
        },
        'api': {
          'enabled': true,
          'level': 'Info',
          'ignore_log_targets': []
        },
        'flame': {
          'enabled': false,
          'path': ''
        }
      }
    };
  }

  /// Initialize Veilid for mobile/desktop (VeilidChat pattern)
  Future<void> initialize(String appName) async {
    if (_initialized) {
      debugPrint('Veilid already initialized');
      return;
    }

    try {
      debugPrint('🌐 Initializing Veilid...');

      // STEP 1: Initialize platform with LOGGING config only
      try {
        final platformConfig = await _getPlatformConfig(appName);
        Veilid.instance.initializeVeilidCore(platformConfig);
        debugPrint('   ✅ Platform initialized');
      } on VeilidAPIExceptionAlreadyInitialized {
        debugPrint('   ℹ️  Platform already initialized');
      }

      // STEP 2: Startup Veilid core with FULL config
      final fullConfig = await getDefaultVeilidConfig(
        isWeb: false,
        programName: appName,
      );

      try {
        _updateStream = await Veilid.instance.startupVeilidCore(fullConfig);
        debugPrint('   ✅ Veilid core started');
      } on VeilidAPIExceptionAlreadyInitialized {
        debugPrint('   ℹ️  Restarting Veilid core...');
        await Veilid.instance.shutdownVeilidCore();
        _updateStream = await Veilid.instance.startupVeilidCore(fullConfig);
        debugPrint('   ✅ Veilid core restarted');
      }

      // Listen for updates
      _updateStream!.listen((update) {
        _handleVeilidUpdate(update);
      });

      // STEP 3: Attach to network
      await Veilid.instance.attach();
      debugPrint('   ✅ Attach requested (waiting for network...)');

      _initialized = true;
      // Don't set _connected yet - wait for VeilidUpdateAttachment event
      _connected = false;

      debugPrint('⏳ Veilid initialized, waiting for network attachment...');
    } catch (e) {
      debugPrint('❌ Veilid initialization failed: $e');
      debugPrint('   App will work offline');
      _initialized = false;
      _connected = false;
      // Don't rethrow - app should work without Veilid
    }
  }

  /// Handle Veilid network updates
  void _handleVeilidUpdate(VeilidUpdate update) {
    if (update is VeilidLog) {
      debugPrint('[Veilid] ${update.logLevel}: ${update.message}');
    } else if (update is VeilidUpdateAttachment) {
      // Check if attached (any state except detached/detaching/attaching)
      final wasConnected = _connected;
      _connected = update.state != AttachmentState.detached &&
                   update.state != AttachmentState.detaching &&
                   update.state != AttachmentState.attaching;

      // Log state changes prominently
      final icon = _connected ? "🟢" : "🔴";
      debugPrint('[Veilid] Attachment: ${update.state.name} $icon');

      if (!wasConnected && _connected) {
        debugPrint('🎉 Veilid connected to network!');
      } else if (wasConnected && !_connected) {
        debugPrint('⚠️ Veilid disconnected from network');
      }
    } else if (update is VeilidUpdateNetwork) {
      debugPrint('[Veilid] Network: ${update.started ? "started" : "stopped"}');
    } else if (update is VeilidAppMessage) {
      debugPrint('[Veilid] App message received');
    } else if (update is VeilidUpdateValueChange) {
      debugPrint('[Veilid] DHT value changed: ${update.key}');
    }
  }

  /// Shutdown Veilid
  Future<void> shutdown() async {
    if (!_initialized) return;

    try {
      debugPrint('Shutting down Veilid...');
      await Veilid.instance.detach();
      await Veilid.instance.shutdownVeilidCore();
      _initialized = false;
      _connected = false;
      debugPrint('✅ Veilid shutdown complete');
    } catch (e) {
      debugPrint('⚠️ Veilid shutdown error: $e');
    }
  }
}
