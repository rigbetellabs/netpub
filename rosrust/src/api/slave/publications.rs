use crate::api::error;
use crate::tcpros::{Publisher, PublisherStream, Topic};
use crate::util::FAILED_TO_LOCK;
use crate::{Message, RawMessageDescription};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct PublicationsTracker {
    mapping: Arc<Mutex<HashMap<String, Publisher>>>,
}

impl PublicationsTracker {
    #[inline]
    pub fn get_topic_names<T: FromIterator<String>>(&self) -> T {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .keys()
            .cloned()
            .collect()
    }

    #[inline]
    pub fn get_topics<T: FromIterator<Topic>>(&self) -> T {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .values()
            .map(Publisher::get_topic)
            .cloned()
            .collect()
    }

    #[inline]
    pub fn get_port(&self, topic: &str) -> Option<i32> {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .get(topic)
            .map(|publisher| i32::from(publisher.port))
    }

    pub fn add<T: Message>(
        &self,
        hostname: &str,
        topic: &str,
        queue_size: usize,
        caller_id: &str,
        message_description: RawMessageDescription,
    ) -> error::tcpros::Result<PublisherStream<T>> {
        use std::collections::hash_map::Entry;
        match self
            .mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .entry(String::from(topic))
        {
            Entry::Occupied(publisher_entry) => publisher_entry
                .get()
                .stream(queue_size, message_description),
            Entry::Vacant(entry) => {
                let publisher = Publisher::new(
                    format!("{}:0", hostname).as_str(),
                    topic,
                    queue_size,
                    caller_id,
                    message_description.clone(),
                )?;
                entry
                    .insert(publisher)
                    .stream(queue_size, message_description)
            }
        }
    }

    #[inline]
    pub fn remove(&self, topic: &str) {
        self.mapping.lock().expect(FAILED_TO_LOCK).remove(topic);
    }
}
