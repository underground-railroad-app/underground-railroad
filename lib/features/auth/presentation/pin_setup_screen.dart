import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:underground_railroad/core/constants/app_constants.dart';
import 'package:underground_railroad/shared/providers/app_providers.dart';

class PinSetupScreen extends ConsumerStatefulWidget {
  final bool includeDuressPin;

  const PinSetupScreen({
    super.key,
    this.includeDuressPin = true,
  });

  @override
  ConsumerState<PinSetupScreen> createState() => _PinSetupScreenState();
}

class _PinSetupScreenState extends ConsumerState<PinSetupScreen> {
  final _pinController = TextEditingController();
  final _confirmPinController = TextEditingController();
  final _duressPinController = TextEditingController();

  bool _obscurePin = true;
  bool _obscureConfirmPin = true;
  bool _obscureDuressPin = true;
  bool _isConfirmingPin = false;
  bool _isSettingDuressPin = false;

  String? _errorMessage;

  @override
  void dispose() {
    _pinController.dispose();
    _confirmPinController.dispose();
    _duressPinController.dispose();
    super.dispose();
  }

  void _handleNext() {
    setState(() {
      _errorMessage = null;
    });

    if (!_isConfirmingPin) {
      // Validate main PIN
      if (_pinController.text.length < AppConstants.pinMinLength) {
        setState(() {
          _errorMessage = 'PIN must be at least ${AppConstants.pinMinLength} digits';
        });
        return;
      }

      setState(() {
        _isConfirmingPin = true;
      });
    } else if (_isConfirmingPin && !_isSettingDuressPin) {
      // Confirm main PIN
      if (_pinController.text != _confirmPinController.text) {
        setState(() {
          _errorMessage = 'PINs do not match';
        });
        return;
      }

      if (widget.includeDuressPin) {
        setState(() {
          _isSettingDuressPin = true;
        });
      } else {
        _finishSetup();
      }
    } else {
      // Validate duress PIN
      if (_duressPinController.text.length < AppConstants.pinMinLength) {
        setState(() {
          _errorMessage = 'Duress PIN must be at least ${AppConstants.pinMinLength} digits';
        });
        return;
      }

      if (_duressPinController.text == _pinController.text) {
        setState(() {
          _errorMessage = 'Duress PIN must be different from main PIN';
        });
        return;
      }

      _finishSetup();
    }
  }

  void _finishSetup() async {
    try {
      final securityManager = ref.read(securityManagerProvider);
      await securityManager.initializeWithPIN(
        pin: _pinController.text,
        duressPin: _duressPinController.text.isEmpty ? null : _duressPinController.text,
      );

      // Generate decoy data if duress PIN was set
      if (_duressPinController.text.isNotEmpty) {
        final duressManager = ref.read(duressManagerProvider);
        final dbService = ref.read(databaseServiceProvider);
        final dbKey = await securityManager.getDatabaseKey(isDecoy: true);

        await dbService.initializeDecoyDatabase(dbKey);
        await duressManager.generateDecoyData();
      }

      // Navigate to PIN entry (which will then navigate to app)
      if (mounted) {
        context.go('/pin-entry');
      }
    } catch (e) {
      setState(() {
        _errorMessage = 'Setup failed: ${e.toString()}';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Secure Your App'),
        leading: _isConfirmingPin || _isSettingDuressPin
            ? IconButton(
                icon: const Icon(Icons.arrow_back),
                onPressed: () {
                  setState(() {
                    if (_isSettingDuressPin) {
                      _isSettingDuressPin = false;
                      _duressPinController.clear();
                    } else {
                      _isConfirmingPin = false;
                      _confirmPinController.clear();
                    }
                    _errorMessage = null;
                  });
                },
              )
            : null,
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const SizedBox(height: 32),
              Icon(
                _isSettingDuressPin
                    ? Icons.shield_outlined
                    : Icons.lock_outline,
                size: 64,
                color: Theme.of(context).colorScheme.primary,
              ),
              const SizedBox(height: 24),
              Text(
                _getTitle(),
                style: Theme.of(context).textTheme.headlineSmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 16),
              Text(
                _getSubtitle(),
                style: Theme.of(context).textTheme.bodyMedium,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 48),
              if (!_isConfirmingPin && !_isSettingDuressPin) ...[
                _buildPinField(
                  controller: _pinController,
                  label: 'Enter PIN',
                  obscure: _obscurePin,
                  onObscureToggle: () => setState(() => _obscurePin = !_obscurePin),
                ),
              ] else if (_isConfirmingPin && !_isSettingDuressPin) ...[
                _buildPinField(
                  controller: _confirmPinController,
                  label: 'Confirm PIN',
                  obscure: _obscureConfirmPin,
                  onObscureToggle: () => setState(() => _obscureConfirmPin = !_obscureConfirmPin),
                ),
              ] else ...[
                _buildPinField(
                  controller: _duressPinController,
                  label: 'Enter Duress PIN',
                  obscure: _obscureDuressPin,
                  onObscureToggle: () => setState(() => _obscureDuressPin = !_obscureDuressPin),
                ),
              ],
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
                onPressed: _handleNext,
                child: Text(_isSettingDuressPin ? 'Finish' : 'Next'),
              ),
              if (_isSettingDuressPin) ...[
                const SizedBox(height: 16),
                TextButton(
                  onPressed: () {
                    setState(() {
                      _isSettingDuressPin = false;
                      _duressPinController.clear();
                    });
                    _finishSetup();
                  },
                  child: const Text('Skip Duress PIN'),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  String _getTitle() {
    if (_isSettingDuressPin) {
      return 'Set Duress PIN';
    } else if (_isConfirmingPin) {
      return 'Confirm Your PIN';
    } else {
      return 'Create Your PIN';
    }
  }

  String _getSubtitle() {
    if (_isSettingDuressPin) {
      return 'This PIN will open a decoy account if you\'re under duress. It should be different from your main PIN.';
    } else if (_isConfirmingPin) {
      return 'Enter your PIN again to confirm.';
    } else {
      return 'Choose a secure PIN to protect your data. Minimum ${AppConstants.pinMinLength} digits.';
    }
  }

  Widget _buildPinField({
    required TextEditingController controller,
    required String label,
    required bool obscure,
    required VoidCallback onObscureToggle,
  }) {
    return TextField(
      controller: controller,
      obscureText: obscure,
      keyboardType: TextInputType.number,
      maxLength: AppConstants.pinMaxLength,
      decoration: InputDecoration(
        labelText: label,
        border: const OutlineInputBorder(),
        suffixIcon: IconButton(
          icon: Icon(obscure ? Icons.visibility : Icons.visibility_off),
          onPressed: onObscureToggle,
        ),
      ),
    );
  }
}
