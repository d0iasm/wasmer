macro_rules! assert_wasi_output {
    ($file:expr, $name:expr, $po_dir_args: expr, $mapdir_args:expr, $envvar_args:expr, $expected:expr) => {{
        use wasmer_dev_utils::stdio::StdioCapturer;
        use wasmer_runtime_core::{backend::Compiler, Func};
        use wasmer_wasi::{generate_import_object_for_version, get_wasi_version};

        #[cfg(feature = "clif")]
        fn get_compiler() -> impl Compiler {
            use wasmer_clif_backend::CraneliftCompiler;
            CraneliftCompiler::new()
        }

        #[cfg(feature = "llvm")]
        fn get_compiler() -> impl Compiler {
            use wasmer_llvm_backend::LLVMCompiler;
            LLVMCompiler::new()
        }

        #[cfg(feature = "singlepass")]
        fn get_compiler() -> impl Compiler {
            use wasmer_singlepass_backend::SinglePassCompiler;
            SinglePassCompiler::new()
        }

        #[cfg(not(any(feature = "llvm", feature = "clif", feature = "singlepass")))]
        fn get_compiler() -> impl Compiler {
            compile_error!("compiler not specified, activate a compiler via features");
            unreachable!();
        }

        let wasm_bytes = include_bytes!($file);

        let module = wasmer_runtime_core::compile_with(&wasm_bytes[..], &get_compiler())
            .expect("WASM can't be compiled");

        let wasi_version = get_wasi_version(&module).expect("WASI module");

        let import_object = generate_import_object_for_version(
            wasi_version,
            vec![],
            vec![],
            $po_dir_args,
            $mapdir_args,
        );

        let instance = module
            .instantiate(&import_object)
            .map_err(|err| format!("Can't instantiate the WebAssembly module: {:?}", err))
            .unwrap(); // NOTE: Need to figure what the unwrap is for ??

        let capturer = StdioCapturer::new();

        let start: Func<(), ()> = instance
            .func("_start")
            .map_err(|e| format!("{:?}", e))
            .expect("start function in wasi module");

        start.call().expect("execute the wasm");

        let output = capturer.end().unwrap().0;
        let expected_output = include_str!($expected);

        assert!(
            output.contains(expected_output),
            "Output: `{}` does not contain expected output: `{}`",
            output,
            expected_output
        );
    }};
}
