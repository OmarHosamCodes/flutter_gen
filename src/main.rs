use anyhow::Result;
use clap::Parser;
use colored::*;
use inquire::{Confirm, Text};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Name of the Flutter project
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(Debug)]
struct Feature {
    name: String,
    layers: Vec<String>,
}

impl Feature {
    fn new(name: &str) -> Self {
        Feature {
            name: name.to_string(),
            layers: vec![
                "data".to_string(),
                "presentation".to_string(),
                "domain".to_string(),
                "logic".to_string(),
            ],
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get project name
    let project_name = match cli.name {
        Some(name) => name,
        None => Text::new("What is your project name?")
            .with_default("my_flutter_app")
            .prompt()?,
    };

    //Get package name
    let package_name = Text::new("What is your package name?")
        .with_default("com.example.my_flutter_app")
        .prompt()?;

    // Create Flutter project
    println!("{}", "Creating Flutter project...".green());
    Command::new("flutter")
        .args([
            "create",
            &project_name,
            "--org",
            &package_name,
            "--platforms",
            "android,ios",
            "--no-pub",
        ])
        .status()?;

    // Get features from user input
    let mut features = Vec::new();
    loop {
        let feature_name = Text::new("Enter feature name (or press enter to finish):").prompt()?;

        if feature_name.trim().is_empty() {
            break;
        }

        // Automatically add sub-features for `auth`
        if feature_name.to_lowercase() == "auth" {
            println!("{}", "Adding auth-related features...".green());
            features.push(Feature::new("login"));
            features.push(Feature::new("register"));
            features.push(Feature::new("forgot_password"));
        } else {
            features.push(Feature::new(&feature_name));
        }
        println!("{}", format!("Added feature: {}", feature_name).green());
    }

    // Ask for state management
    let use_riverpod = Confirm::new("Do you want to use Riverpod for state management?")
        .with_default(true)
        .prompt()?;

    let use_supabase = Confirm::new("Do you want to use Supabase?")
        .with_default(false)
        .prompt()?;

    // Create project structure
    create_project_structure(&project_name, &features, use_riverpod, use_supabase)?;

    println!("{}", "Project structure created successfully!".green());
    Ok(())
}

fn create_project_structure(
    project_name: &str,
    features: &[Feature],
    use_riverpod: bool,
    use_supabase: bool,
) -> Result<()> {
    let lib_path = Path::new(project_name).join("lib");

    // Create base directories
    let base_dirs = vec![
        "app",
        "core/constants",
        "core/utilities",
        "core/services",
        "core/widgets",
        "state",
        "theme",
    ];

    for dir in base_dirs {
        fs::create_dir_all(lib_path.join(dir))?;
    }

    // Create features
    for feature in features {
        let feature_path = lib_path.join("features").join(&feature.name);

        for layer in &feature.layers {
            fs::create_dir_all(feature_path.join(layer))?;
            if layer == "presentation" {
                fs::create_dir_all(feature_path.join(layer).join("widgets"))?;
            }
        }

        // Create basic files for each feature
        create_feature_files(&feature_path, &feature.name, use_riverpod)?;
    }

    // Create core files
    create_core_files(&lib_path, use_supabase, project_name)?;

    // Create app files
    create_app_files(
        &lib_path,
        use_riverpod,
        use_supabase,
        features,
        project_name,
    )?;

    // Create .env file if using Supabase
    if use_supabase {
        fs::write(
            Path::new(project_name).join(".env"),
            "SUPABASE_URL=your_supabase_url\nSUPABASE_ANON_KEY=your_supabase_anon_key\n",
        )?;
    }

    // Run flutter pub commands
    run_flutter_commands(project_name, use_supabase, use_riverpod)?;

    Ok(())
}

fn create_feature_files(feature_path: &Path, feature_name: &str, use_riverpod: bool) -> Result<()> {
    // Create basic files
    let files = vec![
        (
            "data",
            format!("{}_repository.dart", feature_name),
            format!(
                "class {}Repository {{\n  // TODO: Implement repository\n}}",
                pascal_case(feature_name)
            ),
        ),
        (
            "domain",
            format!("{}_model.dart", feature_name),
            format!(
                "class {}Model {{\n  // TODO: Implement model\n}}",
                pascal_case(feature_name)
            ),
        ),
        (
            "presentation",
            format!("{}_screen.dart", feature_name),
            generate_screen_template(feature_name),
        ),
    ];

    for (dir, filename, content) in files {
        fs::write(feature_path.join(dir).join(filename), content)?;
    }

    // Create controller if using Riverpod
    if use_riverpod {
        fs::write(
            feature_path
                .join("logic")
                .join(format!("{}_controller.dart", feature_name)),
            generate_controller_template(feature_name),
        )?;
    }

    Ok(())
}

fn create_core_files(lib_path: &Path, use_supabase: bool, project_name: &str) -> Result<()> {
    let mut core_files: Vec<(&str, String)> = vec![
        (
            "constants/app_theme.dart",
            r#"import 'package:flutter/material.dart';
import 'package:shadcn_ui/shadcn_ui.dart';

class AppColors {
  AppColors._init();
  static AppColors instance = AppColors._init();

  final _theme = ShadThemeData(
    brightness: Brightness.light,
    colorScheme: const ShadSlateColorScheme.light(),
  );

  final _themeDark = ShadThemeData(
    brightness: Brightness.dark,
    colorScheme: const ShadSlateColorScheme.dark(),
  );

  ShadThemeData get theme => _theme;
  ShadThemeData get themeDark => _themeDark;
}"#
            .to_string(),
        ),
        (
            "constants/app_colors.dart",
            "class AppColors {\n  // TODO: Define app colors\n}".to_string(),
        ),
        (
            "constants/app_strings.dart",
            "class AppStrings {\n  // TODO: Define app strings\n}".to_string(),
        ),
        (
            "utilities/logging.dart",
            "class Logger {\n  // TODO: Implement logging\n}".to_string(),
        ),
        (
            "utilities/permissions.dart",
            r#"import 'dart:io' show Platform, exit;

import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:permission_handler/permission_handler.dart';

final permissionUtilProvider =
    StateNotifierProvider<PermissionUtil, bool>((ref) => PermissionUtil());

class PermissionUtil extends StateNotifier<bool> {
  PermissionUtil() : super(false);

  Future<List<Permission>> get _requiredPermissions async {
    if (Platform.isAndroid) {
      if (await _getAndroidSdkVersion() >= 33) {
        // Android 13 and above
        return [
          Permission.photos,
          Permission.videos,
          Permission.activityRecognition,
        ];
      } else if (await _getAndroidSdkVersion() >= 29) {
        // Android 10-12
        return [
          Permission.storage,
          Permission.activityRecognition,
        ];
      } else {
        // Below Android 10
        return [
          Permission.storage,
          Permission.activityRecognition,
        ];
      }
    } else if (Platform.isIOS) {
      return [
        Permission.photos,
        Permission.sensors,
      ];
    }
    return [];
  }

  // Helper method to get Android SDK version
  Future<int> _getAndroidSdkVersion() async {
    try {
      if (Platform.isAndroid) {
        final deviceInfo = DeviceInfoPlugin();
        final androidInfo = await deviceInfo.androidInfo;
        return androidInfo.version.sdkInt;
      }
    } catch (e) {
      print('Error getting Android SDK version: $e');
    }
    return 29; // Default to Android 10 for safety
  }

  Future<bool> requestPermissions() async {
    try {
      final permissions = await _requiredPermissions;
      if (permissions.isEmpty) return false;

      final statuses = await permissions.request();
      return statuses.values.every((status) => status.isGranted);
    } catch (e) {
      print('Error requesting permissions: $e');
      return false;
    }
  }

  Future<bool> checkPermissions() async {
    try {
      final permissions = await _requiredPermissions;
      if (permissions.isEmpty) return false;

      final statuses = await Future.wait(
        permissions.map((permission) => permission.status),
      );
      return statuses.every((status) => status.isGranted);
    } catch (e) {
      print('Error checking permissions: $e');
      return false;
    }
  }

  Future<void> openSettings() async {
    try {
      await openAppSettings();
    } catch (e) {
      print('Error opening settings: $e');
    }
  }

  Future<void> checkAndRequestPermissions(
    BuildContext context,
    WidgetRef ref,
  ) async {
    final hasPermissions = await checkPermissions();
    if (!hasPermissions) {
      final granted = await requestPermissions();
      if (!granted) {
        // Show dialog if permissions are not granted
        if (context.mounted) {
          await showPermissionDialog(context, ref);
        }
      } else {
        state = true;
      }
    } else {
      state = true;
    }
  }

  Future<void> showPermissionDialog(BuildContext context, WidgetRef ref) async {
    return showDialog(
      context: context,
      barrierDismissible: false,
      builder: (BuildContext context) {
        return AlertDialog(
          content: const Text(
              'This app needs access to storage and activity recognition to track your steps. '
              'Please grant the required permissions in settings.'),
          actions: <Widget>[
            TextButton(
              child: const Text('Open Settings'),
              onPressed: () async {
                Navigator.of(context).pop();
                await openSettings();
                if (context.mounted) {
                  await checkAndRequestPermissions(context, ref);
                }
              },
            ),
            TextButton(
              child: const Text('Exit App'),
              onPressed: () async {
                try {
                  await SystemChannels.platform
                      .invokeMethod('SystemNavigator.pop');
                } catch (e) {
                  exit(0);
                }
              },
            ),
          ],
        );
      },
    );
  }
}"#
            .to_string(),
        ),
        (
            "widgets/custom_button.dart",
            generate_custom_button_template(),
        ),
    ];

    // Add auth service files if Supabase is enabled
    if use_supabase {
        core_files.push((
            "services/auth_service.dart",
            r#"import 'package:logging/logging.dart';
import 'package:supabase_flutter/supabase_flutter.dart';

class AuthService {
  AuthService(this.supabase);
  final SupabaseClient supabase;
  final _logger = Logger('AuthService');

  // Sign In
  Future<bool> signIn(String email, String password) async {
    try {
      final response = await supabase.auth.signInWithPassword(
        email: email,
        password: password,
      );
      _logger.info('Sign in response: $response');
      return response.user != null;
    } on AuthException catch (e) {
      _logger.severe('Sign in error: ${e.message}');
      return false;
    } catch (e) {
      _logger.severe('Sign in error: $e');
      return false;
    }
  }

  // Sign Up
  Future<bool> signUp(String email, String password) async {
    try {
      final response = await supabase.auth.signUp(
        email: email,
        password: password,
      );
      _logger.info('Sign up response: $response');
      return response.user != null;
    } on AuthException catch (e) {
      _logger.severe('Sign up error: ${e.message}');
      return false;
    } on PostgrestException catch (e) {
      _logger.severe('Database error: ${e.message}');
      return false;
    } catch (e) {
      _logger.severe('Sign up error: $e');
      return false;
    }
  }

  // Forgot password
  Future<bool> resetPassword(String email) async {
    try {
      await supabase.auth.resetPasswordForEmail(email);
      _logger.info('Password reset email sent to: $email');
      return true;
    } catch (e) {
      _logger.severe('Reset password error: $e');
      return false;
    }
  }

  // Sign Out
  Future<bool> signOut() async {
    try {
      await supabase.auth.signOut();
      _logger.info('User signed out successfully');
      return true;
    } catch (e) {
      _logger.severe('Sign out error: $e');
      return false;
    }
  }

  // Get Current User
  User? getCurrentUser() => supabase.auth.currentUser;

  // Check if Logged In
  bool isLoggedIn() => supabase.auth.currentUser != null;
}"#
            .to_string(),
        ));

        core_files.push((
            "services/auth_service_provider.dart",
            format!(
                r#"import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:{}/core/services/auth_service.dart';
import 'package:supabase_flutter/supabase_flutter.dart';

part 'auth_service_provider.g.dart';

@Riverpod(keepAlive: true)
AuthService authService(Ref ref) {{
  final supabase = Supabase.instance.client;
  return AuthService(supabase);
}}"#,
                project_name
            )
            .to_string(),
        ));
    }

    for (path, content) in core_files {
        fs::write(lib_path.join("core").join(path), content)?;
    }

    Ok(())
}

fn create_app_files(
    lib_path: &Path,
    use_riverpod: bool,
    use_supabase: bool,
    features: &[Feature],
    project_name: &str,
) -> Result<()> {
    let app_files: Vec<(&str, String)> = vec![
        ("app/app.dart", generate_app_template(use_riverpod)),
        (
            "app/router.dart",
            generate_router_template(project_name, use_riverpod, features),
        ),
        (
            "theme/app_theme.dart",
            "import 'package:flutter/material.dart';\n\n// TODO: Implement theme".to_string(),
        ),
    ];

    for (path, content) in app_files {
        fs::write(lib_path.join(path), content)?;
    }

    // Create main.dart
    fs::write(
        lib_path.join("main.dart"),
        generate_main_template(use_supabase),
    )?;

    Ok(())
}

fn generate_router_template(
    project_name: &str,
    use_riverpod: bool,
    features: &[Feature],
) -> String {
    if !use_riverpod {
        return "import 'package:go_router/go_router.dart';\n\n// TODO: Implement router"
            .to_string();
    }

    let mut imports = vec![
        "import 'package:go_router/go_router.dart'".to_string(),
        "import 'package:hooks_riverpod/hooks_riverpod.dart'".to_string(),
    ];

    let mut routes = Vec::new();
    let mut auth_routes = Vec::new();
    let mut has_auth = false;

    // Check for auth-related features
    for feature in features {
        match feature.name.as_str() {
            "auth" | "login" | "register" => {
                has_auth = true;
                imports.push(format!(
                    "import 'package:{}/core/services/auth_service_provider.dart'",
                    project_name
                ));

                if feature.name == "login" || feature.name == "auth" {
                    imports.push(format!(
                        "import 'package:{}/features/auth/presentation/login_screen.dart'",
                        project_name
                    ));
                    auth_routes.push(
                        r#"GoRoute(
    path: 'login',
    name: 'login',
    builder: (context, state) => const LoginScreen(),
),"#
                        .to_string(),
                    );
                }
                if feature.name == "register" {
                    imports.push(format!(
                        "import 'package:{}/features/auth/presentation/register_screen.dart'",
                        project_name
                    ));
                    auth_routes.push(
                        r#"GoRoute(
    path: 'register',
    name: 'register',
    builder: (context, state) => const RegisterScreen(),
),"#
                        .to_string(),
                    );
                }
                if feature.name == "auth" {
                    imports.push(format!(
                        "import 'package:{}/features/auth/presentation/forgot_password_screen.dart'",
                        project_name
                    ));
                    auth_routes.push(
                        r#"GoRoute(
    path: 'forgot-password',
    name: 'forgotPassword',
    builder: (context, state) => const ForgotPasswordScreen(),
),"#
                        .to_string(),
                    );
                }
            }
            "home" => {
                imports.push(format!(
                    "import 'package:{}/features/home/presentation/home_screen.dart'",
                    project_name
                ));
                routes.push(
                    r#"GoRoute(
    path: '/',
    name: 'home',
    builder: (context, state) => const HomeScreen(),
),"#
                    .to_string(),
                );
            }
            "profile" => {
                imports.push(format!(
                    "import 'package:{}/features/profile/presentation/profile_screen.dart'",
                    project_name
                ));
                routes.push(
                    r#"GoRoute(
    path: '/profile',
    name: 'profile',
    builder: (context, state) => const ProfileScreen(),
),"#
                    .to_string(),
                );
            }
            "settings" => {
                imports.push(format!(
                    "import 'package:{}/features/settings/presentation/settings_screen.dart'",
                    project_name
                ));
                routes.push(
                    r#"GoRoute(
    path: '/settings',
    name: 'settings',
    builder: (context, state) => const SettingsScreen(),
),"#
                    .to_string(),
                );
            }
            _ => {}
        }
    }

    // Add default home route if not specified
    if !routes.iter().any(|r| r.contains("path: '/'")) {
        routes.insert(
            0,
            r#"GoRoute(
    path: '/',
    name: 'home',
    builder: (context, state) => const HomeScreen(),
),"#
            .to_string(),
        );
        imports.push(format!(
            "import 'package:{}/features/home/presentation/home_screen.dart'",
            project_name
        ));
    }

    let auth_redirect = if has_auth {
        r#"
    redirect: (context, state) {
        final isLoggedIn = authService.isLoggedIn();
        final location = state.matchedLocation;
        
        // List of auth-related paths
        final authPaths = ['/auth', '/auth/login', '/auth/register', '/auth/forgot-password'];
        
        // If user is not logged in and trying to access protected routes
        if (!isLoggedIn && !authPaths.contains(location)) {
            return '/auth/login';
        }
        
        // If user is logged in and trying to access auth routes
        if (isLoggedIn && authPaths.contains(location)) {
            return '/';
        }
        
        return null;
    },"#
    } else {
        ""
    };

    let auth_service = if has_auth {
        "final authService = ref.read(authServiceProvider);"
    } else {
        ""
    };

    format!(
        r#"{}

final goRouterProvider = Provider<GoRouter>((ref) {{
    {}
    return GoRouter(
        initialLocation: '/',{}
        routes: [
            {}
            {}
            {}
        ],
    );
}});"#,
        imports.join(";\n"),
        auth_service,
        auth_redirect,
        routes.join(",\n            "),
        if has_auth {
            format!(
                r#"GoRoute(
                path: '/auth',
                name: 'auth',
                builder: (context, state) => const LoginScreen(),
                routes: [
                    {}
                ],
            ),"#,
                auth_routes.join("\n                    ")
            )
        } else {
            "".to_string()
        },
        if has_auth { "" } else { "," }
    )
}
fn run_flutter_commands(project_name: &str, use_supabase: bool, use_riverpod: bool) -> Result<()> {
    let project_dir = Path::new(project_name);

    // Base dependencies
    let mut cmd = Command::new("flutter");
    cmd.current_dir(project_dir).args([
        "pub",
        "add",
        "connectivity_plus",
        "device_info_plus",
        "flutter_background_service",
        "flutter_dotenv",
        "flutter_launcher_icons",
        "flutter_native_splash",
        "go_router",
        "logging",
        "path",
        "permission_handler",
        "shadcn_ui",
        "share_plus",
        "simple_circular_progress_bar",
        "sqflite",
    ]);

    if use_riverpod {
        cmd.arg("hooks_riverpod");
        cmd.arg("riverpod_annotation");
        cmd.arg("flutter_riverpod");
        cmd.arg("flutter_hooks");
        cmd.arg("hooks_riverpod");
    }

    if use_supabase {
        cmd.arg("supabase_flutter");
    }

    cmd.status()?;

    // Dev dependencies
    Command::new("flutter")
        .current_dir(project_dir)
        .args([
            "pub",
            "add",
            "--dev",
            "build_runner",
            "flutter_lints",
            "riverpod_generator",
            "very_good_analysis",
        ])
        .status()?;

    Ok(())
}

