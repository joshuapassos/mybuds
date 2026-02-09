PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share

.PHONY: build install uninstall clean

build:
	cargo build --release

install:
	@test -f target/release/mybuds || { echo "Run 'make build' first (as normal user)"; exit 1; }
	install -Dm755 target/release/mybuds $(DESTDIR)$(BINDIR)/mybuds
	install -Dm644 mybuds.desktop $(DESTDIR)$(DATADIR)/applications/mybuds.desktop
	install -Dm644 assets/icon.svg $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/mybuds.svg

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/mybuds
	rm -f $(DESTDIR)$(DATADIR)/applications/mybuds.desktop
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/mybuds.svg

clean:
	cargo clean
