RELEASE ?=

CARGO_TARGET =$(if $(RELEASE),target/release,target/debug)
CARGO_BUILD_FLAGS =$(if $(RELEASE),--release,)


ACLOCAL_AMFLAGS = -I m4

bin_PROGRAMS = hello_c
hello_c_SOURCES = hello.c
hello_c_CPPFLAGS = -I$(top_build_prefix)hams/target -I$(top_build_prefix)ffi-log2/target
hello_c_LDFLAGS = -lhams -L$(top_builddir)/$(CARGO_TARGET)
hello_c_DEPENDENCIES = $(top_builddir)/$(CARGO_TARGET)/libhams.dylib

$(top_build_prefix)$(CARGO_TARGET)/libhams.dylib:
	cd $(top_srcdir) && make $(CARGO_TARGET)/libhams.dylib

# $(CARGO_TARGET)/libhams.dylib:
# 	$(CARGO) build $(CARGO_BUILD_FLAGS)
