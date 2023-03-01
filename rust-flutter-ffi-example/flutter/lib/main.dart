import 'dart:ffi' as ffi;

import 'package:flutter/foundation.dart' show compute;
import 'package:flutter/material.dart';

typedef HelloIntNativeType = ffi.Int32 Function(ffi.Int32, ffi.Int32);
typedef HelloIntType = int Function(int, int);

final dynlib = ffi.DynamicLibrary.open(
    "../rust/target/release/rust_flutter_ffi_example.dll");
final HelloIntType helloInt =
    dynlib.lookupFunction<HelloIntNativeType, HelloIntType>("hello_int");

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'rust-flutter-ffi-example',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: const MyHomePage(),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  String result = "<calculating...>";

  @override
  Widget build(BuildContext context) {
    const x = 52;
    const y = 123;

    () async {
      final result = await compute((void _) async {
        await Future.delayed(const Duration(seconds: 5));
        return helloInt(x, y);
      }, null);
      setState(() {
        this.result = result.toString();
      });
    }();

    return Scaffold(
      appBar: AppBar(
        title: const Text("Rust Flutter FFI Example"),
      ),
      body: Center(
        child: Text("$x plus $y is $result"),
      ),
    );
  }
}
