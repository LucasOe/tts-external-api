use crate::messages::{Answer, Message};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("message was of type {0:?}")]
    MessageError(Message),
    #[error("answer was of type {0:?}")]
    AnswerError(Answer),
}
