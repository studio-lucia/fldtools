INSTALL := install

PREFIX := /usr/local
BINDIR := $(PREFIX)/bin

.PHONY: all fldpack fldunpack clean install

all:
	cargo build --release

clean:
	rm -rf target/release

install: all
	$(INSTALL) -d $(BINDIR)
	$(INSTALL) target/release/fldpack target/release/fldunpack $(BINDIR)
