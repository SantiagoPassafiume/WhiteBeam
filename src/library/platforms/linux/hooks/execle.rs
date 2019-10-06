use libc::{c_char, c_int};
use crate::library::platforms::linux;
use crate::library::common::whitelist;
use crate::library::common::event;

/*
       int execle(const char *path, const char *arg, ...
                       /*, (char *) NULL, char * const envp[] */);
*/
hook! {
    unsafe fn hooked_execle(path: *const c_char, arg: *const c_char, null: *const c_char, envp: *const *const c_char) -> c_int => execle {
		let (program, env) = linux::transform_parameters(path, envp, -1);
		let (hexdigest, uid) = linux::get_hash_and_uid(&program);
        // Permit/deny execution
        if whitelist::is_whitelisted(&program, &env, &hexdigest) {
            event::send_exec_event(uid, &program, &hexdigest, true);
            real!(hooked_execle)(path, arg, null, envp)
        } else {
            event::send_exec_event(uid, &program, &hexdigest, false);
            *linux::errno_location() = libc::EACCES;
            return -1
        }
    }
}
