use serde_json;

#[derive(thiserror::Error, Debug)]
pub enum SocketError {
    BinanceConnectionError(tungstenite::Error),
    BinanceSerdeJsonError(serde_json::Error),

    BybitConnectionError(tungstenite::Error),
    BybitSerdeJsonError(serde_json::Error),

    OkxConnectionError(tungstenite::Error),
    OkxSerdeJsonError(serde_json::Error),
}

impl std::fmt::Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketError::BinanceConnectionError(e) => write!(f, "BinanceConnectionError: {e:?}"),
            SocketError::BinanceSerdeJsonError(e) => write!(f, "BinanceSerdeJsonError: {e:?}"),

            SocketError::BybitConnectionError(e) => write!(f, "BybitConnectionError: {e:?}"),
            SocketError::BybitSerdeJsonError(e) => write!(f, "BybitSerdeJsonError: {e:?}"),

            SocketError::OkxConnectionError(e) => write!(f, "OkxConnectionError: {e:?}"),
            SocketError::OkxSerdeJsonError(e) => write!(f, "OkxSerdeJsonError: {e:?}"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    BinanceApiError(reqwest::Error),
    BinanceSymbolError,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::BinanceApiError(e) => write!(f, "BinanceApiError: {e:?}"),
            HttpError::BinanceSymbolError => write!(f, "BinanceSymbolError"),
        }
    }
}