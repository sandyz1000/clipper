//
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::mem;
use std::os::raw::c_int;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::ptr;
use std::process::{Command, exit};
use std::os::unix::process::{CommandExt, ExitStatusExt};

use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use std::os::raw::{c_char, c_int, c_uchar, c_ulonglong};
use std::path::Path;

const NATIVE_BYTE_ORDER: u8 = 1;
const EXIT_CODE_FAILURE: i32 = 0xff;


fn copy_and_patch_runtime(
    fd: c_int,
    appimage_filename: &str,
    elf_size: isize,
) -> Result<(), Box<dyn Error>> {
    let realfd = File::open(appimage_filename)?;
    let mut buffer = vec![0u8; elf_size as usize];
    realfd.read_exact(&mut buffer)?;
    unsafe {
        libc::write(
            fd,
            buffer.as_ptr() as *const libc::c_void,
            elf_size as usize,
        );
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn create_memfd_with_patched_runtime(
    appimage_filename: &str,
    elf_size: isize,
) -> Result<c_int, Box<dyn Error>> {
    const MFD_CLOEXEC: c_int = 0x0001;
    const MFD_ALLOW_SEALING: c_int = 0x0002;
    const MFD_HUGETLB: c_int = 0x0004;
    const MFD_HUGE_16GB: c_int = 0x4000;
    const MFD_HUGE_512MB: c_int = 0x2000;

    let memfd = unsafe {
        libc::syscall(
            libc::SYS_memfd_create,
            CString::new("runtime")?.as_ptr(),
            MFD_CLOEXEC,
        )
    };
    if memfd < 0 {
        return Err(format!("memfd_create failed: {}", Error::last_os_error()).into());
    }

    copy_and_patch_runtime(memfd, appimage_filename, elf_size)?;

    Ok(memfd)
}

#[cfg(not(target_os = "linux"))]
fn create_shm_fd_with_patched_runtime(
    appimage_filename: &str,
    elf_size: isize,
) -> Result<c_int, Box<dyn Error>> {
    let runtime_filename = CString::new("runtime-XXXXXX")?;
    let writable_fd = unsafe {
        libc::shm_open(
            runtime_filename.as_ptr(),
            libc::O_RDWR | libc::O_CREAT,
            0o700,
        )
    };
    if writable_fd < 0 {
        return Err(format!("shm_open failed (writable): {}", Error::last_os_error()).into());
    }

    let readable_fd = unsafe { libc::shm_open(runtime_filename.as_ptr(), libc::O_RDONLY, 0) };
    if readable_fd < 0 {
        return Err(format!("shm_open failed (read-only): {}", Error::last_os_error()).into());
    }

    if unsafe { libc::shm_unlink(runtime_filename.as_ptr()) } != 0 {
        return Err(format!("shm_unlink failed: {}", Error::last_os_error()).into());
    }

    copy_and_patch_runtime(writable_fd, appimage_filename, elf_size)?;

    unsafe {
        libc::close(writable_fd);
    }

    Ok(readable_fd)
}


#[derive(Debug)]
pub struct TemporaryPreloadLibFile {
    file: NamedTempFile,
}

impl TemporaryPreloadLibFile {
    pub fn new(lib_contents: &[u8]) -> Result<Self> {
        let mut file = NamedTempFile::new()?;
        file.write_all(lib_contents)?;

        Ok(Self { file })
    }

    pub fn path(&self) -> PathBuf {
        self.file.path().to_path_buf()
    }
}

impl Drop for TemporaryPreloadLibFile {
    fn drop(&mut self) {
        let _ = self.file.close();
    }
}

static mut SUBPROCESS_PID: pid_t = 0;

fn forward_signal(signal: c_int) {
    unsafe {
        if SUBPROCESS_PID != 0 {
            log_debug!("forwarding signal {} to subprocess {}\n", signal, SUBPROCESS_PID);
            libc::kill(SUBPROCESS_PID, signal);
        } else {
            log_error!("signal {} received but no subprocess created yet, shutting down\n", signal);
            exit(signal);
        }
    }
}

fn bypass_binfmt_and_run_appimage(appimage_path: &str, target_args: &[&str]) -> i32 {
    // read size of AppImage runtime (i.e., detect size of ELF binary)
    let size = elf_binary_size(appimage_path);

    if size < 0 {
        log_error!("failed to detect runtime size\n");
        return EXIT_CODE_FAILURE;
    }

    let runtime_fd = if cfg!(feature = "memfd_create") {
        // create "file" in memory, copy runtime there and patch out magic bytes
        create_memfd_with_patched_runtime(appimage_path, size)
    } else {
        create_shm_fd_with_patched_runtime(appimage_path, size)
    };

    if runtime_fd < 0 {
        log_error!("failed to set up in-memory file with patched runtime\n");
        return EXIT_CODE_FAILURE;
    }

    // to keep alive the memfd, we launch the AppImage as a subprocess
    unsafe {
        SUBPROCESS_PID = libc::fork();
    }

    if unsafe { SUBPROCESS_PID } == 0 {
        // create new argv array, using passed filename as argv[0]
        let mut new_argv: Vec<CString> = target_args.iter().map(|arg| CString::new(*arg).unwrap()).collect();
        new_argv.insert(0, CString::new(appimage_path).unwrap());

        // needs to be null terminated, of course
        new_argv.push(CString::new("").unwrap());

        // preload our library
        let preload_lib_path = find_preload_library(is_32bit_elf(appimage_path));

        // may or may not be used, but must survive until this application terminates
        let temporary_preload_lib_file = if access(preload_lib_path.as_ptr(), libc::F_OK) != 0 {
            log_warning!("could not find preload library path, using temporary file\n");

            #[cfg(feature = "preload_lib_path_32bit")]
            if is_32bit_elf(appimage_path) {
                Some(TemporaryPreloadLibFile::new(libbinfmt_bypass_preload_32bit_so, libbinfmt_bypass_preload_32bit_so_len))
            } else {
                None
            }
            .unwrap_or_else(|| TemporaryPreloadLibFile::new(libbinfmt_bypass_preload_so, libbinfmt_bypass_preload_so_len))
        } else {
            None
        };

        log_debug!("library to preload: {}\n", preload_lib_path);

        unsafe {
            libc::setenv(CString::new("LD_PRELOAD").unwrap().as_ptr(), preload_lib_path.as_ptr(), 1);
        }

        // calculate absolute path to AppImage, for use in the preloaded lib
        let abs_appimage_path = CString::new(realpath(appimage_path, ptr::null_mut())).unwrap();
        unsafe {
            libc::setenv(CString::new("REDIRECT_APPIMAGE").unwrap().as_ptr(), abs_appimage_path.as_ptr(), 1);
        }

        log_debug!("fexecve(...)\n");
        libc::fexecve(runtime_fd, new_argv[0].as_ptr(), new_argv.as_ptr());

        log_error!("failed to execute patched runtime: {}\n", Error::last_os_error());
        return EXIT_CODE_FAILURE;
    }

    // now that we have a subprocess and know its process ID, it's time to set up signal forwarding
    // note that from this point on, we don't handle signals ourselves any more, but rely on the subprocess to exit
    // properly
    for i in 0..32 {
        unsafe {
            libc::signal(i, forward_signal);
        }
    }

    // wait for child process to exit, and exit with its return code
    let mut status: i32 = 0;
    unsafe {
        libc::wait(&mut status);
    }

    // clean up
    libc::close(runtime_fd);

    // calculate return code based on child's behavior
    let child_retcode = if unsafe { libc::WIFSIGNALED(status) } != 0 {
        unsafe { libc::WTERMSIG(status) }
    } else if unsafe { libc::WIFEXITED(status) } != 0 {
        unsafe { libc::WEXITSTATUS(status) }
    } else {
        log_error!("unknown error: child didn't exit with signal or regular exit code\n");
        EXIT_CODE_FAILURE
    };

    child_retcode
}

trait ByteSwap {
    fn bswap(self) -> Self;
}

impl ByteSwap for u16 {
    fn bswap(self) -> Self {
        self.swap_bytes()
    }
}

impl ByteSwap for u32 {
    fn bswap(self) -> Self {
        self.swap_bytes()
    }
}

impl ByteSwap for u64 {
    fn bswap(self) -> Self {
        self.swap_bytes()
    }
}

fn swap_data_if_necessary<EhdrT, ValT>(ehdr: &EhdrT, val: &mut ValT)
where
    EhdrT: ElfHeader,
    ValT: ByteSwap,
{
    if ehdr.e_ident[EI_DATA] != NATIVE_BYTE_ORDER {
        *val = val.bswap();
    }
}

trait ElfHeader {
    fn e_ident(&self) -> &[c_uchar; EI_NIDENT];
    fn e_shoff(&self) -> usize;
    fn e_shentsize(&self) -> usize;
    fn e_shnum(&self) -> usize;
}

struct Elf32_Ehdr {
    e_ident: [u8; EI_NIDENT],
    e_shoff: u32,
    e_shentsize: u32,
    e_shnum: u32,
    // ...
}

impl ElfHeader for Elf32_Ehdr {
    fn e_ident(&self) -> &[c_uchar; EI_NIDENT] {
        &self.e_ident
    }

    fn e_shoff(&self) -> usize {
        self.e_shoff as usize
    }

    fn e_shentsize(&self) -> usize {
        self.e_shentsize as usize
    }

    fn e_shnum(&self) -> usize {
        self.e_shnum as usize
    }
}

pub struct Elf64_Ehdr {
    e_ident: [u8; EI_NIDENT],
    e_shoff: u64,
    e_shentsize: u64,
    e_shnum: u64,
    // ...
}

impl ElfHeader for Elf64_Ehdr {
    fn e_ident(&self) -> &[c_uchar; EI_NIDENT] {
        &self.e_ident
    }

    fn e_shoff(&self) -> usize {
        self.e_shoff as usize
    }

    fn e_shentsize(&self) -> usize {
        self.e_shentsize as usize
    }

    fn e_shnum(&self) -> usize {
        self.e_shnum as usize
    }
}

trait ElfSectionHeader {
    fn sh_offset(&self) -> usize;
    fn sh_size(&self) -> usize;
}

struct Elf32_Shdr {
    sh_offset: u32,
    sh_size: u32,
    // ...
}

impl ElfSectionHeader for Elf32_Shdr {
    fn sh_offset(&self) -> usize {
        self.sh_offset as usize
    }

    fn sh_size(&self) -> usize {
        self.sh_size as usize
    }
}

pub struct Elf64_Shdr {
    sh_offset: u64,
    sh_size: u64,
    // ...
}