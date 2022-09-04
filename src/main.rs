use defer::defer;
use indoc::printdoc;
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
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const NULL: u16 = b'\0' as u16;
    const TAB: u16 = b'\t' as u16;
    const SPACE: u16 = b' ' as u16;
    const QUOTE: u16 = b'"' as u16;

    // Solve problem of being attached to a new console when running as administrator.
    unsafe { FreeConsole() }.ok()?;
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.ok()?;

    let args = unsafe { GetCommandLineW() }.as_ptr();
    let args = unsafe { slice(args, wcslen(PCWSTR(args)) + 1) }; // Include the null character.

    // The index of the first space.
    let space = match args[0] {
        // When starting with a double quote, the first space is after the second double quote.
        QUOTE => 1 + args[1..].iter().position(|&c| c == QUOTE).unwrap() + 1,
        _ => args.iter().position(|&c| c == SPACE || c == TAB).unwrap(),
    };

    // The index of the first character of the command.
    let command = space
        + args[space..]
            .iter()
            .position(|&c| c != SPACE && c != TAB)
            .unwrap();

    // If there are no parameters, display information and usage.
    if args[command] == NULL {
        printdoc!(
            "
            {NAME} {VERSION}
            {DESCRIPTION}.
            Usage: {NAME} [command]
            "
        );

        return Result::Ok(0);
    }

    let mut command = args[command..].to_owned();

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

    defer! {
        unsafe { CloseHandle(target.hProcess) };
    }

    unsafe { CloseHandle(target.hThread) };
    WIN32_ERROR(unsafe { WaitForSingleObject(target.hProcess, INFINITE) }).ok()?;
    let mut code = 0;
    unsafe { GetExitCodeProcess(target.hProcess, &mut code) }.ok()?;
    Result::Ok(code)
}
