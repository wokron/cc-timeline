pub struct ThreadPlacer {
    num_threads: usize,
    place_state: Vec<u64>,
}

impl ThreadPlacer {
    pub fn new(num_threads: usize) -> Self {
        ThreadPlacer {
            num_threads,
            place_state: vec![0u64; num_threads],
        }
    }

    pub fn place_thread(&mut self, prev_tid: u32, ts_ns: u64, dur_ns: u64) -> u32 {
        let mut best_thread = -1isize;
        let mut best_end_ns = 0u64;

        for thread_id in 0..self.num_threads {
            let thread_end_ns = self.place_state[thread_id];
            if thread_end_ns <= ts_ns {
                if best_thread == -1 || thread_end_ns > best_end_ns {
                    best_end_ns = thread_end_ns;
                    best_thread = thread_id as isize;
                }
            }
        }

        if best_thread == -1 {
            // keep original thread
            prev_tid
        } else {
            self.place_state[best_thread as usize] = ts_ns + dur_ns;
            best_thread as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_placer() {
        let mut placer = ThreadPlacer::new(2);
        // 0, 0

        let tid1 = placer.place_thread(10, 1, 99);
        assert_eq!(tid1, 0);
        // 100, 0

        let tid2 = placer.place_thread(11, 50, 100);
        assert_eq!(tid2, 1);
        // 100, 150

        let tid3 = placer.place_thread(12, 120, 50);
        assert_eq!(tid3, 0);
        // 170, 150

        let tid4 = placer.place_thread(13, 130, 50);
        assert_eq!(tid4, 13);
        // 170, 150 (no available thread, keep original)

        let tid6 = placer.place_thread(15, 175, 50);
        assert_eq!(tid6, 0);
        // 255, 150
    }
}
