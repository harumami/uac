use std::{
    mem::size_of,
    ptr::null,
    slice::from_raw_parts as slice,
};
use windows::{
    core::{
        wcslen,
        Result,
        PCWSTR,
        PWSTR,
    },
    Win32::{
        Foundation::{
            CloseHandle,
            WIN32_ERROR,
        },
        System::{
            Console::{
                AttachConsole,
                FreeConsole,
                ATTACH_PARENT_PROCESS,
            },
            Environment::GetCommandLineW,
            Threading::{
                CreateProcessW,
                ExitProcess,
                GetExitCodeProcess,
                WaitForSingleObject,
                PROCESS_CREATION_FLAGS,
                PROCESS_INFORMATION,
                STARTUPINFOW,
            },
            WindowsProgramming::INFINITE,
        },
    },
};

fn main() {
    unsafe {
        ExitProcess(match run() {
            Result::Ok(code) => code,
            Result::Err(error) => {
                eprintln!("{error}");
                1
            },
        })
    };
}

fn run() -> Result<u32> {
    unsafe { FreeConsole() }.ok()?;
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.ok()?;
    let args = unsafe { GetCommandLineW() }.as_ptr();
    let args = unsafe { slice(args, wcslen(PCWSTR(args)) + 1) };
    const TAB: u16 = '\t' as u16;
    const SPACE: u16 = ' ' as u16;
    const QUOTE: u16 = '"' as u16;

    let space = match args[0] {
        QUOTE => 1 + args[1..].iter().position(|&c| c == QUOTE).unwrap() + 1,
        _ => args.iter().position(|&c| c == SPACE || c == TAB).unwrap(),
    };

    let mut command = args[space
        + args[space..]
            .iter()
            .position(|&c| c != SPACE && c != TAB)
            .unwrap()..]
        .to_owned();

    let info = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..STARTUPINFOW::default()
    };

    let mut target = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessW(
            PCWSTR::null(),
            PWSTR(command.as_mut_ptr()),
            null(),
            null(),
            false,
            PROCESS_CREATION_FLAGS::default(),
            null(),
            PCWSTR::null(),
            &info,
            &mut target,
        )
    }
    .ok()?;

    let _defer = defer(|| unsafe { CloseHandle(target.hProcess) });
    unsafe { CloseHandle(target.hThread) };
    WIN32_ERROR(unsafe { WaitForSingleObject(target.hProcess, INFINITE) }).ok()?;
    let mut code = 0;
    unsafe { GetExitCodeProcess(target.hProcess, &mut code) }.ok()?;
    Result::Ok(code)
}

struct Defer<T, F: FnOnce() -> T> {
    f: Option<F>,
}

impl<T, F: FnOnce() -> T> Drop for Defer<T, F> {
    fn drop(&mut self) {
        if let Option::Some(f) = self.f.take() {
            f();
        }
    }
}

fn defer<T, F: FnOnce() -> T>(f: F) -> Defer<T, F> {
    Defer { f: Option::Some(f) }
}