fn pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;

    for c in s.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn generate_screen_template(feature_name: &str) -> String {
    format!(
        r#"import 'package:flutter/material.dart';

class {}Screen extends StatelessWidget {{
  const {}Screen({{super.key}});

  @override
  Widget build(BuildContext context) {{
    return Scaffold(
      appBar: AppBar(
        title: const Text('{}'),
      ),
      body: const Center(
        child: Text('{}Screen'),
      ),
    );
  }}
}}"#,
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
    )
}

fn generate_controller_template(feature_name: &str) -> String {
    format!(
        r#"import 'package:flutter_riverpod/flutter_riverpod.dart';

final {}Controller = StateNotifierProvider<{}Notifier, {}State>((ref) {{
  return {}Notifier();
}});

class {}State {{
  // TODO: Implement state
}}

class {}Notifier extends StateNotifier<{}State> {{
  {}Notifier() : super({}State());

  // TODO: Implement methods
}}"#,
        feature_name,
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
        pascal_case(feature_name),
    )
}

fn generate_custom_button_template() -> String {
    r#"import 'package:flutter/material.dart';

class CustomButton extends StatelessWidget {
  final String text;
  final VoidCallback onPressed;
  final bool isLoading;

  const CustomButton({
    super.key,
    required this.text,
    required this.onPressed,
    this.isLoading = false,
  });

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      onPressed: isLoading ? null : onPressed,
      child: isLoading
          ? const CircularProgressIndicator()
          : Text(text),
    );
  }
}"#
    .to_string()
}

