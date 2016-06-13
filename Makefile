DESTDIR = /usr
version = $(shell awk 'NR == 3 {print substr($$3, 2, length($$3)-2)}' Cargo.toml)
arch = $(shell dpkg --print-architecture)

all: gtk

gtk:
	cargo build --release --features "enable_gtk"
	echo "#!/bin/sh" > "target/release/tv-renamer-gtk"
	echo "tv-renamer gtk" >> "target/release/tv-renamer-gtk"

cli:
	cargo build --release

install-cli: install-docs
	install -Dm 755 "target/release/tv-renamer" "${DESTDIR}/bin/"

install-gtk: install-cli install-docs
	install -Dm 755 "target/release/tv-renamer-gtk" "${DESTDIR}/bin/"
	install -Dm 644 "assets/tv-renamer.desktop" "${DESTDIR}/share/applications/"

install-docs:
	install -Dm 644 README.md "${DESTDIR}/share/doc/tv-renamer/README"
	install -Dm 644 LICENSE "${DESTDIR}/share/licenses/tv-renamer/COPYING"

create-tar:
	install -Dm 755 "target/release/tv-renamer-gtk" "tv-renamer/bin/tv-renamer-gtk"
	install -Dm 755 "target/release/tv-renamer" "tv-renamer/bin/tv-renamer"
	install -Dm 644 "assets/tv-renamer.desktop" "tv-renamer/share/applications/tv-renamer.desktop"
	tar cf - "tv-renamer" | xz -zf > "tv-renamer_$(version)_$(arch).tar.xz"

deb:
	sudo apt install libgtk-3-dev libssl-dev
	cargo build --release --features "enable_gtk"
	sed "2s/.*/Version: $(version)/g" -i "debian/DEBIAN/control"
	sed "7s/.*/Architecture: $(arch)/g" -i "debian/DEBIAN/control"
	install -Dsm 755 "target/release/tv-renamer" "debian/usr/bin/tv-renamer"
	install -Dm 755 "target/release/tv-renamer-gtk" "debian/usr/bin/tv-renamer-gtk"
	install -Dm 644 "assets/tv-renamer.desktop" "debian/usr/share/applications/tv-renamer.desktop"
	install -Dm 644 README.md "debian/usr/share/doc/tv-renamer/README"
	install -Dm 644 LICENSE "debian/usr/share/licenses/tv-renamer/COPYING"
	fakeroot dpkg-deb --build debian tv-renamer_$(version)_$(arch).deb

install-deb: debian
	sudo dpkg -i tv-renamer_$(version)_$(arch).deb

uninstall:
	rm "${DESTDIR}/tv-renamer"
	rm "${DESTDIR}/tv-renamer-gtk"

clean:
	cargo clean
