#[cfg(all(test, target_os = "windows"))]
mod win32_subprocess_test {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    use windows::{
        Win32::{
            Foundation::*,
            System::{IO::*, Threading::*},
        },
        core::*,
    };

    fn to_wstring(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(once(0)).collect()
    }

    #[test]
    fn test_long_path_createprocess_iocp() {
        if !unsafe { crate::fs::is_long_path_aware_runtime() } {
            // println!("WARN: not Long path aware, this may fail");
            panic!("must have LPE to run this test");
        }
        let ch = unsafe {
            crate::fs::rs_chdir_ntdll_longpath(
                ".\\mithril_test_zone\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\ABC1234567890ABC123\\",
            )
        };
        assert!(ch != 0);
        unsafe {
            // --- 1. Build a LONG path ---
            // let long_dir = "C:\\temp\\".to_string()
            //     + &"a".repeat(240)  // adjust to exceed MAX_PATH
            //     + "\\tool.exe";

            // let long_path = format!("\\\\?\\{}", long_dir);
            let long_path = "..\\..\\tool.exe";

            let mut cmd = to_wstring(&long_path);

            // --- 2. Setup process structs ---
            let mut si = STARTUPINFOW::default();
            si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

            let mut pi = PROCESS_INFORMATION::default();

            // --- 3. Try to spawn ---
            let ok = CreateProcessW(
                PCWSTR::null(),
                Some(PWSTR(cmd.as_mut_ptr())),
                None,
                None,
                false,
                PROCESS_CREATION_FLAGS(0),
                None,
                PCWSTR::null(),
                &si,
                &mut pi,
            );

            // We EXPECT failure if path doesn't exist,
            // but NOT "filename too long"
            if let Err(err) = ok {
                // let err = GetLastError();
                println!("CreateProcessW failed: {:?}", err);

                // This is the key assertion:
                // it should NOT be ERROR_FILENAME_EXCED_RANGE (206)
                assert_ne!(err.code().0, ERROR_FILENAME_EXCED_RANGE.to_hresult().0);
                return;
            }

            // --- 4. Minimal IOCP setup ---
            let iocp = CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 1).unwrap();

            assert!(iocp.0 != std::ptr::null_mut());

            // Associate process handle if valid
            if pi.hProcess.0 != std::ptr::null_mut() {
                let _ = CreateIoCompletionPort(pi.hProcess, Some(iocp), 1, 0);
            }

            // --- 5. Wait (non-blocking minimal test) ---
            let mut bytes = 0;
            let mut key = 0;
            let mut overlapped: *mut OVERLAPPED = null_mut();

            GetQueuedCompletionStatus(
                iocp,
                &mut bytes,
                &mut key,
                &mut overlapped,
                10, // short timeout
            )
            .unwrap();

            // Cleanup
            if pi.hProcess.0 != std::ptr::null_mut() {
                CloseHandle(pi.hProcess).unwrap();
            }
            if pi.hThread.0 != std::ptr::null_mut() {
                CloseHandle(pi.hThread).unwrap();
            }
            CloseHandle(iocp).unwrap();
        }
    }
}
