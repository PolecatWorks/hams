#
# Minimal configure.ac to get started
# inspired by https://ljones.dev/blog/gsoc-2/

AC_PREREQ([2.71])

AC_INIT([hams],[0.1])

AM_INIT_AUTOMAKE([1.6 foreign subdir-objects])
m4_ifdef([AM_SILENT_RULES], [
    AM_SILENT_RULES([yes])
])

AC_CANONICAL_HOST

AC_CONFIG_HEADERS([src-c/config.h])

AC_PROG_CC
AM_PROG_CC_C_O

AC_PATH_PROG([CARGO], [cargo], [notfound])
AS_IF([test "$CARGO" = "notfound"], [AC_MSG_ERROR([cargo is required])])

AC_PATH_PROG([RUSTC], [rustc], [notfound])
AS_IF([test "$RUSTC" = "notfound"], [AC_MSG_ERROR([rustc is required])])

LT_INIT

AC_CONFIG_MACRO_DIRS([m4])

AC_CONFIG_FILES([
    Makefile
    src-c/Makefile
])


AC_OUTPUT
