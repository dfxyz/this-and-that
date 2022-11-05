use std::ffi::CStr;
use std::mem;
use std::ptr::null_mut;

use winapi::shared::minwindef;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::wininet::{
    InternetQueryOptionA, INTERNET_OPTION_PER_CONNECTION_OPTION, INTERNET_PER_CONN_AUTOCONFIG_URL,
    INTERNET_PER_CONN_FLAGS, INTERNET_PER_CONN_OPTIONA, INTERNET_PER_CONN_OPTION_LISTA,
    INTERNET_PER_CONN_PROXY_BYPASS, INTERNET_PER_CONN_PROXY_SERVER, PROXY_TYPE_AUTO_PROXY_URL,
    PROXY_TYPE_PROXY,
};

pub struct ProxyOption {
    pub pac_enabled: bool,
    pub pac_url: String,
    pub proxy_enabled: bool,
    pub proxy_address: String,
    pub proxy_bypass_list: Vec<String>,
}

pub fn get_proxy_options() -> Result<ProxyOption, u32> {
    // see: https://learn.microsoft.com/en-us/windows/win32/api/wininet/nf-wininet-internetqueryoptiona
    //      https://learn.microsoft.com/en-us/windows/win32/wininet/option-flags
    //      https://learn.microsoft.com/en-us/windows/win32/api/wininet/ns-wininet-internet_per_conn_option_lista
    //      https://learn.microsoft.com/en-us/windows/win32/api/wininet/ns-wininet-internet_per_conn_optiona

    let mut options = vec![
        INTERNET_PER_CONN_OPTIONA {
            dwOption: INTERNET_PER_CONN_FLAGS,
            Value: unsafe { mem::zeroed() },
        },
        INTERNET_PER_CONN_OPTIONA {
            dwOption: INTERNET_PER_CONN_AUTOCONFIG_URL,
            Value: unsafe { mem::zeroed() },
        },
        INTERNET_PER_CONN_OPTIONA {
            dwOption: INTERNET_PER_CONN_PROXY_SERVER,
            Value: unsafe { mem::zeroed() },
        },
        INTERNET_PER_CONN_OPTIONA {
            dwOption: INTERNET_PER_CONN_PROXY_BYPASS,
            Value: unsafe { mem::zeroed() },
        },
    ];
    let mut list = INTERNET_PER_CONN_OPTION_LISTA {
        dwSize: 0,
        pszConnection: null_mut(),
        dwOptionCount: options.len() as u32,
        dwOptionError: 0,
        pOptions: options.as_mut_ptr(),
    };
    let mut len = mem::size_of::<INTERNET_PER_CONN_OPTION_LISTA>() as u32;
    let ok = unsafe {
        InternetQueryOptionA(
            null_mut(),
            INTERNET_OPTION_PER_CONNECTION_OPTION,
            (&mut list) as *mut _ as _,
            (&mut len) as *mut _,
        )
    };
    if ok == minwindef::FALSE {
        return Err(unsafe { GetLastError() });
    }

    let flags = *unsafe { options[0].Value.dwValue() };
    let pac_enabled = flags & PROXY_TYPE_AUTO_PROXY_URL != 0;
    let proxy_enabled = flags & PROXY_TYPE_PROXY != 0;

    let str_ptr = *unsafe { options[1].Value.pszValue() };
    let pac_url = if str_ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(str_ptr) }
            .to_string_lossy()
            .to_string()
    };
    unsafe { libc::free(str_ptr as _) };

    let str_ptr = *unsafe { options[2].Value.pszValue() };
    let proxy_address = if str_ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(str_ptr) }
            .to_string_lossy()
            .to_string()
    };
    unsafe { libc::free(str_ptr as _) };

    let str_ptr = *unsafe { options[3].Value.pszValue() };
    let proxy_bypass_list = if str_ptr.is_null() {
        vec![]
    } else {
        unsafe { CStr::from_ptr(str_ptr) }
            .to_string_lossy()
            .to_string()
            .split(';')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    unsafe { libc::free(str_ptr as _) };

    Ok(ProxyOption {
        pac_enabled,
        pac_url,
        proxy_enabled,
        proxy_address,
        proxy_bypass_list,
    })
}

#[cfg(test)]
#[test]
fn test_get_proxy_option() {
    match get_proxy_options() {
        Ok(options) => {
            println!("pac enabled: {}", options.pac_enabled);
            println!("pac url: '{}'", options.pac_url);
            println!("proxy enabled: {}", options.proxy_enabled);
            println!("proxy address: '{}'", options.proxy_address);
            println!("proxy bypass: {:#?}", options.proxy_bypass_list);
        }
        Err(errcode) => {
            panic!("errcode: {errcode}",);
        }
    }
}
