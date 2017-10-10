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

extern crate gio;
extern crate glib_sys;
extern crate glib;
extern crate gtk;
extern crate libudev;
extern crate gudev;
extern crate rustc_serialize;

use gio::prelude::*;

use mgapplication::MgApplication;

mod mgapplication;
mod devices;
mod drivers;
mod gpsbabel;
mod utils;


pub enum Format {
    None,
    Gpx,
    Kml,
}

/// Init Gtk and stuff.
fn init() {
    use std::sync::{Once, ONCE_INIT};

    static START: Once = ONCE_INIT;

    START.call_once(|| {
        // run initialization here
        if gtk::init().is_err() {
            panic!("Failed to initialize GTK.");
        }
    });
}

fn main() {

    init();

    let gapp = gtk::Application::new(Some("net.figuiere.gpsami"),
                                         gio::APPLICATION_FLAGS_NONE).unwrap();

    gapp.connect_activate(move |gapp| {
        let app = MgApplication::new(&gapp);

        app.borrow_mut().start();
    });

    gapp.run(&[]);
}

#[test]
fn it_works() {
}
