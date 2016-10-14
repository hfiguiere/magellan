
desktop_files = data/gpsami.desktop

desktopdir   = $(datadir)/applications
desktop_DATA = $(desktop_files)

ui_files = src/mgwindow.ui \
	$(null)

EXTRA_DIST = Cargo.toml \
	src/devices.json  \
	src/devices.rs  \
	src/drivers.rs  \
	src/gpsbabel.rs  \
	src/main.rs \
	src/mgapplication.rs \
	src/utils.rs \
	$(ui_files) \
	$(desktop_in_files) \
	$(null)

EXTRA_PROGRAMS = target/@CARGO_TARGET_DIR@/gpsami

# we need this dummy target for make dist
target_@CARGO_TARGET_DIR@_gpsami_SOURCES =

all-local:
	cargo build --release

clean-local:
	-cargo clean

install-exec-local:
	$(MKDIR_P) $(DESTDIR)$(bindir)
	$(INSTALL) -c -m 755 target/@CARGO_TARGET_DIR@/gpsami $(DESTDIR)$(bindir)

uninstall-local:
	rm $(DESTDIR)$(bindir)/gpsami

DISTCLEANFILES =                                \
	$(desktop_files)                        \
	$(null)

check:
	desktop-file-validate $(desktop_files)
