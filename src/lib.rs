mod api;
mod error;
mod file;
use std::{collections::HashMap, sync::Once};

use api::*;
use error::ErrorKind;
#[cfg(feature = "runtime")]
use tokio::runtime::{Builder, Runtime};

const HOST: &str = "127.0.0.1";
const PORT: u16 = 5001;

#[cfg(feature = "runtime")]
pub fn get_runtime() -> Option<&'static Runtime> {
    static mut RUNTIME: Option<Runtime> = None;
    static RUNTIME_ONCE: Once = Once::new();
    RUNTIME_ONCE.call_once(|| {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        unsafe {
            RUNTIME = Some(rt);
        };
    });
    unsafe { RUNTIME.as_ref() }
}

pub fn get_ctx() -> Option<&'static mut HashMap<u32, Respone>> {
    static mut CTX: Option<HashMap<u32, Respone>> = None;
    static CTX_ONCE: Once = Once::new();
    CTX_ONCE.call_once(|| {
        unsafe {
            CTX = Some(HashMap::new());
        };
    });
    unsafe { CTX.as_mut() }
}

pub fn increase_fd() -> Option<u32> {
    static mut MAX_HANDLE: u32 = 0;
    unsafe { 
        MAX_HANDLE += 1;
        Some(MAX_HANDLE)
    }
}

pub async fn command(cmd: &str) -> Result<(u16, u32), ErrorKind> {
    let rs = inner_command(cmd).await?;
    let fd = increase_fd().unwrap();
    let status = rs.status;
    get_ctx().unwrap().insert(fd, rs);
    Ok((status, fd))
}

pub async fn read_body(handle: u32, buf: &mut [u8]) -> Result<u32, ErrorKind> {
    let ctx = get_ctx().unwrap();
    if buf.len() == 0 {
        return Err(ErrorKind::ParameterError);
    }
    match ctx.get_mut(&handle) {
        Some(resp) => {
            Ok(resp.copy_body_remain(buf) as _)
        }
        None => return Err(ErrorKind::HandleError),
    }
}

async fn inner_command(cmd: &str) -> Result<Respone, ErrorKind> {
    let json = match json::parse(cmd) {
        Ok(o) => o,
        Err(_) => return Err(ErrorKind::JsonError),
    };
    let api = match json["api"].as_str() {
        Some(s) => String::from(s),
        None => return Err(ErrorKind::ParameterError),
    };
    match api.as_str() {
        "file/ls" => Api::new(HOST, PORT).file_api().ls().await,
        _ => return Err(ErrorKind::Unsupport),
    }
}
