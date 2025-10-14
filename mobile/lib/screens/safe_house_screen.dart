import 'package:flutter/material.dart';

class SafeHouseScreen extends StatelessWidget {
  const SafeHouseScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.house, size: 80, color: Colors.grey),
              const SizedBox(height: 16),
              const Text(
                'No Safe Houses Available',
                style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 8),
              const Text(
                'Register yours to help others',
                style: TextStyle(color: Colors.grey),
              ),
              const SizedBox(height: 24),
              FilledButton.icon(
                icon: const Icon(Icons.add_home),
                label: const Text('Register Safe House'),
                onPressed: () {
                  _showRegisterDialog(context);
                },
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showRegisterDialog(BuildContext context) {
    final nameController = TextEditingController();
    final regionController = TextEditingController();
    final capacityController = TextEditingController(text: '2');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Register Safe House'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: nameController,
                decoration: const InputDecoration(
                  labelText: 'Code Name',
                  hintText: 'e.g., Green House',
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: regionController,
                decoration: const InputDecoration(
                  labelText: 'Region',
                  hintText: 'e.g., Northeast Area',
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: capacityController,
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'Capacity',
                  hintText: 'How many people?',
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              // TODO: Save to database via FFI
              Navigator.of(context).pop();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Safe house registered!'),
                  backgroundColor: Colors.green,
                ),
              );
            },
            child: const Text('Register'),
          ),
        ],
      ),
    );
  }
}
