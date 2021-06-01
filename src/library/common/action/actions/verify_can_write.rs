#[macro_use]
build_action! { VerifyCanWrite (src_prog, hook, arg_id, args, do_return, return_value) {
        let directory_index = args.iter().position(|arg| arg.id == arg_id).expect("WhiteBeam: Lost track of environment");
        let directory_argument: crate::common::db::ArgumentRow = args[directory_index].clone();
        let library: &str = &hook.library;
        let symbol: &str = &hook.symbol;
        if !(crate::common::db::get_prevention()) {
            return (hook, args, do_return, return_value);
        }
        // Permit authorized writes
        if crate::common::db::get_valid_auth_env() {
            return (hook, args, do_return, return_value);
        }
        let is_read_only: bool = match (library, symbol) {
            ("/lib/x86_64-linux-gnu/libc.so.6", "fdopen") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fopen") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fopen64") => {
                let mode = args[1].real as *const libc::c_char;
                let mode_string = String::from(unsafe { std::ffi::CStr::from_ptr(mode) }.to_str().expect("WhiteBeam: Unexpected null reference"));
                if !(mode_string.contains("w") ||
                     mode_string.contains("a") ||
                     mode_string.contains("+")) {
                    true
                } else {
                    false
                }
            },
            ("/lib/x86_64-linux-gnu/libc.so.6", "open") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "open64") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "openat") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "openat64") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "__open") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "__open_2") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "__open64") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "__open64_2") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "__openat_2") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "__openat64_2") => {
                let flags = args[2].real as libc::c_int;
                if !((flags & (libc::O_RDWR | libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_TMPFILE | libc::O_APPEND | libc::O_TRUNC)) > 0) {
                    true
                } else {
                    false
                }
            },
            _ => false
        };
        // Permit read-only
        if is_read_only {
            return (hook, args, do_return, return_value);
        }
        let any = String::from("ANY");
        let class = String::from("Filesystem/Directory/Writable");
        let all_allowed_directories: Vec<String> = {
            let whitelist_cache_lock = crate::common::db::WL_CACHE.lock().expect("WhiteBeam: Failed to lock mutex");
            whitelist_cache_lock.iter().filter(|whitelist| (whitelist.class == class) && ((whitelist.path == src_prog) || (whitelist.path == any))).map(|whitelist| whitelist.value.clone()).collect()
        };
        // Permit ANY
        if all_allowed_directories.iter().any(|directory| directory == &any) {
            return (hook, args, do_return, return_value);
        }
        // NB: Do not dereference paths here
        let canonical_path = platform::canonicalize_fd(directory_argument.real as i32).expect("WhiteBeam: Lost track of environment");
        // Minor performance hit by defining here instead of match statement
        let parent: std::path::PathBuf = match (&canonical_path).parent() {
            Some(f) => f.to_owned(),
            None => std::path::PathBuf::from("/")
        };
        let mut filename: String = String::from(".");
        let mut target_directory: String = match (library, symbol) {
            ("/lib/x86_64-linux-gnu/libc.so.6", "fopen") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fopen64") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "truncate") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "truncate64") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fchmod") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fchown") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fdopen") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "ftruncate") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "ftruncate64") => {
                // This function passes file descriptors
                filename = String::from((&canonical_path).file_name().unwrap_or(&std::ffi::OsStr::new(".")).to_str().expect("WhiteBeam: Unexpected null reference"));
                parent.into_os_string().into_string().expect("WhiteBeam: Unexpected null reference")
            },
            ("/lib/x86_64-linux-gnu/libc.so.6", "fchownat") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "linkat") => {
                let flags = args.last().expect("WhiteBeam: Lost track of environment");
                if (flags.real as i32 & libc::AT_EMPTY_PATH) > 0 {
                    filename = String::from((&canonical_path).file_name().unwrap_or(&std::ffi::OsStr::new(".")).to_str().expect("WhiteBeam: Unexpected null reference"));
                    parent.into_os_string().into_string().expect("WhiteBeam: Unexpected null reference")
                } else {
                    filename = unsafe { String::from(std::ffi::CStr::from_ptr(args[directory_index+1].real as *const libc::c_char).to_str().expect("WhiteBeam: Unexpected null reference")) };
                    canonical_path.into_os_string().into_string().expect("WhiteBeam: Unexpected null reference")
                }
            },
            _ => {
                // This function passes directory file descriptors
                filename = unsafe { String::from(std::ffi::CStr::from_ptr(args[directory_index+1].real as *const libc::c_char).to_str().expect("WhiteBeam: Unexpected null reference")) };
                canonical_path.into_os_string().into_string().expect("WhiteBeam: Unexpected null reference")
            }
        };
        target_directory.push('/');
        let full_path = format!("{}{}", target_directory, filename);
        // Special cases. We don't want to whitelist /dev (although pts and related subdirectories are fine).
        if (full_path == "/dev/tty") || (full_path == "/dev/null") {
            return (hook, args, do_return, return_value);
        }
        // Permit whitelisted directories
        if all_allowed_directories.iter().any(|directory| glob::Pattern::new(directory).expect("WhiteBeam: Invalid glob pattern").matches(&target_directory)) {
            return (hook, args, do_return, return_value);
        }
        // Deny by default
        event::send_log_event(event::LogClass::Warn as i64, format!("Blocked {} from writing to {} (VerifyCanWrite)", &src_prog, &target_directory));
        eprintln!("WhiteBeam: {}: Permission denied", &full_path);
        do_return = true;
        match (library, symbol) {
            ("/lib/x86_64-linux-gnu/libc.so.6", "fopen") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fopen64") |
            ("/lib/x86_64-linux-gnu/libc.so.6", "fdopen") => {
                return_value = 0;
            }
            _ => {
                return_value = -1;
            }
        };
}}
