Device support
==============

This application has currently only been tested with a Holux M-1200E,
because that's the only device I have.

Supported devices
-----------------

GPSAmi currently only supports devices that gpsbabel supports. Albeit
it is mostly untested. gpsbabel is used to perform the download.

USB Serial or Bluetooth Serial (SPP through rfcomm) are the only
connectivity types supported for now.

Adding devices
--------------

# Device definitions

A device list is in `src/devices.json`. The file is currently inlined in
the code, so you'd need to rebuild the application for changes to take
effect. It is in JSON format.

Each device is defined with the following fields:

* id: string for unique ID
* label: human readable string
* cap: capabilities (a struct)
* driver: the id of the driver.

Capabilities are feature the driver support. This is lifted from
gpsbabel.
* can_erase: the device can be erased after downloading
* can_erase_only: the device can be erased separately
Unsupported capabilties:
* can_log_enable: command to enable logging on the device
* can_shutoff: there is a command to shut the device off

Drivers are defined with the following struct:
* id: id of the driver as referenced by entry in the devices list
* ports: kind of ports the driver support (an array). "UsbSerial" and
  "RfComm" are the only currently supported values.

# If your device needs a new driver.

If it works with gpsbabel already, it probably just needs the driver
entry in the devices.json and adding the proper match pattern in
`devices::get_device()`

If it is something else then it is more complicated. A new driver
might need to be written.

Feel free to file an issue https://gitlab.gnome.org/hub/gpsami/issues

If your device is supported by gpsbabel, please indicate which type
(-i option) or eventually the whole command line you use.

Misc
----

Applications to configure GPS:

* Wintec WBT 201 GPS and Free Operating Systems http://www.daria.co.uk/gps

* BT747 http://www.bt747.org/

* gpsd http://www.catb.org/gpsd/
