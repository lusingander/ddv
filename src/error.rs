pub type AppResult<T> = std::result::Result<T, AppError>;

#[allow(dead_code)]
pub struct AppError {
    pub msg: String,
    pub cause: Option<Box<dyn std::error::Error + Send + 'static>>,
}

#[allow(dead_code)]
impl AppError {
    pub fn new<E: std::error::Error + Send + 'static>(msg: impl Into<String>, e: E) -> AppError {
        AppError {
            msg: msg.into(),
            cause: Some(Box::new(e)),
        }
    }

    pub fn msg(msg: impl Into<String>) -> AppError {
        AppError {
            msg: msg.into(),
            cause: None,
        }
    }

    pub fn error<E: std::error::Error + Send + 'static>(e: E) -> AppError {
        AppError {
            msg: e.to_string(),
            cause: Some(Box::new(e)),
        }
    }
}
