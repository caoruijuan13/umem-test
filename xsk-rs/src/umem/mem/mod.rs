mod mmap;
use mmap::Mmap;

use std::{
    io,
    io::{IoSlice, IoSliceMut},
    num::NonZeroU32,
    ptr::NonNull,
    slice,
    sync::{Arc, Mutex},
    collections::VecDeque,
};

use super::{
    frame::{Data, DataMut, FrameDesc, Headroom, HeadroomMut},
    FrameLayout,
};

/// A framed, memory mapped region which functions as the working
/// memory for some UMEM.
#[derive(Clone, Debug)]
pub struct UmemRegion {
    layout: FrameLayout,
    // Keep a copy of the pointer to the mmap region to avoid a double
    // deref, through for example an `Arc<Mmap>`. We know this won't
    // dangle since this struct holds an `Arc`d copy of the mmap
    // region.
    addr: NonNull<libc::c_void>,
    len: usize,
    _mmap: Arc<Mutex<Mmap>>,
    /// available queue of FrameDesc
    available_queue: Arc<Mutex<VecDeque<FrameDesc>>>
}

unsafe impl Send for UmemRegion {}

// SAFETY: this impl is only safe in the context of this library and
// assuming the various unsafe requirements are upheld. Mutations to
// the memory region may occur concurrently but always in disjoint
// sections by either the user space process xor the kernel.
unsafe impl Sync for UmemRegion {}

