use rszlm_sys::*;

use crate::const_str_to_ptr;

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
    /// ???
    /// - 0: 不输出日志
    /// - 1: 输出错误日志
    /// - 2: 输出错误和警告日志
    /// - 3: 输出错误、警告和调试日志
    /// - 4: 输出错误、警告、调试和信息日志
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

    pub fn log_file_path(mut self, log_file_path: &str) -> Self {
        self.0.log_file_path = const_str_to_ptr!(log_file_path);
        self
    }

    pub fn log_file_days(mut self, log_file_days: i32) -> Self {
        self.0.log_file_days = log_file_days;
        self
    }

    pub fn ini_is_path(mut self, ini_is_path: i32) -> Self {
        self.0.ini_is_path = ini_is_path;
        self
    }

    pub fn ini(mut self, ini: &str) -> Self {
        self.0.ini = const_str_to_ptr!(ini);
        self
    }

    pub fn ssl_is_path(mut self, ssl_is_path: i32) -> Self {
        self.0.ssl_is_path = ssl_is_path;
        self
    }

    pub fn ssl(mut self, ssl: &str) -> Self {
        self.0.ssl = const_str_to_ptr!(ssl);
        self
    }

    pub fn ssl_pwd(mut self, ssl_pwd: &str) -> Self {
        self.0.ssl_pwd = const_str_to_ptr!(ssl_pwd);
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
