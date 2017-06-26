use std;

use super::Error;

pub use self::platform_impl::*;

#[cfg(target_os = "windows")]
mod platform_impl {
    use super::*;
    use super::super::WinApiError;

    extern crate winapi;
    extern crate kernel32;

    use std::io::Write;

    use self::winapi::winbase;
    use self::winapi::winnt::{HANDLE, WCHAR};
    use self::winapi::wincon::CONSOLE_SCREEN_BUFFER_INFOEX as ScreenBufferInfo;
    use self::winapi::minwindef::{FALSE, DWORD};
    use self::winapi::wincon::COORD;

    fn get_handle() -> Result<HANDLE, Error> {
        let handle = unsafe { kernel32::GetStdHandle(winbase::STD_OUTPUT_HANDLE) };
        if handle.is_null() {
            Err(Error::WinApiError(WinApiError::GetStdHandleError))
        } else {
            Ok(handle)
        }
    }

    fn get_screen_buffer_info() -> Result<ScreenBufferInfo, Error> {
        let mut info: ScreenBufferInfo = unsafe { std::mem::uninitialized() };
        info.cbSize = std::mem::size_of_val(&info) as u32;
        let res =
            unsafe { kernel32::GetConsoleScreenBufferInfoEx(get_handle()?, &mut info as *mut _) };
        match res {
            FALSE => Err(Error::WinApiError(WinApiError::GetConsoleScreenBufferInfoError)),
            _ => Ok(info),
        }
    }

    pub fn set_cursor_pos(x: i32, y: i32) -> Result<(), Error> {
        std::io::stdout().flush()?;
        let coord = COORD {
            X: x as i16,
            Y: y as i16,
        };
        let res = unsafe { kernel32::SetConsoleCursorPosition(get_handle()?, coord) };
        match res {
            FALSE => Err(Error::WinApiError(WinApiError::SetConsoleCursorPositionError)),
            _ => Ok(()),
        }
    }

    pub fn get_cursor_pos() -> Result<(i32, i32), Error> {
        std::io::stdout().flush()?;
        let info = get_screen_buffer_info()?;
        Ok((
            info.dwCursorPosition.X as i32,
            info.dwCursorPosition.Y as i32,
        ))
    }

    pub fn clear() -> Result<(), Error> {
        let info = get_screen_buffer_info()?;
        let size = info.dwSize.X as u32 * info.dwSize.Y as u32;
        let coord = COORD { X: 0, Y: 0 };

        let mut _written: DWORD = 0;

        let res = unsafe {
            kernel32::FillConsoleOutputCharacterW(
                get_handle()?,
                ' ' as WCHAR,
                size,
                coord,
                &mut _written as *mut _,
            )
        };

        if res == FALSE { return Err(Error::WinApiError(WinApiError::FillConsoleOutputCharacterError)); }

        let res = unsafe {
            kernel32::FillConsoleOutputAttribute(
                get_handle()?,
                info.wAttributes,
                size,
                coord,
                &mut _written as *mut _,
            )
        };

        if res == FALSE {
            Err(Error::WinApiError(WinApiError::FillConsoleOutputAttributeError))
        } else {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform_impl {
    use super::*;

    extern crate termios;

    use std::io::{Read, Write};
    use self::termios::{Termios, tcsetattr, TCSAFLUSH, ICANON, ECHO, CREAD};

    const FD_STDIN: ::std::os::unix::io::RawFd = 1;

    pub fn set_cursor_pos(x: i32, y: i32) -> Result<(), Error> {
        std::io::stdout()
            .write_fmt(format_args!("\x1B[{};{}H", y, x))?;
        Ok(())
    }

    pub fn get_cursor_pos() -> Result<(i32, i32), Error> {
        let mut stdout = std::io::stdout();

        // Set noncanonical mode
        let orig = Termios::from_fd(FD_STDIN)?;
        let mut noncan = orig.clone();
        noncan.c_lflag &= !ICANON;
        noncan.c_lflag &= !ECHO;
        noncan.c_lflag &= !CREAD;
        tcsetattr(FD_STDIN, TCSAFLUSH, &noncan)?;

        // Write command
        stdout.write(b"\x1B[6n")?;
        stdout.flush()?;

        // Read back result
        let mut buf = [0u8; 2];
        // Expect `ESC[`
        std::io::stdin().read_exact(&mut buf)?;
        if buf[0] != 0x1B || buf[1] as char != '[' {
            return Err(Error::GetCursorPosParseError);
        }

        // Read rows and cols through a ad-hoc integer parsing function
        let read_num = || -> Result<(i32, char), Error> {
            let mut num = 0;
            let mut c;

            loop {
                let mut buf = [0u8; 1];
                std::io::stdin().read_exact(&mut buf)?;
                c = buf[0] as char;
                if let Some(d) = c.to_digit(10) {
                    num = if num == 0 { 0 } else { num * 10 };
                    num += d as i32;
                } else {
                    break;
                }
            }

            Ok((num, c))
        };

        // Read rows and expect `;`
        let (rows, c) = read_num()?;
        if c != ';' {
            return Err(Error::GetCursorPosParseError);
        }

        // Read cols
        let (cols, c) = read_num()?;

        // Expect `R`
        let res = if c == 'R' { Ok((cols, rows)) } else { Err(Error::GetCursorPosParseError) };

        // Reset terminal
        tcsetattr(FD_STDIN, TCSAFLUSH, &orig)?;
        res
    }

    pub fn clear() -> Result<(), Error> {
        std::io::stdout().write(b"\x1Bc")?;
        Ok(())
    }
}
