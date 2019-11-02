use std::collections::HashMap;
use std::sync::Arc;
use std::{error, result};

use hyper::upgrade::Upgraded;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_io::split::WriteHalf;

use crate::endpoints::Frame;

pub type StdResult<T, E> = result::Result<T, E>;
pub type Result<T> = StdResult<T, Box<dyn error::Error + Send + Sync>>;

pub type Receiver = UnboundedReceiver<Frame>;
pub type Sender = UnboundedSender<Frame>;
pub type ClientsMap = HashMap<u128, WriteHalf<Upgraded>>;
pub type ClientsArc = Arc<Mutex<ClientsMap>>;
