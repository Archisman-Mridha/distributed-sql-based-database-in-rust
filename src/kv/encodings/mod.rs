pub mod key;

use serde::{Deserialize, Serialize};

pub trait Key<'deserializedValue>: Serialize + Deserialize<'deserliazedValue> {}