impl UmemRegion {
    pub(super) fn new(
        frame_count: NonZeroU32,
        frame_layout: FrameLayout,
        use_huge_pages: bool,
    ) -> io::Result<Self> {
        let len = (frame_count.get() as usize) * frame_layout.frame_size();

        let mmap = Mmap::new(len, use_huge_pages)?;

        Ok(Self {
            layout: frame_layout,
            addr: mmap.addr(),
            len,
            _mmap: Arc::new(Mutex::new(mmap)),
            available_queue: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// The size of the underlying memory region.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get a pointer to the start of the memory region.
    #[inline]
    pub fn as_ptr(&self) -> *mut libc::c_void {
        self.addr.as_ptr()
    }

    /// Get an available frame from available queue.
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    // #[inline]
    pub fn get_frame(&mut self) -> Option<FrameDesc> {
        self.available_queue.lock().unwrap().pop_front()
    }

    /// Release a frame to available queue
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    // #[inline]
    pub fn release_frame(&mut self, desc: FrameDesc) {
        self.available_queue.lock().unwrap().push_back(desc);
    }

    /// Get a headroom's IoSlice instance from the frame described by `desc`.
    ///
    /// # Safety
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    /// 'headroom_delta' means the right delta of headroom(just before `data` segment)
    #[inline]
    pub unsafe fn get_headroom_IoSlice(&self, desc: &FrameDesc, headroom_delta: usize) -> IoSlice {
        let ptr = unsafe { self.headroom_delta_ptr(desc, headroom_delta) };
        IoSlice::new(unsafe { slice::from_raw_parts_mut(ptr, headroom_delta) })
    }

    /// Get a headroom's IoSliceMut instance from the frame described by `desc`.
    ///
    /// # Safety
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    /// 'headroom_delta' means the right delta of headroom(just before `data` segment)
    #[inline]
    pub unsafe fn get_headroom_IoSliceMut(&self, desc: &mut FrameDesc, headroom_delta: usize) -> IoSliceMut {
        let ptr = unsafe { self.headroom_delta_ptr(desc, headroom_delta) };
        let d = unsafe { slice::from_raw_parts_mut(ptr, headroom_delta) };
        desc.adjust_headroom(headroom_delta);
        IoSliceMut::new(d)
    }

    /// Get a data's IoSlice instance from the frame described by `desc`.
    ///
    /// # Safety
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    #[inline]
    pub unsafe fn get_data_IoSlice(&self, desc: &FrameDesc) -> IoSlice {
        let data_ptr = unsafe { self.data_ptr(&desc) };
        IoSlice::new(unsafe { slice::from_raw_parts_mut(data_ptr, desc.lengths.data) })
    }

    /// Get a data's IoSliceMut instance from the frame described by `desc`.
    ///
    /// # Safety
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    /// `length` means data length of slice
    #[inline]
    pub unsafe fn get_data_IoSliceMut(&self, desc: &mut FrameDesc, length: usize) -> IoSliceMut {
        let data_ptr = unsafe { self.data_ptr(&desc) };
        let d = unsafe { slice::from_raw_parts_mut(data_ptr, length) };
        desc.adjust_data(length);
        IoSliceMut::new(d)
    }

    /// A pointer to the headroom segment of the frame described by
    /// `desc`.
    ///
    /// # Safety
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    #[inline]
    unsafe fn headroom_ptr(&self, desc: &FrameDesc) -> *mut u8 {
        let addr = desc.addr - self.layout.frame_headroom;
        unsafe { self.as_ptr().add(addr) as *mut u8 }
    }

    /// A pointer to the delta right headroom segment of the frame described by
    /// `desc`.
    ///
    /// # Safety
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    /// `delta` means the right delta of headroom(just before `data` segment) 
    #[inline]
    unsafe fn headroom_delta_ptr(&self, desc: &FrameDesc, delta: usize) -> *mut u8 {
        let addr = desc.addr - delta;
        unsafe { self.as_ptr().add(addr) as *mut u8 }
    }

    /// A pointer to the data segment of the frame described to by
    /// `desc`.
    ///
    /// # Safety
    ///
    /// `desc` must describe a frame belonging to this [`UmemRegion`].
    #[inline]
    pub unsafe fn data_ptr(&self, desc: &FrameDesc) -> *mut u8 {
        unsafe { self.as_ptr().add(desc.addr) as *mut u8 }
    }

    /// See docs for [`super::Umem::frame`].
    #[inline]
    pub unsafe fn frame(&self, desc: &FrameDesc) -> (Headroom, Data) {
        // SAFETY: see `super::Umem::frame`
        unsafe { (self.headroom(desc), self.data(desc)) }
    }

    /// See docs for [`super::Umem::headroom`].
    #[inline]
    pub unsafe fn headroom(&self, desc: &FrameDesc) -> Headroom {
        // SAFETY: see `frame`.
        let headroom_ptr = unsafe { self.headroom_ptr(desc) };

        Headroom::new(unsafe { slice::from_raw_parts(headroom_ptr, desc.lengths.headroom) })
    }

    /// See docs for [`super::Umem::data`].
    #[inline]
    pub unsafe fn data(&self, desc: &FrameDesc) -> Data {
        // SAFETY: see `frame`.
        let data_ptr = unsafe { self.data_ptr(desc) };

        Data::new(unsafe { slice::from_raw_parts(data_ptr, desc.lengths.data) })
    }

    /// See docs for [`super::Umem::frame_mut`].
    #[inline]
    pub unsafe fn frame_mut<'a>(
        &'a self,
        desc: &'a mut FrameDesc,
    ) -> (HeadroomMut<'a>, DataMut<'a>) {
        // SAFETY: see `super::Umem::frame_mut`
        let headroom_ptr = unsafe { self.headroom_ptr(desc) };
        let data_ptr = unsafe { self.data_ptr(desc) };

        let headroom =
            unsafe { slice::from_raw_parts_mut(headroom_ptr, self.layout.frame_headroom) };

        let data = unsafe { slice::from_raw_parts_mut(data_ptr, self.layout.mtu) };

        (
            HeadroomMut::new(&mut desc.lengths.headroom, headroom),
            DataMut::new(&mut desc.lengths.data, data),
        )
    }

    /// See docs for [`super::Umem::headroom_mut`].
    #[inline]
    pub unsafe fn headroom_mut<'a>(&'a self, desc: &'a mut FrameDesc) -> HeadroomMut<'a> {
        // SAFETY: see `frame_mut`.
        let headroom_ptr = unsafe { self.headroom_ptr(desc) };

        let headroom =
            unsafe { slice::from_raw_parts_mut(headroom_ptr, self.layout.frame_headroom) };

        HeadroomMut::new(&mut desc.lengths.headroom, headroom)
    }

    /// See docs for [`super::Umem::data_mut`].
    #[inline]
    pub unsafe fn data_mut<'a>(&'a self, desc: &'a mut FrameDesc) -> DataMut<'a> {
        // SAFETY: see `frame_mut`.
        let data_ptr = unsafe { self.data_ptr(desc) };

        let data = unsafe { slice::from_raw_parts_mut(data_ptr, self.layout.mtu) };

        DataMut::new(&mut desc.lengths.data, data)
    }
}
