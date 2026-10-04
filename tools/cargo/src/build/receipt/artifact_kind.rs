//! 收据中构建产物的分类身份。
//! 发行、跨分区唯一性检查和落盘核验共同消费；类别参与签名闭包，不能只按文件扩展名推断。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Executable,
    DynamicLibrary,
    SymbolFile,
    Resource,
    Sbom,
}
