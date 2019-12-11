use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use hyper::upgrade::Upgraded;
use tokio::io::WriteHalf;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::ws::event::Event;

pub type Receiver = UnboundedReceiver<Event>;
pub type Sender = UnboundedSender<Event>;
pub type ClientsMap = HashMap<u128, WriteHalf<Upgraded>>;
pub type ClientsRc = Rc<RefCell<ClientsMap>>;
