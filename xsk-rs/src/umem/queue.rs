use std::collections::VecDeque;
use super::{
    frame::{Data, DataMut, FrameDesc, Headroom, HeadroomMut},
    FrameLayout,
};

/// Queues about umem 
#[derive(Clone, Debug)]
pub struct Queue {
    /// available queue of FrameDesc
    available_queue: VecDeque<usize>
    // available_queue: Arc<Mutex<VecDeque<FrameDesc>>>
}

impl Default for Queue {
    fn default() -> Self {
        Queue {
            available_queue: VecDeque::default()
        }
    }
}

impl Queue {
    /// Init Queue
    #[inline]
    pub fn init_queue(q: VecDeque<usize>) -> Self {
        Queue {
            available_queue: q
        }
    }

    /// Get an available frame from available queue.
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    #[inline]
    pub fn get_frame(&mut self) -> Option<usize> {
        // self.available_queue.lock().unwrap().pop_front()
        self.available_queue.pop_front()
    }

    /// Release a frame to available queue
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    #[inline]
    pub fn release_frame(&mut self, desc: usize) {
        // self.available_queue.lock().unwrap().push_back(desc);
        self.available_queue.push_back(desc);
    }
}