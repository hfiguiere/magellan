
pkgdatadir = join_paths(datadir, meson.project_name())
gnome = import('gnome')

resources = gnome.compile_resources('gpsami',
  'gpsami.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('localedir', join_paths(get_option('prefix'), get_option('localedir')))
conf.set_quoted('pkgdatadir', pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

# include_bytes! only takes a string literal
resource_conf = configuration_data()
resource_conf.set_quoted('RESOURCEFILE', resources.full_path())
resource_rs = configure_file(
  input: 'static_resources.rs.in',
  output: 'static_resources.rs',
  configuration: resource_conf
)

run_command(
  'cp',
  resource_rs,
  meson.current_source_dir(),
  check: true
)

sources = files(
  'actionqueue.rs',
  'config.rs',
  'devices.rs',
  'drivers.rs',
  'gpsbabel.rs',
  'main.rs',
  'mgapplication.rs',
  'static_resources.rs',
  'utils.rs',
)

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  depends: resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ]
)
