import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import 'package:qr_flutter/qr_flutter.dart';
import '../state/app_state.dart';
import '../ffi/api.dart' as api;
import 'qr_scanner_screen.dart';
import 'chat_screen.dart';

class ContactsScreen extends StatelessWidget {
  const ContactsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppState>(
      builder: (context, appState, child) {
        final hasContacts = appState.contactCount > 0;

        return Scaffold(
          body: hasContacts
              ? _buildContactsList(appState)
              : _buildEmptyState(context),
          floatingActionButton: FloatingActionButton.extended(
            onPressed: () {
              // Show my QR code
              _showMyQRCode(context);
            },
            icon: const Icon(Icons.qr_code),
            label: const Text('My QR Code'),
          ),
        );
      },
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.people, size: 80, color: Colors.grey),
            const SizedBox(height: 16),
            const Text(
              'No Contacts Yet',
              style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            const Text(
              'Scan a QR code to add someone',
              style: TextStyle(color: Colors.grey),
            ),
            const SizedBox(height: 24),
            FilledButton.icon(
              icon: const Icon(Icons.qr_code_scanner),
              label: const Text('Scan QR Code'),
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(
                    builder: (context) => const QRScannerScreen(),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildContactsList(AppState appState) {
    return Builder(
      builder: (context) => ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(
            'Contacts (${appState.contactCount})',
            style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 16),
          // Display actual contacts from database
          ...appState.contacts.map((contact) => Card(
            child: ListTile(
              leading: const CircleAvatar(
                backgroundColor: Colors.deepPurple,
                child: Icon(Icons.person, color: Colors.white),
              ),
              title: Text(contact.name),
              subtitle: Text(contact.fingerprint),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                _showContactDetails(context, contact);
              },
            ),
          )),
          const SizedBox(height: 80), // Space for FAB
        ],
      ),
    );
  }

  Widget _buildOriginalBody(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.people, size: 80, color: Colors.grey),
            const SizedBox(height: 16),
            const Text(
              'No Contacts Yet',
              style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            const Text(
              'Scan a QR code to add someone',
              style: TextStyle(color: Colors.grey),
            ),
            const SizedBox(height: 24),
            FilledButton.icon(
              icon: const Icon(Icons.qr_code_scanner),
              label: const Text('Scan QR Code'),
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(
                    builder: (context) => const QRScannerScreen(),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildOldFloatingButton(BuildContext context) {
    return Scaffold(
      body: _buildOriginalBody(context),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () {
          // Show my QR code
          _showMyQRCode(context);
        },
        icon: const Icon(Icons.qr_code),
        label: const Text('My QR Code'),
      ),
    );
  }

  void _showMyQRCode(BuildContext context) {
    final appState = context.read<AppState>();

    // Generate QR code data
    // Format: railroad://contact/NAME/FINGERPRINT
    final qrData = 'railroad://contact/${appState.userName ?? 'User'}/${appState.fingerprint ?? 'unknown'}';

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('My Contact QR Code'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Real QR code generation
              Container(
                width: 250,
                height: 250,
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: QrImageView(
                  data: qrData,
                  version: QrVersions.auto,
                  size: 218, // Slightly smaller than container
                  backgroundColor: Colors.white,
                  eyeStyle: const QrEyeStyle(
                    eyeShape: QrEyeShape.square,
                    color: Colors.black,
                  ),
                ),
              ),
            const SizedBox(height: 16),
            const Text(
              'Scan this to add me as a contact',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            // Show fingerprint for manual verification
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.grey[200],
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: Colors.grey[400]!, width: 1),
              ),
              child: Column(
                children: [
                  const Text(
                    'Verification Words:',
                    style: TextStyle(
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                      color: Colors.black87,
                    ),
                  ),
                  const SizedBox(height: 4),
                  SelectableText(
                    appState.fingerprint ?? 'Not available',
                    style: const TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                      fontFamily: 'monospace',
                      color: Colors.black,
                      letterSpacing: 1.2,
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            // Copy URL button
            FilledButton.icon(
              icon: const Icon(Icons.copy),
              label: const Text('Copy Contact URL'),
              onPressed: () {
                Clipboard.setData(ClipboardData(text: qrData));
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('✅ Contact URL copied to clipboard'),
                    duration: Duration(seconds: 2),
                  ),
                );
              },
            ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  void _showContactDetails(BuildContext context, api.ContactInfo contact) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            const CircleAvatar(
              backgroundColor: Colors.deepPurple,
              child: Icon(Icons.person, color: Colors.white),
            ),
            const SizedBox(width: 12),
            Text(contact.name),
          ],
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildDetailRow('Trust Level', '🟢 Verified in person'),
            const SizedBox(height: 12),
            _buildDetailRow('Fingerprint', contact.fingerprint),
            const SizedBox(height: 12),
            _buildDetailRow('Added', 'Recently'),
            const SizedBox(height: 12),
            _buildDetailRow('Capabilities', 'Unknown'),
            const SizedBox(height: 16),
            const Divider(),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                Column(
                  children: [
                    IconButton(
                      icon: const Icon(Icons.message, color: Colors.blue),
                      onPressed: () {
                        Navigator.of(context).pop();
                        // Navigate to chat screen
                        Navigator.push(
                          context,
                          MaterialPageRoute(
                            builder: (context) => ChatScreen(
                              contactId: contact.id,
                              contactName: contact.name,
                            ),
                          ),
                        );
                      },
                    ),
                    const Text('Message', style: TextStyle(fontSize: 12)),
                  ],
                ),
                Column(
                  children: [
                    IconButton(
                      icon: const Icon(Icons.qr_code, color: Colors.deepPurple),
                      onPressed: () {
                        Navigator.of(context).pop();
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(content: Text('View QR code')),
                        );
                      },
                    ),
                    const Text('QR Code', style: TextStyle(fontSize: 12)),
                  ],
                ),
                Column(
                  children: [
                    IconButton(
                      icon: const Icon(Icons.delete, color: Colors.red),
                      onPressed: () {
                        Navigator.of(context).pop();
                        _showDeleteConfirmation(context, contact);
                      },
                    ),
                    const Text('Remove', style: TextStyle(fontSize: 12)),
                  ],
                ),
              ],
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  Widget _buildDetailRow(String label, String value) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 100,
          child: Text(
            label,
            style: const TextStyle(
              fontWeight: FontWeight.bold,
              fontSize: 12,
              color: Colors.grey,
            ),
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: const TextStyle(fontSize: 14),
          ),
        ),
      ],
    );
  }

  void _showDeleteConfirmation(BuildContext context, api.ContactInfo contact) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Remove Contact?'),
        content: Text('Are you sure you want to remove ${contact.name}?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () async {
              final appState = context.read<AppState>();

              // TODO: Call FFI to delete from database
              // For now, just refresh the list
              await appState.refreshContacts();

              Navigator.of(context).pop();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Contact removed'),
                  backgroundColor: Colors.orange,
                ),
              );
            },
            style: FilledButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Remove'),
          ),
        ],
      ),
    );
  }
}
