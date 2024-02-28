import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter/material.dart';

final DynamicLibrary native = DynamicLibrary.open("minecraft_hold_api.dll");

final _nativeSuspendMinecraft =
    native.lookupFunction<Void Function(Uint32), void Function(int)>(
        "suspend_minecraft");

void suspendMinecraft(int pid) => _nativeSuspendMinecraft(pid);

final _nativeResumeMinecraft =
    native.lookupFunction<Void Function(Uint32), void Function(int)>(
        "resume_minecraft");

void resumeMinecraft(int pid) => _nativeResumeMinecraft(pid);

final _nativeFindMinecraftes =
    native.lookupFunction<Pointer<Utf8> Function(), Pointer<Utf8> Function()>(
        "find_minecrafts_native");

@immutable
class MinecraftInfo {
  final int pid;
  final String name;
  const MinecraftInfo(this.pid, this.name);

  static MinecraftInfo fromMap(Map<String, dynamic> data) {
    return MinecraftInfo(data["pid"], data["name"]);
  }
}

List<MinecraftInfo> findMinecrafts() {
  var data = _nativeFindMinecraftes().toDartString();
  return (jsonDecode(data) as List)
      .map((e) => MinecraftInfo.fromMap(e))
      .toList();
}
