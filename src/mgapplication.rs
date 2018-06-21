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

use actionqueue::{ActionQueueSource, MgAction, QUEUE};
use devices;
use drivers;
use utils;
use Format;

fn post_event(action: MgAction) {
    if let Ok(ref mut q) = QUEUE.lock() {
        q.queue.push_back(action);
    }
}

pub struct MgApplication {
    win: gtk::ApplicationWindow,
    erase_checkbtn: gtk::CheckButton,
    model_combo: gtk::ComboBox,
    model_store: gtk::ListStore,
    port_combo: gtk::ComboBox,
    port_store: gtk::ListStore,

    device_manager: devices::Manager,
    prefs_store: glib::KeyFile,

    model_changed_signal: Option<glib::SignalHandlerId>,
    port_changed_signal: Option<glib::SignalHandlerId>,

    output_dest_dir: path::PathBuf,
}

impl MgApplication {
    pub fn new(gapp: &gtk::Application) -> Rc<RefCell<Self>> {
        let builder = gtk::Builder::new_from_string(include_str!("mgwindow.ui"));
        let window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();
        let erase_checkbtn: gtk::CheckButton = builder.get_object("erase_checkbtn").unwrap();
        let model_combo: gtk::ComboBox = builder.get_object("model_combo").unwrap();
        let port_combo: gtk::ComboBox = builder.get_object("port_combo").unwrap();
        let output_dir_chooser: gtk::FileChooserButton =
            builder.get_object("output_dir_chooser").unwrap();

        gapp.add_window(&window);

        let app = MgApplication {
            win: window,
            erase_checkbtn: erase_checkbtn,
            model_combo: model_combo,
            model_store: gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String]),
            port_combo: port_combo,
            port_store: gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String]),
            device_manager: devices::Manager::new(),
            prefs_store: glib::KeyFile::new(),
            model_changed_signal: None,
            port_changed_signal: None,
            output_dest_dir: path::PathBuf::new(),
        };

        let me = Rc::new(RefCell::new(app));
        me.borrow()
            .device_manager
            .gudev_client
            .connect_uevent(move |_, action, device| {
                let subsystem = device.get_subsystem().unwrap_or("".to_string());
                println!("received event {} {}", action, subsystem);
                post_event(MgAction::RescanDevices)
            });
        {
            let signal_id = me.borrow_mut().model_combo.connect_changed(move |combo| {
                if let Some(id) = combo.get_active_id() {
                    post_event(MgAction::ModelChanged(id));
                }
            });
            me.borrow_mut().model_changed_signal = Some(signal_id);
        }
        {
            let signal_id = me.borrow_mut().port_combo.connect_changed(move |entry| {
                if let Some(id) = entry.get_active_id() {
                    post_event(MgAction::PortChanged(id));
                }
            });
            me.borrow_mut().port_changed_signal = Some(signal_id);
        }
        {
            let dload_action = gio::SimpleAction::new("download", None);
            dload_action.connect_activate(move |_, _| {
                post_event(MgAction::StartDownload);
            });
            dload_action.set_enabled(false);
            me.borrow_mut().win.add_action(&dload_action);
        }

        {
            let erase_action = gio::SimpleAction::new("erase", None);
            erase_action.connect_activate(move |_, _| {
                post_event(MgAction::StartErase);
            });
            erase_action.set_enabled(false);
            me.borrow_mut().win.add_action(&erase_action);
        }
        {
            output_dir_chooser.connect_file_set(move |w| {
                let file_name = w.get_filename();
                match file_name {
                    Some(f) => post_event(MgAction::SetOutputDir(f)),
                    _ => {}
                }
            });
        }

        if me.borrow_mut().load_settings().is_err() {
            println!("Error loading settings");
        }
        output_dir_chooser.set_current_folder(
            me.borrow()
                .prefs_store
                .get_string("output", "dir")
                .unwrap_or("".to_owned()),
        );

        let ctx = glib::MainContext::default();
        if ctx.is_some() {
            let metoo = me.clone();
            let source = ActionQueueSource::new(metoo);
            source.attach(Some(ctx.as_ref().unwrap()));
        }
        me
    }

    fn do_download(&mut self) {
        // we wrap into this because we have early returns
        // and want to ensure the event is posted.
        self.really_do_download();
        post_event(MgAction::DoneDownload);
    }

    fn really_do_download(&self) {
        let device = self.device_manager.get_device();
        if device.is_none() {
            println!("nodriver");
        } else {
            let output_file: path::PathBuf;
            let chooser = gtk::FileChooserDialog::new(
                Some("Save File"),
                Some(&self.win),
                gtk::FileChooserAction::Save,
            );
            chooser.add_buttons(&[
                ("Save", gtk::ResponseType::Ok.into()),
                ("Cancel", gtk::ResponseType::Cancel.into()),
            ]);
            chooser.set_current_folder(
                self.prefs_store
                    .get_string("output", "dir")
                    .unwrap_or("".to_owned()),
            );
            if chooser.run() == gtk::ResponseType::Ok.into() {
                let result = chooser.get_filename();
                chooser.destroy();
                match result {
                    Some(f) => output_file = f,
                    _ => return,
                }
            } else {
                chooser.destroy();
                return;
            }
            let mut d = device.unwrap();
            if d.open() {
                match d.download(Format::Gpx, false) {
                    Ok(temp_output_filename) => {
                        println!("success {}", temp_output_filename.to_str().unwrap());
                        match std::fs::copy(temp_output_filename, &output_file) {
                            Err(e) => self.report_error(
                                &format!("Failed to save {}", output_file.to_str().unwrap()),
                                &e.to_string(),
                            ),
                            _ => {}
                        }
                    }
                    Err(e) => {
                        self.report_error(&format!("Failed to download GPS data."), &e.to_string())
                    }
                }
            }
        }
    }

    fn report_error(&self, message: &str, reason: &str) {
        let dialog = gtk::MessageDialog::new(
            Some(&self.win),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Close,
            message,
        );
        dialog.set_property_secondary_text(Some(reason));
        dialog.run();
        dialog.destroy();
    }

    fn do_erase(&mut self) {
        self.real_do_erase();
        post_event(MgAction::DoneErase);
    }

    fn real_do_erase(&self) {
        let device = self.device_manager.get_device();
        if device.is_none() {
            println!("nodriver");
        } else {
            let mut d = device.unwrap();
            if d.open() {
                match d.erase() {
                    Ok(_) => println!("success erasing"),
                    Err(e) => {
                        self.report_error(&format!("Failed to erase GPS data."), &e.to_string())
                    }
                }
            }
        }
    }

    fn settings_dir() -> path::PathBuf {
        // XXX replace this by glib stuff when we can.
        // Also we treat a failure of this as fatal.
        let mut path: path::PathBuf = std::env::home_dir().unwrap();
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
        match std::fs::create_dir_all(path.clone()) {
            Err(e) => {
                return Err(glib::Error::new(
                    glib::FileError::Failed,
                    &format!("Can't create settings dir '{:?}': {}", path, e),
                ))
            }
            Ok(_) => {}
        }
        path.push("gpsami.ini");

        match self.prefs_store
            .load_from_file(path, glib::KeyFileFlags::NONE)
        {
            Err(e) => {
                println!("error with g_key_file {}", e);
                Err(e)
            }
            Ok(_) => Ok(()),
        }
    }

    /// Start the app.
    pub fn start(&mut self) {
        utils::setup_text_combo(&self.model_combo, &self.model_store);
        utils::setup_text_combo(&self.port_combo, &self.port_store);
        self.populate_model_combo();
        self.win.show_all();
    }

    /// Rescan devices. On start and when new device is connected.
    fn rescan_devices(&mut self) {
        self.populate_model_combo();
    }

    fn populate_port_combo(&mut self, ports: &Vec<drivers::Port>) {
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

        let model = self.prefs_store
            .get_string("device", "model")
            .unwrap_or("".to_string());
        let port = self.prefs_store
            .get_string("device", "port")
            .unwrap_or("".to_string());

        // XXX this is a hack to not have the signal called as we'll end up
        // recursively borrow_mut self via the RefCell
        let model_too = model.clone();
        utils::block_signal(
            &mut self.model_combo,
            self.model_changed_signal.as_ref().unwrap(),
            |obj| {
                obj.set_active_id(model_too.as_ref());
            },
        );
        self.model_changed(&model);

        let port_too = port.clone();
        utils::block_signal(
            &mut self.port_combo,
            self.port_changed_signal.as_ref().unwrap(),
            |obj| {
                obj.set_active_id(port_too.as_ref());
            },
        );
        self.port_changed(&port);
    }

    fn model_changed(&mut self, id: &String) {
        println!("model changed to {}", id);
        self.prefs_store.set_string("device", "model", &id);
        if self.save_settings().is_err() {
            println!("Error loading settings");
        }

        let cap = self.device_manager.device_capability(id);
        if cap.is_some() {
            self.update_device_capability(&cap.unwrap());
            self.device_manager.set_model(id.clone());
            let ports = self.device_manager.get_ports_for_model(id);
            self.populate_port_combo(&ports.unwrap_or(Vec::new()));
        } else {
            // XXX clear device.
        }
    }

    fn update_device_capability(&self, capability: &devices::Capability) {
        self.erase_checkbtn.set_sensitive(capability.can_erase);
        match self.win.lookup_action("erase") {
            Some(a) => match a.downcast::<gio::SimpleAction>() {
                Ok(sa) => sa.set_enabled(capability.can_erase_only),
                _ => {}
            },
            _ => {}
        }
    }

    fn port_changed(&mut self, id: &str) {
        self.prefs_store.set_string("device", "port", id);
        if self.save_settings().is_err() {
            println!("Error loading settings");
        }

        self.device_manager.set_port(id.to_string());
        match self.win.lookup_action("download") {
            Some(a) => match a.downcast::<gio::SimpleAction>() {
                Ok(sa) => sa.set_enabled(id != ""),
                _ => {}
            },
            _ => {}
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
                self.do_erase();
            }
            MgAction::DoneErase => {}
            MgAction::StartDownload => {
                self.do_download();
            }
            MgAction::DoneDownload => {}
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
