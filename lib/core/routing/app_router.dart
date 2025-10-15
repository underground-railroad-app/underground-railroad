import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:underground_railroad/features/auth/presentation/pin_entry_screen.dart';
import 'package:underground_railroad/features/auth/presentation/pin_setup_screen.dart';
import 'package:underground_railroad/features/auth/presentation/splash_screen.dart';
import 'package:underground_railroad/features/contacts/presentation/contacts_screen.dart';
import 'package:underground_railroad/features/messaging/presentation/chat_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    debugLogDiagnostics: true,
    initialLocation: '/splash',
    routes: [
      GoRoute(
        path: '/splash',
        name: 'splash',
        builder: (context, state) => const SplashScreen(),
      ),
      GoRoute(
        path: '/pin-setup',
        name: 'pin-setup',
        builder: (context, state) => const PinSetupScreen(),
      ),
      GoRoute(
        path: '/pin-entry',
        name: 'pin-entry',
        builder: (context, state) => const PinEntryScreen(),
      ),
      GoRoute(
        path: '/contacts',
        name: 'contacts',
        builder: (context, state) => const ContactsScreen(),
      ),
      GoRoute(
        path: '/chat/:contactId',
        name: 'chat',
        builder: (context, state) {
          final contactId = state.pathParameters['contactId']!;
          return ChatScreen(contactId: contactId);
        },
      ),
      // TODO: Add settings, alerts, and other routes
    ],
    errorBuilder: (context, state) => Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 64, color: Colors.red),
            const SizedBox(height: 16),
            Text(
              'Route not found: ${state.uri.path}',
              style: Theme.of(context).textTheme.titleLarge,
            ),
          ],
        ),
      ),
    ),
  );
});
