pub use self::client::{Client, ClientResponse};
pub use self::error::Error;
pub use self::publisher::{Publisher, PublisherStream};
pub use self::service::Service;
pub use self::subscriber::SubscriberRosConnection;

use crate::rosmsg::RosMsg;
use crate::Clock;
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

mod client;
pub mod error;
mod header;
mod publisher;
mod service;
mod subscriber;
mod util;

pub type ServiceResult<T> = Result<T, String>;

pub trait Message: Clone + Debug + Default + PartialEq + RosMsg + Send + Sync + 'static {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
    fn set_header(&mut self, _clock: &Arc<dyn Clock>, _seq: &Arc<AtomicUsize>) {}
}

pub trait ServicePair: Clone + Debug + Default + PartialEq + Message {
    type Request: RosMsg + Send + 'static;
    type Response: RosMsg + Send + 'static;
}

#[derive(Clone, Debug)]
pub struct Topic {
    pub name: String,
    pub msg_type: String,
    pub md5sum: String,
}
