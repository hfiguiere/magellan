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

use dirs;
use gio;
use gio::prelude::*;
use glib;
use gtk;
use gtk::prelude::*;
use gudev::{ClientExt, DeviceExt};

use std;
use std::cell::RefCell;
use std::path;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

use actionqueue::{ActionQueueSource, MgAction, QUEUE};
use devices;
use drivers;
use utils;
use Format;

enum UIState {
    Idle,
    InProgress,
}

fn post_event(action: MgAction) {
    if let Ok(ref mut q) = QUEUE.lock() {
        q.queue.push_back(action);
    }
}

pub struct MgApplication {
    window: gtk::ApplicationWindow,
    content_box: gtk::Box,
    erase_checkbtn: gtk::CheckButton,
    model_combo: gtk::ComboBox,
    model_store: gtk::ListStore,
    port_combo: gtk::ComboBox,
    port_store: gtk::ListStore,

    device_manager: devices::Manager,
    prefs_store: glib::KeyFile,

    output_dest_dir: path::PathBuf,
}

impl MgApplication {
    pub fn new(gapp: &gtk::Application) -> Rc<RefCell<Self>> {
        let builder = gtk::Builder::new_from_string(include_str!("mgwindow.ui"));
        let window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();
        let content_box = builder.get_object::<gtk::Box>("content_box").unwrap();
        let erase_checkbtn: gtk::CheckButton = builder.get_object("erase_checkbtn").unwrap();
        let model_combo: gtk::ComboBox = builder.get_object("model_combo").unwrap();
        let port_combo: gtk::ComboBox = builder.get_object("port_combo").unwrap();
        let output_dir_chooser: gtk::FileChooserButton =
            builder.get_object("output_dir_chooser").unwrap();

        gapp.add_window(&window);

        model_combo.connect_changed(move |combo| {
            if let Some(id) = combo.get_active_id() {
                post_event(MgAction::ModelChanged(id));
            }
        });
        port_combo.connect_changed(move |entry| {
            if let Some(id) = entry.get_active_id() {
                post_event(MgAction::PortChanged(id));
            }
        });
        let dload_action = gio::SimpleAction::new("download", None);
        dload_action.connect_activate(move |_, _| {
            post_event(MgAction::StartDownload);
        });
        dload_action.set_enabled(false);
        window.add_action(&dload_action);

        let erase_action = gio::SimpleAction::new("erase", None);
        erase_action.connect_activate(move |_, _| {
            post_event(MgAction::StartErase);
        });
        erase_action.set_enabled(false);
        window.add_action(&erase_action);

        output_dir_chooser.connect_file_set(move |w| {
            let file_name = w.get_filename();
            if let Some(f) = file_name {
                post_event(MgAction::SetOutputDir(f));
            }
        });

        let device_manager = devices::Manager::new();
        device_manager
            .gudev_client
            .connect_uevent(move |_, action, device| {
                let subsystem = device.get_subsystem().unwrap_or_default();
                println!("received event {} {}", action, subsystem);
                post_event(MgAction::RescanDevices)
            });

        let app = MgApplication {
            window,
            content_box,
            erase_checkbtn,
            model_combo,
            model_store: gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String]),
            port_combo,
            port_store: gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String]),
            device_manager,
            prefs_store: glib::KeyFile::new(),
            output_dest_dir: path::PathBuf::new(),
        };

        let me = Rc::new(RefCell::new(app));
        if me.borrow_mut().load_settings().is_err() {
            println!("Error loading settings");
        }
        output_dir_chooser.set_current_folder(
            me.borrow()
                .prefs_store
                .get_string("output", "dir")
                .unwrap_or_default(),
        );

        let ctx = glib::MainContext::default();
        let metoo = me.clone();
        let source = ActionQueueSource::new_source(metoo);
        source.attach(Some(&ctx));

        me
    }

    fn do_download(&mut self) {
        let device = self.device_manager.get_device();
        if device.is_none() {
            println!("nodriver");
            post_event(MgAction::DoneDownload(drivers::Error::NoDriver));
            return;
        }
        let output_file: path::PathBuf;
        let chooser = gtk::FileChooserDialog::new(
            Some("Save File"),
            Some(&self.window),
            gtk::FileChooserAction::Save,
        );
        chooser.add_buttons(&[
            ("Save", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into()),
        ]);
        chooser.set_current_folder(
            self.prefs_store
                .get_string("output", "dir")
                .unwrap_or_default(),
        );
        if chooser.run() == gtk::ResponseType::Ok.into() {
            let result = chooser.get_filename();
            chooser.destroy();
            if let Some(f) = result {
                output_file = f;
            } else {
                post_event(MgAction::DoneDownload(drivers::Error::Cancelled));
                return;
            }
        } else {
            chooser.destroy();
            post_event(MgAction::DoneDownload(drivers::Error::Cancelled));
            return;
        }
        let mut d = device.unwrap();
        thread::spawn(move || {
            post_event(if Arc::get_mut(&mut d).unwrap().open() {
                match d.download(Format::Gpx, false) {
                    Ok(temp_output_filename) => {
                        println!("success {}", temp_output_filename.to_str().unwrap());
                        if let Err(e) = std::fs::copy(temp_output_filename, &output_file) {
                            MgAction::DoneDownload(drivers::Error::IOError(e))
                        } else {
                            MgAction::DoneDownload(drivers::Error::Success)
                        }
                    }
                    Err(e) => MgAction::DoneDownload(e),
                }
            } else {
                MgAction::DoneErase(drivers::Error::Failed("open failed".to_string()))
            });
        });
    }

    fn report_error(&self, message: &str, reason: &str) {
        let dialog = gtk::MessageDialog::new(
            Some(&self.window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Close,
            message,
        );
        dialog.set_property_secondary_text(Some(reason));
        dialog.run();
        dialog.destroy();
    }

    fn do_erase(&self) {
        let device = self.device_manager.get_device();
        if device.is_none() {
            println!("nodriver");
            post_event(MgAction::DoneErase(drivers::Error::NoDriver));
            return;
        }
        let mut d = device.unwrap();
        thread::spawn(move || {
            post_event(if Arc::get_mut(&mut d).unwrap().open() {
                match d.erase() {
                    Ok(_) => {
                        println!("success erasing");
                        MgAction::DoneErase(drivers::Error::Success)
                    }
                    Err(e) => MgAction::DoneErase(e),
                }
            } else {
                MgAction::DoneErase(drivers::Error::Failed("open failed".to_string()))
            });
        });
    }

    fn settings_dir() -> path::PathBuf {
        // XXX replace this by glib stuff when we can.
        // Also we treat a failure of this as fatal.
        let mut path: path::PathBuf = dirs::home_dir().unwrap();
        path.push(".gpsami");
        path
    }

    fn save_settings(&self) -> Result<(), glib::Error> {
        let mut path = Self::settings_dir();
        path.push("gpsami.ini");
        self.prefs_store.save_to_file(path.to_str().unwrap())
    }

    fn set_output_destination_dir(&mut self, output: &path::Path) {
        self.output_dest_dir = output.to_owned();
    }

    pub fn load_settings(&mut self) -> Result<(), glib::Error> {
        let mut path = Self::settings_dir();
        if let Err(e) = std::fs::create_dir_all(path.clone()) {
            return Err(glib::Error::new(
                glib::FileError::Failed,
                &format!("Can't create settings dir '{:?}': {}", path, e),
            ));
        }
        path.push("gpsami.ini");

        if let Err(e) = self
            .prefs_store
            .load_from_file(path, glib::KeyFileFlags::NONE)
        {
            println!("error with g_key_file {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Start the app.
    pub fn start(&mut self) {
        utils::setup_text_combo(&self.model_combo, &self.model_store);
        utils::setup_text_combo(&self.port_combo, &self.port_store);
        self.populate_model_combo();
        self.window.show_all();
    }

    /// Rescan devices. On start and when new device is connected.
    fn rescan_devices(&mut self) {
        self.populate_model_combo();
    }

    fn populate_port_combo(&mut self, ports: &[drivers::Port]) {
        self.port_store.clear();
        for port in ports {
            println!("adding port {:?}", port);
            utils::add_text_row(&self.port_store, &port.path.to_str().unwrap(), &port.id);
        }
    }

    fn populate_model_combo(&mut self) {
        self.model_store.clear();
        {
            let devices = self.device_manager.devices_desc();
            for dev in devices {
                utils::add_text_row(&self.model_store, &dev.id, &dev.label);
            }
        }

        let model = self
            .prefs_store
            .get_string("device", "model")
            .unwrap_or_default();
        let port = self
            .prefs_store
            .get_string("device", "port")
            .unwrap_or_default();

        self.model_combo.set_active_id(model.as_ref());
        self.port_combo.set_active_id(port.as_ref());
    }

    fn model_changed(&mut self, id: &str) {
        println!("model changed to {}", id);
        self.prefs_store.set_string("device", "model", &id);
        if self.save_settings().is_err() {
            println!("Error loading settings");
        }

        let cap = self.device_manager.device_capability(id);
        if cap.is_some() {
            self.update_device_capability(&cap.unwrap());
            self.device_manager.set_model(id);
            let ports = self.device_manager.get_ports_for_model(id);
            self.populate_port_combo(&ports.unwrap_or_default());
        } else {
            // XXX clear device.
        }
    }

    fn update_device_capability(&self, capability: &devices::Capability) {
        self.erase_checkbtn.set_sensitive(capability.can_erase);
        if let Some(a) = self.window.lookup_action("erase") {
            if let Ok(sa) = a.downcast::<gio::SimpleAction>() {
                sa.set_enabled(capability.can_erase_only);
            }
        }
    }

    fn port_changed(&mut self, id: &str) {
        self.prefs_store.set_string("device", "port", id);
        if self.save_settings().is_err() {
            println!("Error loading settings");
        }

        self.device_manager.set_port(id);
        if let Some(a) = self.window.lookup_action("download") {
            if let Ok(sa) = a.downcast::<gio::SimpleAction>() {
                sa.set_enabled(id != "");
            }
        }
    }

    fn set_state(&mut self, state: UIState) {
        match state {
            UIState::Idle => {
                self.content_box.set_sensitive(true);
            }
            UIState::InProgress => {
                self.content_box.set_sensitive(false);
            }
        }
    }

    pub fn process_event(&mut self, evt: MgAction) {
        match evt {
            MgAction::RescanDevices => {
                self.rescan_devices();
            }
            MgAction::ModelChanged(ref id) => {
                self.model_changed(id);
            }
            MgAction::PortChanged(ref id) => self.port_changed(id),
            MgAction::StartErase => {
                self.set_state(UIState::InProgress);
                self.do_erase();
            }
            MgAction::DoneErase(e) => {
                match e {
                    drivers::Error::Success | drivers::Error::Cancelled => {}
                    _ => self.report_error("Error erasing GPS data.", &e.to_string()),
                }
                self.set_state(UIState::Idle);
            }
            MgAction::StartDownload => {
                self.set_state(UIState::InProgress);
                self.do_download();
            }
            MgAction::DoneDownload(e) => {
                match e {
                    drivers::Error::Success | drivers::Error::Cancelled => {}
                    _ => self.report_error("Error downloading GPS data.", &e.to_string()),
                }
                self.set_state(UIState::Idle);
            }
            MgAction::SetOutputDir(f) => {
                self.set_output_destination_dir(f.as_ref());
                self.prefs_store
                    .set_string("output", "dir", f.to_str().unwrap());
                if self.save_settings().is_err() {
                    println!("Error loading settings");
                }
            }
        }
    }
}
