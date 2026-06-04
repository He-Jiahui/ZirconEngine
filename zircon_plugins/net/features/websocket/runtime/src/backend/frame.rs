use zircon_runtime::core::framework::net::{NetWebSocketCloseReason, NetWebSocketFrame};

pub(super) fn frame_to_message(
    frame: NetWebSocketFrame,
) -> tokio_tungstenite::tungstenite::Message {
    match frame {
        NetWebSocketFrame::Text(text) => tokio_tungstenite::tungstenite::Message::Text(text.into()),
        NetWebSocketFrame::Binary(bytes) => {
            tokio_tungstenite::tungstenite::Message::Binary(bytes.into())
        }
        NetWebSocketFrame::Ping(bytes) => {
            tokio_tungstenite::tungstenite::Message::Ping(bytes.into())
        }
        NetWebSocketFrame::Pong(bytes) => {
            tokio_tungstenite::tungstenite::Message::Pong(bytes.into())
        }
        NetWebSocketFrame::Close(reason) => tokio_tungstenite::tungstenite::Message::Close(Some(
            tokio_tungstenite::tungstenite::protocol::CloseFrame {
                code: close_code_from_u16(reason.code),
                reason: reason.reason.into(),
            },
        )),
    }
}

pub(super) fn message_to_frame(
    message: tokio_tungstenite::tungstenite::Message,
) -> NetWebSocketFrame {
    match message {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            NetWebSocketFrame::Text(text.to_string())
        }
        tokio_tungstenite::tungstenite::Message::Binary(bytes) => {
            NetWebSocketFrame::Binary(bytes.to_vec())
        }
        tokio_tungstenite::tungstenite::Message::Ping(bytes) => {
            NetWebSocketFrame::Ping(bytes.to_vec())
        }
        tokio_tungstenite::tungstenite::Message::Pong(bytes) => {
            NetWebSocketFrame::Pong(bytes.to_vec())
        }
        tokio_tungstenite::tungstenite::Message::Close(reason) => {
            let reason = reason
                .map(|reason| NetWebSocketCloseReason {
                    code: u16::from(reason.code),
                    reason: reason.reason.to_string(),
                    clean: true,
                })
                .unwrap_or_else(|| NetWebSocketCloseReason::normal("peer closed"));
            NetWebSocketFrame::Close(reason)
        }
        tokio_tungstenite::tungstenite::Message::Frame(_) => {
            NetWebSocketFrame::Close(NetWebSocketCloseReason::normal("raw frame unsupported"))
        }
    }
}

fn close_code_from_u16(
    code: u16,
) -> tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode {
    use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

    match code {
        1000 => CloseCode::Normal,
        1001 => CloseCode::Away,
        1002 => CloseCode::Protocol,
        1003 => CloseCode::Unsupported,
        1005 => CloseCode::Status,
        1006 => CloseCode::Abnormal,
        1007 => CloseCode::Invalid,
        1008 => CloseCode::Policy,
        1009 => CloseCode::Size,
        1010 => CloseCode::Extension,
        1011 => CloseCode::Error,
        1012 => CloseCode::Restart,
        1013 => CloseCode::Again,
        other => CloseCode::Library(other),
    }
}
