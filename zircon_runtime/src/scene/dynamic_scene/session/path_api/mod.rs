//! 将 RuntimeSessionArchive 的路径型公开方法连接到会话存档服务，供调用方统一处理载入、查询、恢复和跨文件操作。
//! 这里仅组织外部入口；读写、校验和场景状态变化由对应的路径工作流承担。

mod export;
mod load_save;
mod merge;
mod query;
mod restore;
mod transfer;
