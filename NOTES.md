
    flutter pub get

to build library:

    cargo ndk -o ../android/app/src/main/jniLibs build

to gen code:

    flutter_rust_bridge_codegen --rust-input native/src/api.rs --dart-output lib/src/bridge_generated.dart

then fixup bridge_generated and maybe bridge_generated.io
like this:

    void store_dart_post_cobject(
      DartPostCObjectFnType ptr,
    ) {
      return _store_dart_post_cobject(
        ptr,
      );
    }

    late final _store_dart_post_cobjectPtr = _lookup<ffi.NativeFunction<ffi.Void Function(DartPostCObjectFnType)>>('store_dart_post_cobject');
    late final _store_dart_post_cobject = _store_dart_post_cobjectPtr.asFunction<void Function(DartPostCObjectFnType)>();
