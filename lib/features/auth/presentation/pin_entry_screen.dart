import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:underground_railroad/core/constants/app_constants.dart';
import 'package:underground_railroad/core/security/security_manager.dart';
import 'package:underground_railroad/core/services/message_listener_service.dart';
import 'package:underground_railroad/core/services/notification_service.dart';
import 'package:underground_railroad/shared/providers/app_providers.dart';

class PinEntryScreen extends ConsumerStatefulWidget {
  const PinEntryScreen({super.key});

  @override
  ConsumerState<PinEntryScreen> createState() => _PinEntryScreenState();
}

class _PinEntryScreenState extends ConsumerState<PinEntryScreen> {
  final _pinController = TextEditingController();
  bool _obscurePin = true;
  bool _isLoading = false;
  String? _errorMessage;
  int _failedAttempts = 0;

  @override
  void dispose() {
    _pinController.dispose();
    super.dispose();
  }

  Future<void> _handleUnlock() async {
    setState(() {
      _errorMessage = null;
      _isLoading = true;
    });

    try {
      // TODO: Get security manager from provider
      // final securityManager = ref.read(securityManagerProvider);
      // final result = await securityManager.verifyPIN(_pinController.text);

      // Placeholder verification
      final result = PINVerificationResult.invalid;

      switch (result) {
        case PINVerificationResult.real:
          // Load real database and navigate to app
          _navigateToApp(isDuressMode: false);
          break;

        case PINVerificationResult.duress:
          // Load decoy database and navigate to app in duress mode
          _navigateToApp(isDuressMode: true);
          break;

        case PINVerificationResult.invalid:
          _failedAttempts++;

          if (_failedAttempts >= AppConstants.maxFailedAttempts) {
            _handleMaxFailedAttempts();
          } else {
            setState(() {
              _errorMessage = 'Invalid PIN. ${AppConstants.maxFailedAttempts - _failedAttempts} attempts remaining.';
              _isLoading = false;
            });
          }
          break;
      }
    } catch (e) {
      setState(() {
        _errorMessage = 'An error occurred. Please try again.';
        _isLoading = false;
      });
    }
  }

  void _navigateToApp({required bool isDuressMode}) async {
    // Set authentication state
    ref.read(authStateProvider.notifier).setAuthenticated(
      isDuressMode: isDuressMode,
    );

    // Initialize database with appropriate key
    final securityManager = ref.read(securityManagerProvider);
    final dbService = ref.read(databaseServiceProvider);

    try {
      final dbKey = await securityManager.getDatabaseKey(isDecoy: isDuressMode);
      await dbService.initializeRealDatabase(dbKey);

      if (isDuressMode) {
        await dbService.initializeDecoyDatabase(dbKey);
        await dbService.switchToDecoyMode();
      }

      // Initialize Veilid
      final veilidService = ref.read(veilidServiceProvider);
      await veilidService.initialize();

      // Start message listener
      final messageListener = ref.read(messageListenerServiceProvider);
      await messageListener.startListening();

      // Initialize notifications
      final notificationService = ref.read(notificationServiceProvider);
      await notificationService.initialize();

      if (mounted) {
        context.go('/contacts');
      }
    } catch (e) {
      setState(() {
        _errorMessage = 'Failed to initialize: ${e.toString()}';
        _isLoading = false;
      });
    }
  }

  void _handleMaxFailedAttempts() {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => AlertDialog(
        title: const Text('Security Alert'),
        content: const Text(
          'Maximum failed attempts reached. The app will now restart.',
        ),
        actions: [
          FilledButton(
            onPressed: () {
              // TODO: Clear any cached data
              // TODO: Restart app or exit
            },
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Icon(
                Icons.lock_outline,
                size: 80,
                color: Theme.of(context).colorScheme.primary,
              ),
              const SizedBox(height: 32),
              Text(
                'Welcome Back',
                style: Theme.of(context).textTheme.headlineMedium,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),
              Text(
                'Enter your PIN to unlock',
                style: Theme.of(context).textTheme.bodyLarge,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 48),
              TextField(
                controller: _pinController,
                obscureText: _obscurePin,
                keyboardType: TextInputType.number,
                maxLength: AppConstants.pinMaxLength,
                enabled: !_isLoading,
                decoration: InputDecoration(
                  labelText: 'PIN',
                  border: const OutlineInputBorder(),
                  suffixIcon: IconButton(
                    icon: Icon(_obscurePin ? Icons.visibility : Icons.visibility_off),
                    onPressed: () => setState(() => _obscurePin = !_obscurePin),
                  ),
                ),
                onSubmitted: (_) => _handleUnlock(),
              ),
              if (_errorMessage != null) ...[
                const SizedBox(height: 16),
                Text(
                  _errorMessage!,
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.error,
                  ),
                  textAlign: TextAlign.center,
                ),
              ],
              const SizedBox(height: 32),
              FilledButton(
                onPressed: _isLoading ? null : _handleUnlock,
                child: _isLoading
                    ? const SizedBox(
                        height: 20,
                        width: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Text('Unlock'),
              ),
              const SizedBox(height: 16),
              OutlinedButton.icon(
                onPressed: _isLoading ? null : _handleBiometricAuth,
                icon: const Icon(Icons.fingerprint),
                label: const Text('Use Biometric'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _handleBiometricAuth() async {
    // TODO: Implement biometric authentication
    // final localAuth = LocalAuthentication();
    // final canAuthenticateWithBiometrics = await localAuth.canCheckBiometrics;
    // if (canAuthenticateWithBiometrics) {
    //   final didAuthenticate = await localAuth.authenticate(
    //     localizedReason: 'Please authenticate to unlock',
    //   );
    //   if (didAuthenticate) {
    //     // Load stored PIN from secure storage and verify
    //   }
    // }
  }
}
