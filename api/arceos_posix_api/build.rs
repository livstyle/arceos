fn main() {
    use std::io::Write;

    fn gen_pthread_mutex(out_file: &str) -> std::io::Result<()> {
        // TODO: generate size and initial content automatically.
        let (mutex_size, mutex_init) = if cfg!(feature = "multitask") {
            if cfg!(feature = "smp") {
                (6, "{0, 0, 8, 0, 0, 0}") // core::mem::transmute::<_, [usize; 6]>(axsync::Mutex::new(()))
            } else {
                (5, "{0, 8, 0, 0, 0}") // core::mem::transmute::<_, [usize; 5]>(axsync::Mutex::new(()))
            }
        } else {
            (1, "{0}")
        };

        let mut output = Vec::new();
        writeln!(
            output,
            "// Generated by arceos_posix_api/build.rs, DO NOT edit!"
        )?;
        writeln!(
            output,
            r#"
typedef struct {{
    long __l[{mutex_size}];
}} pthread_mutex_t;

#define PTHREAD_MUTEX_INITIALIZER {{ .__l = {mutex_init}}}
"#
        )?;
        std::fs::write(out_file, output)?;
        Ok(())
    }

    fn gen_c_to_rust_bindings(in_file: &str, out_file: &str) {
        println!("cargo:rerun-if-changed={in_file}");

        let allow_types = [
            "stat",
            "size_t",
            "ssize_t",
            "off_t",
            "mode_t",
            "sock.*",
            "fd_set",
            "timeval",
            "pthread_t",
            "pthread_attr_t",
            "pthread_mutex_t",
            "pthread_mutexattr_t",
            "epoll_event",
            "iovec",
            "clockid_t",
            "rlimit",
            "aibuf",
        ];
        let allow_vars = [
            "O_.*",
            "AF_.*",
            "SOCK_.*",
            "IPPROTO_.*",
            "FD_.*",
            "F_.*",
            "_SC_.*",
            "EPOLL_CTL_.*",
            "EPOLL.*",
            "RLIMIT_.*",
            "EAI_.*",
            "MAXADDRS",
        ];

        #[derive(Debug)]
        struct MyCallbacks;

        impl bindgen::callbacks::ParseCallbacks for MyCallbacks {
            fn include_file(&self, fname: &str) {
                if !fname.contains("ax_pthread_mutex.h") {
                    println!("cargo:rerun-if-changed={}", fname);
                }
            }
        }

        let mut builder = bindgen::Builder::default()
            .header(in_file)
            .clang_arg("-I./../../ulib/axlibc/include")
            .parse_callbacks(Box::new(MyCallbacks))
            .derive_default(true)
            .size_t_is_usize(false)
            .use_core();
        for ty in allow_types {
            builder = builder.allowlist_type(ty);
        }
        for var in allow_vars {
            builder = builder.allowlist_var(var);
        }

        builder
            .generate()
            .expect("Unable to generate c->rust bindings")
            .write_to_file(out_file)
            .expect("Couldn't write bindings!");
    }

    gen_pthread_mutex("../../ulib/axlibc/include/ax_pthread_mutex.h").unwrap();
    gen_c_to_rust_bindings("ctypes.h", "src/ctypes_gen.rs");
}
