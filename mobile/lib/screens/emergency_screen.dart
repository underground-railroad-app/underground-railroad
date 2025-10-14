import 'package:flutter/material.dart';

class EmergencyScreen extends StatefulWidget {
  const EmergencyScreen({super.key});

  @override
  State<EmergencyScreen> createState() => _EmergencyScreenState();
}

class _EmergencyScreenState extends State<EmergencyScreen> {
  final Set<String> _selectedNeeds = {};
  int _numPeople = 1;
  int _numChildren = 0;

  final List<Map<String, dynamic>> _needs = [
    {'id': 'shelter', 'label': 'Safe place to hide', 'icon': Icons.house},
    {'id': 'transport', 'label': 'Transportation', 'icon': Icons.directions_car},
    {'id': 'medical', 'label': 'Medical help', 'icon': Icons.medical_services},
    {'id': 'food', 'label': 'Food and water', 'icon': Icons.restaurant},
    {'id': 'financial', 'label': 'Money/financial help', 'icon': Icons.attach_money},
    {'id': 'danger', 'label': 'In immediate danger', 'icon': Icons.warning},
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Emergency Request'),
        backgroundColor: Colors.red,
        foregroundColor: Colors.white,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16.0),
        children: [
          // Warning
          const Card(
            color: Colors.red,
            child: Padding(
              padding: EdgeInsets.all(16.0),
              child: Row(
                children: [
                  Icon(Icons.emergency, color: Colors.white, size: 32),
                  SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      'This will alert your trusted network',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 16,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),

          // What do you need?
          const Text(
            'What do you need?',
            style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 12),

          // Needs checkboxes
          ..._needs.map((need) => CheckboxListTile(
                title: Text(need['label']),
                secondary: Icon(need['icon']),
                value: _selectedNeeds.contains(need['id']),
                onChanged: (checked) {
                  setState(() {
                    if (checked == true) {
                      _selectedNeeds.add(need['id']);
                    } else {
                      _selectedNeeds.remove(need['id']);
                    }
                  });
                },
              )),

          const SizedBox(height: 24),

          // How many people?
          const Text(
            'How many people?',
            style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 12),

          Row(
            children: [
              Expanded(
                child: _NumberPicker(
                  label: 'Adults',
                  value: _numPeople,
                  onChanged: (val) => setState(() => _numPeople = val),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: _NumberPicker(
                  label: 'Children',
                  value: _numChildren,
                  onChanged: (val) => setState(() => _numChildren = val),
                ),
              ),
            ],
          ),
          const SizedBox(height: 32),

          // Send button
          FilledButton(
            onPressed: _selectedNeeds.isEmpty ? null : _handleSendEmergency,
            style: FilledButton.styleFrom(
              backgroundColor: Colors.red,
              padding: const EdgeInsets.symmetric(vertical: 16),
            ),
            child: const Text(
              'SEND EMERGENCY REQUEST',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
          ),
        ],
      ),
    );
  }

  void _handleSendEmergency() {
    // TODO: Call Rust FFI to create emergency
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Emergency Sent'),
        content: Text(
          'Your request has been created:\n\n'
          'Needs: ${_selectedNeeds.join(', ')}\n'
          'People: $_numPeople adults, $_numChildren children\n\n'
          'Your trusted network will be notified.',
        ),
        actions: [
          TextButton(
            onPressed: () {
              Navigator.of(context).pop();
              Navigator.of(context).pop(); // Back to home
            },
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }
}

class _NumberPicker extends StatelessWidget {
  final String label;
  final int value;
  final ValueChanged<int> onChanged;

  const _NumberPicker({
    required this.label,
    required this.value,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Column(
          children: [
            Text(label, style: const TextStyle(fontSize: 12)),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                IconButton(
                  icon: const Icon(Icons.remove_circle_outline),
                  onPressed: value > 0 ? () => onChanged(value - 1) : null,
                ),
                Text(
                  value.toString(),
                  style: const TextStyle(
                    fontSize: 32,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                IconButton(
                  icon: const Icon(Icons.add_circle_outline),
                  onPressed: () => onChanged(value + 1),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
