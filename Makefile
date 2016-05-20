DESTDIR = /usr/bin

all: gtk cli

gtk:
	cargo build --release --features "enable_gtk"
	mv target/release/tv-renamer target/release/tv-renamer-gtk

cli:
	cargo build --release

install: install-cli install-gtk

install-cli:
	cp target/release/tv-renamer "${DESTDIR}"

install-gtk:
	cp target/release/tv-renamer-gtk "${DESTDIR}"

uninstall:
	rm "${DESTDIR}/tv-renamer"
	rm "${DESTDIR}/tv-renamer-gtk"

clean:
	cargo clean