fn generate_app_template(_use_riverpod: bool) -> String {
    format!(
        r#"import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:shadcn_ui/shadcn_ui.dart';
import '../core/constants/app_colors.dart';

final themeModeProvider = StateProvider<ThemeMode>((ref) => ThemeMode.dark);

class App extends ConsumerWidget {{
  const App({{super.key}});

  @override
  Widget build(BuildContext context, WidgetRef ref) {{
    final themeMode = ref.watch(themeModeProvider);
    final goRouter = ref.watch(goRouterProvider);

    return ShadApp.router(
      debugShowCheckedModeBanner: false,
      darkTheme: AppColors.instance.themeDark,
      theme: AppColors.instance.theme,
      themeMode: themeMode,
      routerConfig: goRouter,
    );
  }}
}}"#
    )
}

fn generate_main_template(use_supabase: bool) -> String {
    let supabase_imports = if use_supabase {
        "import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:supabase_flutter/supabase_flutter.dart';"
    } else {
        ""
    };

    let supabase_init = if use_supabase {
        r#"  // Ensure Flutter binding is initialized
  WidgetsFlutterBinding.ensureInitialized();
  // Load .env file
  await dotenv.load();
  // Supabase init
  await Supabase.initialize(
    url: dotenv.env['SUPABASE_URL'] ?? '',
    anonKey: dotenv.env['SUPABASE_ANON_KEY'] ?? '',
  );"#
    } else {
        ""
    };

    format!(
        r#"import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'app/app.dart';
{}

void main() async {{
{}
  runApp(
    const ProviderScope(
      child: App(),
    ),
  );
}}"#,
        supabase_imports, supabase_init,
    )
}
