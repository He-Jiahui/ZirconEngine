//! 收据管线统一的可传播错误。
//! 构建、签发与核验把失败返回 CLI 出口，避免把无效收据当作成功工件发布。

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductReceiptError {
    message: String,
}

impl ProductReceiptError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ProductReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProductReceiptError {}
