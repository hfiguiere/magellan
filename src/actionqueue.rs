// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use glib;
use glib::translate;
use glib_sys as glib_ffi;

use std;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem;
use std::path;
use std::rc::Rc;
use std::sync::Mutex;

use mgapplication::MgApplication;

pub enum MgAction {
    RescanDevices,
    ModelChanged(String),
    PortChanged(String),
    StartErase,
    DoneErase,
    StartDownload,
    DoneDownload,
    SetOutputDir(path::PathBuf),
}

#[derive(Default)]
pub struct ActionQueue {
    pub queue: VecDeque<MgAction>,
}

#[repr(C)]
pub struct ActionQueueSource {
    source: glib_ffi::GSource,
    app: Rc<RefCell<MgApplication>>,
}

lazy_static! {
    pub static ref QUEUE: Mutex<ActionQueue> = Mutex::new(ActionQueue::default());
}

impl ActionQueueSource {
    pub fn new_source(app: Rc<RefCell<MgApplication>>) -> glib::Source {
        unsafe {
            let source = glib_ffi::g_source_new(
                translate::mut_override(&SOURCE_FUNCS),
                mem::size_of::<ActionQueueSource>() as u32,
            );
            {
                let source = &mut *(source as *mut ActionQueueSource);
                std::ptr::write(&mut source.app, app);
            }
            translate::from_glib_full(source)
        }
    }
}

unsafe extern "C" fn prepare(
    _source: *mut glib_ffi::GSource,
    timeout: *mut i32,
) -> glib_ffi::gboolean {
    *timeout = -1;
    if let Ok(ref mut q) = QUEUE.lock() {
        if !q.queue.is_empty() {
            return glib_ffi::GTRUE;
        }
    }
    glib_ffi::GFALSE
}

unsafe extern "C" fn check(_source: *mut glib_ffi::GSource) -> glib_ffi::gboolean {
    if let Ok(ref mut q) = QUEUE.lock() {
        if !q.queue.is_empty() {
            return glib_ffi::GTRUE;
        }
    }
    glib_ffi::GFALSE
}

unsafe extern "C" fn dispatch(
    source: *mut glib_ffi::GSource,
    callback: glib_ffi::GSourceFunc,
    _user_data: glib_ffi::gpointer,
) -> glib_ffi::gboolean {
    assert!(callback.is_none());
    let a: Option<MgAction>;
    if let Ok(ref mut q) = QUEUE.lock() {
        a = q.queue.pop_front();
    } else {
        return glib_ffi::G_SOURCE_REMOVE;
    }
    if let Some(a) = a {
        let source = &mut *(source as *mut ActionQueueSource);
        source.app.borrow_mut().process_event(a);
    }
    glib_ffi::G_SOURCE_CONTINUE
}

unsafe extern "C" fn finalize(source: *mut glib_ffi::GSource) {
    let _source = &mut *(source as *mut ActionQueueSource);
    // XXX not sure, I think I need to get rid of the app.
}

static SOURCE_FUNCS: glib_ffi::GSourceFuncs = glib_ffi::GSourceFuncs {
    check: Some(check),
    prepare: Some(prepare),
    dispatch: Some(dispatch),
    finalize: Some(finalize),
    closure_callback: None,
    closure_marshal: None,
};
