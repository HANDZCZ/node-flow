#[derive(Debug, PartialEq, Eq)]
pub enum NodeOutput<T> {
    SoftFail,
    Ok(T),
}

impl<T> NodeOutput<T> {
    pub fn ok(self) -> Option<T> {
        match self {
            Self::SoftFail => None,
            Self::Ok(val) => Some(val),
        }
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Self::SoftFail => Err(err),
            Self::Ok(val) => Ok(val),
        }
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn ok_or_else<E>(self, err: impl Fn() -> E) -> Result<T, E> {
        match self {
            Self::SoftFail => Err(err()),
            Self::Ok(val) => Ok(val),
        }
    }
}
