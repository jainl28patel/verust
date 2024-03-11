use std::fmt;
use std::mem::MaybeUninit;

pub struct ReadBuf<'a> {
    buf: &'a mut [MaybeUninit<u8>],
    filled: usize,
    initialized: usize,
}

impl <'a> ReadBuf<'a> {

    /// constructor
    #[inline]
    pub fn new(buf: &'a mut[u8]) -> ReadBuf<'a> {
        let initialized = buf.len();
        let buf = unsafe { slice_to_uninit_mut(buf) };
        ReadBuf {
            buf,
            filled: 0,
            initialized,
        }
    }

    /// construct uninitialized buffer
    #[inline]
    pub fn unint(buf: &'a mut [MaybeUninit<u8>]) -> ReadBuf<'a> {
        ReadBuf {
            buf,
            filled: 0,
            initialized: 0
        }
    }

    /// total capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// shared ref to filled portion
    #[inline]
    pub fn filled(&self) -> &[u8] {
        let slice = &self.buf[..self.filled];
        unsafe {
            slice_assume_init(slice)
        }
    }

    /// mutable ref to filled portion
    #[inline]
    pub fn filled_mut(&mut self) -> &mut [u8] {
        let slice = &mut self.buf[..self.filled];
        unsafe {
            slice_assume_init_mut(slice)
        }
    }

    /// no of unfilled bytes
    #[inline]
    pub fn remaining(&self) -> usize {
        self.capacity() - self.filled
    }

    /// ref to inner buffer
    #[inline]
    pub unsafe fn inner_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.buf
    }

    #[inline]
    pub fn clear(&mut self) {
        self.filled = 0;
    }

    /// mut ref to unfilled portion
    #[inline]
    pub fn unfilled_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.buf[self.filled..]
    }

    /// `ReadBuf` comprised of the unfilled section up to `n`
    #[inline]
    pub fn take(&mut self, n: usize) -> ReadBuf<'_> {
        let mx = std::cmp::min(self.remaining(), n);
        ReadBuf::unint(&mut self.unfilled_mut()[..mx])
    }

    #[inline]
    pub fn initialized(&self) -> &[u8] {
        let slice = &self.buf[..self.initialized];
        unsafe { slice_assume_init(slice) }
    }

    #[inline]
    pub fn initialized_mut(&mut self) -> &mut [u8] {
        let slice = &mut self.buf[..self.initialized];
        unsafe { slice_assume_init_mut(slice) }
    }

    #[inline]
    pub fn initialize_unfilled(&mut self) -> &mut [u8] {
        self.initialize_unfilled_to(self.remaining())
    }

    #[inline]
    #[track_caller]
    pub fn initialize_unfilled_to(&mut self, n: usize) -> &mut [u8] {
        assert!(self.remaining() >= n, "n overflows remaining");
        let end = self.filled + n;

        if self.initialized < end {
            unsafe {
                self.buf[self.initialized..end]
                    .as_mut_ptr()
                    .write_bytes(0, end - self.initialized);
            }
            self.initialized = end;
        }

        let slice = &mut self.buf[self.filled..end];
        unsafe { slice_assume_init_mut(slice) }
    }

    #[inline]
    #[track_caller]
    pub fn advance(&mut self, n: usize) {
        let new = self.filled.checked_add(n).expect("filled overflow");
        self.set_filled(new);
    }

    #[inline]
    #[track_caller]
    pub fn set_filled(&mut self, n: usize) {
        assert!(
            n <= self.initialized,
            "filled must not become larger than initialized"
        );
        self.filled = n;
    }

    #[inline]
    pub unsafe fn assume_init(&mut self, n: usize) {
        let new = self.filled + n;
        if new > self.initialized {
            self.initialized = new;
        }
    }

    #[inline]
    #[track_caller]
    pub fn put_slice(&mut self, buf: &[u8]) {
        assert!(
            self.remaining() >= buf.len(),
            "buf.len() must fit in remaining()"
        );

        let amt = buf.len();
        let end = self.filled + amt;

        unsafe {
            self.buf[self.filled..end]
                .as_mut_ptr()
                .cast::<u8>()
                .copy_from_nonoverlapping(buf.as_ptr(), amt);
        }

        if self.initialized < end {
            self.initialized = end;
        }
        self.filled = end;
    }

}

unsafe impl<'a> bytes::BufMut for ReadBuf<'a> {
    fn remaining_mut(&self) -> usize {
        self.remaining()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.assume_init(cnt);
        self.advance(cnt);
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        let unfilled = self.unfilled_mut();
        let len = unfilled.len();
        let ptr = unfilled.as_mut_ptr() as *mut u8;

        unsafe { bytes::buf::UninitSlice::from_raw_parts_mut(ptr, len) }
    }
}

impl fmt::Debug for ReadBuf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadBuf")
            .field("filled", &self.filled)
            .field("initialized", &self.initialized)
            .field("capacity", &self.capacity())
            .finish()
    }
}


unsafe fn slice_to_uninit_mut(slice: &mut [u8]) -> &mut [MaybeUninit<u8>] {
    &mut *(slice as *mut [u8] as *mut [MaybeUninit<u8>])
}

unsafe fn slice_assume_init(slice: &[MaybeUninit<u8>]) -> &[u8] {
    &*(slice as *const [MaybeUninit<u8>] as *const [u8])
}

unsafe fn slice_assume_init_mut(slice: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    &mut *(slice as *mut [MaybeUninit<u8>] as *mut [u8])
}
