AC_INIT([gpsami], m4_esyscmd([grep ^version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n"]), [hub@figuiere.net])
AM_INIT_AUTOMAKE([1.11 foreign no-dependencies no-dist-gzip dist-xz subdir-objects])

VERSION=$(grep ^version $srcdir/Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")

AC_CHECK_PROG(CARGO, [cargo], [yes], [no])
AS_IF(test x$CARGO = xno,
    AC_MSG_ERROR([cargo is required])
)
AC_CHECK_PROG(RUSTC, [rustc], [yes], [no])
AS_IF(test x$RUSTC = xno,
    AC_MSG_ERROR([rustc is required])
)

dnl Release build we do.
CARGO_TARGET_SUBDIR=release
AC_SUBST(CARGO_TARGET_SUBDIR)

AC_SUBST(VERSION)

AC_CONFIG_FILES([Makefile
        data/gpsami.desktop])

AC_OUTPUT


echo "
You can now build using 'make'

Alternatively you can use 'cargo build'
"
