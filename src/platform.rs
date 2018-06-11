use std;

use super::Error;

pub use self::platform_impl::*;

#[cfg(target_os = "windows")]
mod platform_impl {
    extern crate winapi;
    use super::*;

    use std::io::Write;

    use self::winapi::{shared::minwindef::{DWORD, FALSE},
                       um::{processenv, winbase, wincon,
                            wincon::{CONSOLE_SCREEN_BUFFER_INFOEX as ScreenBufferInfo, COORD},
                            winnt::{HANDLE, WCHAR}}};

    fn get_handle() -> Result<HANDLE, Error> {
        match unsafe { processenv::GetStdHandle(winbase::STD_OUTPUT_HANDLE) } {
            hdl if hdl.is_null() => Err(Error::PlatformSpecific),
            hdl => Ok(hdl),
        }
    }

    fn get_screen_buffer_info() -> Result<ScreenBufferInfo, Error> {
        unsafe {
            let mut info: ScreenBufferInfo = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<ScreenBufferInfo>() as u32;
            match wincon::GetConsoleScreenBufferInfoEx(get_handle()?, &mut info as *mut _) {
                FALSE => Err(Error::PlatformSpecific),
                _ => Ok(info),
            }
        }
    }

    pub fn set_cursor_pos(x: i32, y: i32) -> Result<(), Error> {
        std::io::stdout().flush()?;
        let coord = COORD {
            X: x as i16,
            Y: y as i16,
        };
        match unsafe { wincon::SetConsoleCursorPosition(get_handle()?, coord) } {
            FALSE => Err(Error::PlatformSpecific),
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
            wincon::FillConsoleOutputCharacterW(
                get_handle()?,
                ' ' as WCHAR,
                size,
                coord,
                &mut _written as *mut _,
            )
        };

        if res == FALSE {
            return Err(Error::PlatformSpecific);
        }

        let res = unsafe {
            wincon::FillConsoleOutputAttribute(
                get_handle()?,
                info.wAttributes,
                size,
                coord,
                &mut _written as *mut _,
            )
        };

        if res == FALSE {
            Err(Error::PlatformSpecific)
        } else {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform_impl {
    use super::*;

    extern crate termios;

    use self::termios::{tcsetattr, Termios, CREAD, ECHO, ICANON, TCSAFLUSH};
    use std::io::{Read, Write};

    const FD_STDIN: ::std::os::unix::io::RawFd = 1;

    pub fn set_cursor_pos(x: i32, y: i32) -> Result<(), Error> {
        std::io::stdout().write_fmt(format_args!("\x1B[{};{}H", y, x))?;
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

        let mut buf = [0u8; 2];

        // Expect `ESC[`
        std::io::stdin().read_exact(&mut buf)?;
        if buf[0] != 0x1B || buf[1] as char != '[' {
            return Err(Error::PlatformSpecific);
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
            return Err(Error::PlatformSpecific);
        }

        // Read cols
        let (cols, c) = read_num()?;

        // Expect `R`
        let res = if c == 'R' {
            Ok((cols, rows))
        } else {
            Err(Error::PlatformSpecific)
        };

        // Reset terminal
        tcsetattr(FD_STDIN, TCSAFLUSH, &orig)?;
        res
    }

    pub fn clear() -> Result<(), Error> {
        std::io::stdout().write(b"\x1Bc")?;
        Ok(())
    }
}
