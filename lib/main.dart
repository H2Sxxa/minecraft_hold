import 'dart:async';

import 'package:dynamic_color/dynamic_color.dart';
import 'package:flutter/material.dart';
import 'package:minecraft_hold/native.dart';

void main() {
  runApp(const MainApplication());
}

class MainApplication extends StatelessWidget {
  const MainApplication({super.key});

  @override
  Widget build(BuildContext context) {
    return DynamicColorBuilder(
      builder: (lightDynamic, darkDynamic) {
        return MaterialApp(
          theme: ThemeData(
            colorScheme: lightDynamic,
            useMaterial3: true,
            typography: Typography.material2021(),
          ),
          darkTheme: ThemeData(
            colorScheme: darkDynamic,
            useMaterial3: true,
            typography: Typography.material2021(),
            brightness: Brightness.dark,
          ),
          themeMode: ThemeMode.system,
          home: const HomePage(),
        );
      },
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<StatefulWidget> createState() => HomePageState();
}

class HomePageState extends State<HomePage> {
  List<MinecraftInfo> info = [];
  List<int> targets = [];
  late Timer timer;

  @override
  void initState() {
    super.initState();
    info = findMinecrafts();

    timer = Timer.periodic(const Duration(seconds: 5), (timer) {
      setState(() {
        info = findMinecrafts();
        var active = info.map((e) => e.pid).toList();
        targets = targets.where((element) => active.contains(element)).toList();
      });
    });
  }

  @override
  void dispose() {
    super.dispose();
    timer.cancel();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text("Minecraft Hold"),
        actions: [
          IconButton(
              onPressed: () {
                setState(() {
                  info = findMinecrafts();
                });
              },
              icon: const Icon(Icons.refresh))
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(8),
        child: Column(
          children: [
            Expanded(
              flex: 17,
              child: Card(
                child: SizedBox.expand(
                  child: Visibility(
                    visible: info.isNotEmpty,
                    replacement: const Center(
                      child: Text("暂未找到 Minecraft 进程"),
                    ),
                    child: ListView(
                      children: info
                          .map(
                            (e) => CheckboxListTile(
                              value: targets.contains(e.pid),
                              onChanged: (value) {
                                if (value == true) {
                                  targets.add(e.pid);
                                }

                                if (value == false) {
                                  targets.remove(e.pid);
                                }

                                setState(() {});
                              },
                              title: Text(
                                "${e.name} (${e.pid})",
                              ),
                            ),
                          )
                          .toList(),
                    ),
                  ),
                ),
              ),
            ),
            Expanded(
              flex: 3,
              child: Column(
                children: [
                  Expanded(
                    flex: 1,
                    child: Padding(
                      padding: const EdgeInsets.all(4),
                      child: ElevatedButton.icon(
                        onPressed: () {
                          for (var target in targets) {
                            suspendMinecraft(target);
                          }
                        },
                        icon: const Icon(Icons.save),
                        label: const SizedBox(
                          width: double.infinity,
                          child: Center(
                            child: Text("挂起"),
                          ),
                        ),
                      ),
                    ),
                  ),
                  Expanded(
                    flex: 1,
                    child: Padding(
                      padding: const EdgeInsets.all(4),
                      child: ElevatedButton.icon(
                        onPressed: () {
                          for (var target in targets) {
                            resumeMinecraft(target);
                          }
                        },
                        icon: const Icon(Icons.restore),
                        label: const SizedBox(
                          width: double.infinity,
                          child: Center(
                            child: Text("恢复"),
                          ),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            )
          ],
        ),
      ),
    );
  }
}
