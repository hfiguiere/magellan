# Adatped from:
# https://gitlab.gnome.org/World/podcasts/blob/master/meson.build

project(
  'gpsami', 'rust',
  version: '0.1.0',
  license: 'GPLv3',
)

if get_option('profile') == 'development'
  profile = 'Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format (vcs_tag)
  endif
else
  profile = ''
  version_suffix = ''
endif

application_id = 'net.figuiere.gpsami@0@'.format(profile)
i18n = import('i18n')

subdir('po')
podir = join_paths (meson.source_root (), 'po')

gpsami_version = meson.project_version()
version_array = gpsami_version.split('.')
gpsami_major_version = version_array[0].to_int()
gpsami_minor_version = version_array[1].to_int()
gpsami_version_micro = version_array[2].to_int()

gpsami_prefix = get_option('prefix')
gpsami_bindir = join_paths(gpsami_prefix, get_option('bindir'))
gpsami_localedir = join_paths(gpsami_prefix, get_option('localedir'))

gpsami_conf = configuration_data()
gpsami_conf.set('appid', application_id)
gpsami_conf.set('bindir', gpsami_bindir)

datadir = get_option('datadir')
#subdir('gpsami-gtk/resources')

cargo = find_program('cargo', required: false)
gresource = find_program('glib-compile-resources', required: false)
cargo_vendor = find_program('cargo-vendor', required: false)
cargo_script = find_program('scripts/cargo.sh')

cargo_release = custom_target('cargo-build',
                        build_always_stale: true,
                        output: ['gpsami'],
                        install: true,
                        install_dir: gpsami_bindir,
                        command: [cargo_script,
                                  '@CURRENT_SOURCE_DIR@',
                                  '@OUTPUT@',
                                  gpsami_localedir,
                                  application_id,
                                  gpsami_version + version_suffix,
                                  profile
                                 ])

run_target('release', command: ['scripts/release.sh',
                                meson.project_name() + '-' + gpsami_version
                               ])
