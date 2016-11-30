
desktop_files = data/gpsami.desktop

desktopdir   = $(datadir)/applications
desktop_DATA = $(desktop_files)

icons_files = data/gpsami.svg
iconsdir    = $(datadir)/icons/scalable/
icons_DATA  = $(icons_files)

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
	$(icons_files) \
	$(null)

gtk_update_icon_cache = gtk-update-icon-cache -f -t $(datadir)/icons/scalable

install-data-hook: update-icon-cache
uninstall-hook: update-icon-cache
update-icon-cache:
	@-if test -z "$(DESTDIR)"; then \
		echo "Updating Gtk icon cache."; \
		$(gtk_update_icon_cache); \
	else \
		echo "*** Icon cache not updated.  After (un)install, run this:"; \
		echo "***   $(gtk_update_icon_cache)"; \
	fi

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
