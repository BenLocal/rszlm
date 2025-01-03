use std::sync::Mutex;

use once_cell::sync::Lazy;
use rszlm_sys::*;

use crate::{const_ptr_to_string, const_str_to_ptr};

pub struct EnvInitBuilder(mk_config);

impl EnvInitBuilder {
    pub fn new() -> Self {
        Self(mk_config {
            thread_num: 0,
            log_level: 0,
            log_mask: LOG_CONSOLE as i32,
            log_file_path: std::ptr::null(),
            log_file_days: 0,
            ini_is_path: 0,
            ini: std::ptr::null(),
            ssl_is_path: 0,
            ssl: std::ptr::null(),
            ssl_pwd: std::ptr::null(),
        })
    }

    pub fn thread_num(mut self, thread_num: i32) -> Self {
        self.0.thread_num = thread_num;
        self
    }

    /// 设置日志级别
    ///
    /// - 0: Trace
    /// - 1: Debug
    /// - 2: Info
    /// - 3: Warn
    /// - 4: Error
    ///
    pub fn log_level(mut self, log_level: i32) -> Self {
        self.0.log_level = log_level;
        self
    }

    /// 设置日志输出方式
    ///
    /// - 0: 输出到控制台
    /// - 2: 输出到文件
    /// - 4: 输出到回调方法
    ///
    /// example:
    ///  三种方式全部输出，使用 0 ｜ 2 ｜ 4
    ///
    pub fn log_mask(mut self, log_mask: i32) -> Self {
        self.0.log_mask = log_mask;
        self
    }

    /// 文件日志保存路径,路径可以不存在(内部可以创建文件夹)
    /// 默认设置为NULL关闭日志输出至文件
    ///
    pub fn log_file_path(mut self, log_file_path: &str) -> Self {
        let log_file_path = const_str_to_ptr!(log_file_path);
        self.0.log_file_path = log_file_path.into_raw();
        self
    }

    /// 文件日志保存天数
    /// 默认设置为0关闭日志文件
    ///
    pub fn log_file_days(mut self, log_file_days: i32) -> Self {
        self.0.log_file_days = log_file_days;
        self
    }

    pub fn ini_is_path(mut self, ini_is_path: i32) -> Self {
        self.0.ini_is_path = ini_is_path;
        self
    }

    pub fn ini(mut self, ini_txt: &str) -> Self {
        let ini_txt = const_str_to_ptr!(ini_txt);
        self.0.ini = ini_txt.into_raw();
        self
    }

    pub fn ini_by_file(mut self, path: &str) -> Self {
        let path = const_str_to_ptr!(path);
        self.0.ini = path.into_raw();
        self.0.ini_is_path = 1;
        self
    }

    pub fn ssl_is_path(mut self, ssl_is_path: i32) -> Self {
        self.0.ssl_is_path = ssl_is_path;
        self
    }

    pub fn ssl(mut self, ssl: &str) -> Self {
        let ssl = const_str_to_ptr!(ssl);
        self.0.ssl = ssl.into_raw();
        self
    }

    pub fn ssl_pwd(mut self, ssl_pwd: &str) -> Self {
        let ssl_pwd = const_str_to_ptr!(ssl_pwd);
        self.0.ssl_pwd = ssl_pwd.into_raw();
        self
    }

    pub fn build(self) {
        unsafe { mk_env_init(&self.0 as *const mk_config) }
    }
}

impl Default for EnvInitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

static EVN_INI: Lazy<Mutex<EnvIni>> = Lazy::new(|| Mutex::new(EnvIni(unsafe { mk_ini_default() })));

pub struct EnvIni(mk_ini);

impl EnvIni {
    /// 创建ini配置对象
    pub fn new() -> Self {
        Self(unsafe { mk_ini_create() })
    }

    /// 创建ini配置对象
    /// 全局默认ini配置，请勿用mk_ini_release释放它
    ///
    pub fn global() -> &'static Mutex<EnvIni> {
        &EVN_INI
    }

    pub fn set_option(&self, key: &str, val: &str) {
        let val = const_str_to_ptr!(val);
        let key = const_str_to_ptr!(key);
        unsafe { mk_ini_set_option(self.0, key.as_ptr(), val.as_ptr()) }
    }

    pub fn set_option_int(&self, key: &str, val: i32) {
        let key = const_str_to_ptr!(key);
        unsafe { mk_ini_set_option_int(self.0, key.as_ptr(), val) }
    }

    pub fn get_option(&self, key: &str) -> String {
        let key = const_str_to_ptr!(key);
        unsafe { const_ptr_to_string!(mk_ini_get_option(self.0, key.as_ptr())) }
    }

    pub fn remove_option(&self, key: &str) -> bool {
        let key = const_str_to_ptr!(key);
        unsafe { mk_ini_del_option(self.0, key.as_ptr()) != 0 }
    }

    pub fn dump(&self) -> String {
        unsafe { const_ptr_to_string!(mk_ini_dump_string(self.0)) }
    }
}

impl Drop for EnvIni {
    fn drop(&mut self) {
        unsafe { mk_ini_release(self.0) }
    }
}

impl AsRef<mk_ini> for EnvIni {
    fn as_ref(&self) -> &mk_ini {
        &self.0
    }
}

unsafe impl Send for EnvIni {}
unsafe impl Sync for EnvIni {}
