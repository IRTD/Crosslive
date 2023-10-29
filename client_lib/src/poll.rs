use super::CrossHandle;
use cross_messages::*;
use tokio::sync::mpsc;

pub struct Poll {
    pool: rayon::ThreadPool,
    pub(super) handle: CrossHandle,
}

impl Poll {
    pub fn new(
        registered_id: ID,
        num_threads: usize,
    ) -> anyhow::Result<(Self, mpsc::Receiver<Message>)> {
        let (tx, rx) = mpsc::channel(24);
        let handle = CrossHandle { tx, registered_id };
        let poll = Poll {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()?,
            handle,
        };

        Ok((poll, rx))
    }

    pub fn register<F>(&self, op: F)
    where
        F: FnOnce(CrossHandle) + Send + 'static,
    {
        let cloned_handle = self.handle.clone();
        self.pool.spawn(|| op(cloned_handle))
    }
}
